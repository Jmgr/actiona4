use std::io::Write as _;

use assert_cmd::prelude::*;

const HELPERS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/scripts/helpers.ts"));

/// Combines helpers.ts with the named test script into a single temp file
/// that actiona-run can execute directly.
fn script_file(name: &str) -> tempfile::NamedTempFile {
    let script_path = format!("{}/scripts/{name}", env!("CARGO_MANIFEST_DIR"));
    let script = std::fs::read_to_string(&script_path)
        .unwrap_or_else(|e| panic!("failed to read {script_path}: {e}"));

    let mut tmp = tempfile::Builder::new()
        .suffix(".ts")
        .tempfile()
        .expect("failed to create temp file");
    writeln!(tmp, "{HELPERS}").unwrap();
    writeln!(tmp, "{script}").unwrap();
    tmp
}

/// Run a test script through actiona-run and return the assert handle.
///
/// The temp file lives until the process exits (actiona-run reads the file
/// synchronously before `assert()` returns).
pub fn run(name: &str) -> assert_cmd::assert::Assert {
    let script = script_file(name);
    std::process::Command::new(e2e::actiona_run_bin())
        .arg(script.path())
        .assert()
}
