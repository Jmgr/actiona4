use eyre::Result;
use swc_common::{
    FileName, GLOBALS, Globals, Mark, SourceMap,
    errors::{ColorConfig, Handler},
    sync::Lrc,
};
use swc_ecma_ast::EsVersion;
use swc_ecma_codegen::{Emitter, text_writer::JsWriter};
use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer};
use swc_ecma_transforms_base::{fixer::fixer, hygiene::hygiene, resolver};
use swc_ecma_transforms_typescript::strip;

pub struct TsToJs {
    js_code: String,
    sourcemap: sourcemap::SourceMap,
}

impl TsToJs {
    pub fn new(code: &str) -> Result<Self> {
        let cm: Lrc<SourceMap> = Default::default();
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
            .expect("failed to parse module.");

        let globals = Globals::default();
        let (code, srcmap) = GLOBALS.set(&globals, || {
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

                emitter.emit_program(&program).unwrap();
            }

            let srcmap = cm.build_source_map(&srcmap);
            let code = String::from_utf8(code).expect("codegen generated non-utf8 output");

            (code, srcmap)
        });

        Ok(Self {
            js_code: code,
            sourcemap: srcmap,
        })
    }

    pub fn code(&self) -> &str {
        &self.js_code
    }
}
