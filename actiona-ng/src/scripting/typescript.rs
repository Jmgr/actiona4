use color_eyre::{Result, eyre::eyre};
use swc_common::{
    FileName, FilePathMapping, GLOBALS, Globals, Mark, SourceMap,
    errors::{ColorConfig, Handler},
    source_map::DefaultSourceMapGenConfig,
    sync::Lrc,
};
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::{Emitter, text_writer::JsWriter};
use swc_ecma_parser::{Parser, StringInput, Syntax, lexer::Lexer};
use swc_ecma_transforms_base::{fixer::fixer, hygiene::hygiene, resolver};
use swc_ecma_transforms_typescript::strip;

pub(crate) struct TsToJs {
    js_code: String,
    sourcemap: swc_sourcemap::SourceMap,
}

impl TsToJs {
    pub fn new(code: &str) -> Result<Self> {
        let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
        let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

        let fm = cm.new_source_file(
            Lrc::new(FileName::Custom("file.ts".to_string())),
            code.to_string(),
        );

        let lexer = Lexer::new(
            Syntax::Typescript(Default::default()),
            EsVersion::Es2020,
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);

        for e in parser.take_errors() {
            e.into_diagnostic(&handler).emit();
        }

        let module = parser
            .parse_program()
            .map_err(|e| e.into_diagnostic(&handler).emit())
            .map_err(|_| eyre!("Module parsing failed"))?;

        let globals = Globals::default();
        let (code, srcmap) = GLOBALS.set(
            &globals,
            || -> Result<(std::string::String, swc_sourcemap::SourceMap)> {
                let unresolved_mark = Mark::new();
                let top_level_mark = Mark::new();

                // Optionally transforms decorators here before the resolver pass
                // as it might produce runtime declarations.

                // Conduct identifier scope analysis
                let module = module.apply(resolver(unresolved_mark, top_level_mark, true));

                // Remove typescript types
                let module = module.apply(strip(unresolved_mark, top_level_mark));

                // Fix up any identifiers with the same name, but different contexts
                let module = module.apply(hygiene());

                // Ensure that we have enough parenthesis.
                let program = module.apply(fixer(None));

                let mut code = Vec::new();
                let mut srcmap = Vec::new();

                {
                    let mut emitter = Emitter {
                        cfg: Default::default(),
                        cm: cm.clone(),
                        comments: None,
                        wr: JsWriter::new(cm.clone(), "\n", &mut code, Some(&mut srcmap)),
                    };

                    emitter.emit_program(&program)?;
                }

                let srcmap = cm.build_source_map(&srcmap, None, DefaultSourceMapGenConfig);
                let code = String::from_utf8(code)?;

                Ok((code, srcmap))
            },
        )?;

        Ok(Self {
            js_code: code,
            sourcemap: srcmap,
        })
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn code(&self) -> &str {
        &self.js_code
    }

    /// Looks up the original TypeScript location for a given JavaScript line and column.
    ///
    /// # Arguments
    ///
    /// * `js_line` - The 1-based line number in the generated JavaScript code.
    /// * `js_col` - The 1-based column number in the generated JavaScript code.
    ///
    /// # Returns
    ///
    /// An `Option` containing a tuple `(filename, ts_line, ts_col)` if a mapping exists:
    /// * `filename`: The original source file name (e.g., "file.ts").
    /// * `ts_line`: The 1-based line number in the original TypeScript file.
    /// * `ts_col`: The 1-based column number in the original TypeScript file.
    ///
    /// Returns `None` if no mapping is found for the given JavaScript location.
    pub fn lookup_source_location(&self, js_line: u32, js_col: u32) -> Option<(String, u32, u32)> {
        // Input validation: lines and columns are typically 1-based for users,
        // but the sourcemap crate expects 0-based.
        if js_line == 0 || js_col == 0 {
            return None; // Or handle as an error, 1-based indexing is conventional
        }

        // Convert 1-based user input to 0-based for the lookup
        let zero_based_line = js_line - 1;
        let zero_based_col = js_col - 1;

        // Perform the lookup using the sourcemap crate
        self.sourcemap
            .lookup_token(zero_based_line, zero_based_col)
            .and_then(|token| {
                // Check if the token has source information
                match (
                    token.get_source(),
                    token.get_src_line(),
                    token.get_src_col(),
                ) {
                    (Some(filename), src_line, src_col) => {
                        // Convert 0-based source location back to 1-based for the user
                        let one_based_ts_line = src_line + 1;
                        let one_based_ts_col = src_col + 1;
                        Some((filename.to_string(), one_based_ts_line, one_based_ts_col))
                    }
                    _ => None, // No mapping found for this specific token
                }
            })
    }
}
