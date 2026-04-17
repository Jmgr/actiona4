use std::{path::Path, process::Command};

use color_eyre::{
    Result,
    eyre::{WrapErr, eyre},
};
use swc_common::{FileName, SourceMap, sync::Lrc};
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax, lexer::Lexer};

use crate::util::run_command;

fn validate_typescript_declarations(path: &Path) -> Result<()> {
    let code = std::fs::read_to_string(path).wrap_err_with(|| {
        format!(
            "Failed to read generated TypeScript declarations at {}.",
            path.display()
        )
    })?;

    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(FileName::Real(path.to_path_buf()).into(), code);
    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            dts: true,
            ..Default::default()
        }),
        EsVersion::Es2020,
        StringInput::from(&*fm),
        None,
    );
    let mut parser = Parser::new_from(lexer);

    let parse_result = parser.parse_program();
    let parser_errors = parser.take_errors();

    if parse_result.is_err() || !parser_errors.is_empty() {
        return Err(eyre!(
            "Generated TypeScript declarations are not valid syntax: result={parse_result:?}, errors={parser_errors:?}"
        ));
    }

    Ok(())
}

pub async fn generate_docs(workspace_root: &Path) -> Result<()> {
    let rustdoc_json_path = workspace_root.join("target/doc/actiona_core.json");
    let output_path = workspace_root.join("target/doc/index.d.ts");
    let run_assets_directory = workspace_root.join("crates/run/assets");

    run_command(
        Command::new("cargo")
            .arg("+nightly")
            .arg("rustdoc")
            .arg("--package")
            .arg("core")
            .arg("--lib")
            .arg("--")
            .arg("--output-format")
            .arg("json")
            .arg("-Z")
            .arg("unstable-options")
            .current_dir(workspace_root),
        "Failed to generate rustdoc JSON for the core crate.",
    )?;

    run_command(
        Command::new("cargo")
            .arg("run")
            .arg("--package")
            .arg("doc-generator")
            .arg("--")
            .arg(&rustdoc_json_path)
            .arg(&output_path)
            .current_dir(workspace_root),
        "Failed to generate TypeScript declarations.",
    )?;

    validate_typescript_declarations(&output_path)?;

    tokio::fs::create_dir_all(&run_assets_directory).await?;
    tokio::fs::copy(&output_path, run_assets_directory.join("index.d.ts")).await?;

    Ok(())
}
