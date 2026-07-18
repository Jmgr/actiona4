use std::string::String as StdString;

use swc_common::{
    FileName, GLOBALS, Globals, Mark, source_map::DefaultSourceMapGenConfig, sync::Lrc,
};
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::{Config, Emitter, text_writer::JsWriter};
use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer};
use swc_ecma_transforms_base::{fixer::fixer, hygiene::hygiene, resolver};
use swc_ecma_transforms_typescript::strip;
use thiserror::Error;

/// Errors produced while transpiling TypeScript to JavaScript.
#[derive(Debug, Error)]
pub enum TranspileError {
    /// A parse error whose rich SWC diagnostic was already emitted to the terminal.
    #[error("{message}")]
    Emitted { message: String },

    /// A parse error surfaced in silent mode; nothing was emitted to the terminal.
    #[error("{0}")]
    Parse(String),

    /// A code generation or output-encoding failure.
    #[error("code generation failed: {0}")]
    Codegen(String),
}

impl TranspileError {
    /// Whether a rich diagnostic for this error was already printed to the terminal,
    /// meaning the caller should not print its own fallback message.
    pub(crate) const fn diagnostics_already_emitted(&self) -> bool {
        matches!(self, Self::Emitted { .. })
    }
}

pub(crate) struct TsToJs {
    js_code: String,
    sourcemap: swc_sourcemap::SourceMap,
    filename: String,
}

impl TsToJs {
    pub fn new(code: &str, filename: &str) -> Result<Self, TranspileError> {
        Self::new_with_diagnostics(code, filename, true)
    }

    pub fn new_silent(code: &str, filename: &str) -> Result<Self, TranspileError> {
        Self::new_with_diagnostics(code, filename, false)
    }

    fn new_with_diagnostics(
        code: &str,
        filename: &str,
        emit_diagnostics: bool,
    ) -> Result<Self, TranspileError> {
        let (cm, handler) = super::new_tty_handler();

        let fm = cm.new_source_file(
            Lrc::new(FileName::Custom(filename.to_string())),
            code.to_string(),
        );

        let lexer = Lexer::new(
            Syntax::Typescript(TsSyntax::default()),
            EsVersion::Es2020,
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);

        let module = if emit_diagnostics {
            for e in parser.take_errors() {
                e.into_diagnostic(&handler).emit();
            }

            parser.parse_program().map_err(|e| {
                let message = e.kind().msg().into_owned();
                e.into_diagnostic(&handler).emit();
                TranspileError::Emitted { message }
            })?
        } else {
            let _ = parser.take_errors();
            parser
                .parse_program()
                .map_err(|e| TranspileError::Parse(e.kind().msg().into_owned()))?
        };

        let globals = Globals::default();
        let (code, srcmap) = GLOBALS.set(
            &globals,
            || -> Result<(StdString, swc_sourcemap::SourceMap), TranspileError> {
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
                        cfg: Config::default(),
                        cm: cm.clone(),
                        comments: None,
                        wr: JsWriter::new(cm.clone(), "\n", &mut code, Some(&mut srcmap)),
                    };

                    emitter
                        .emit_program(&program)
                        .map_err(|e| TranspileError::Codegen(e.to_string()))?;
                }

                let srcmap = cm.build_source_map(&srcmap, None, DefaultSourceMapGenConfig);
                let code =
                    String::from_utf8(code).map_err(|e| TranspileError::Codegen(e.to_string()))?;

                Ok((code, srcmap))
            },
        )?;

        Ok(Self {
            js_code: code,
            sourcemap: srcmap,
            filename: filename.to_string(),
        })
    }

    /// Creates a passthrough entry for plain JS files (no transpilation, no sourcemap).
    pub fn passthrough(code: &str, filename: &str) -> Self {
        Self {
            js_code: code.to_string(),
            sourcemap: swc_sourcemap::SourceMap::new(None, vec![], vec![], vec![], None),
            filename: filename.to_string(),
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn code(&self) -> &str {
        &self.js_code
    }

    pub fn filename(&self) -> &str {
        &self.filename
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
        let token = self
            .sourcemap
            .lookup_token(zero_based_line, zero_based_col)?;

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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_already_emitted_only_for_emitted() {
        assert!(
            TranspileError::Emitted {
                message: "boom".to_string()
            }
            .diagnostics_already_emitted()
        );
        assert!(!TranspileError::Parse("boom".to_string()).diagnostics_already_emitted());
        assert!(!TranspileError::Codegen("boom".to_string()).diagnostics_already_emitted());
    }

    #[test]
    fn silent_parse_error_is_parse_variant() {
        // `TsToJs` isn't `Debug`, so match instead of `unwrap_err`.
        let Err(err) = TsToJs::new_silent("function {{{ invalid", "bad.ts") else {
            panic!("expected a parse error");
        };
        assert!(
            matches!(err, TranspileError::Parse(_)),
            "expected a Parse variant, got: {err:?}"
        );
        assert!(!err.diagnostics_already_emitted());
    }

    #[test]
    fn valid_typescript_transpiles() {
        let ts_to_js = TsToJs::new_silent("const x: number = 1; x;", "ok.ts").unwrap();
        assert!(ts_to_js.code().contains("const x"));
        assert_eq!(ts_to_js.filename(), "ok.ts");
    }
}
