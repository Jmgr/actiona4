use std::io::Write as _;

use assert_cmd::prelude::*;
use httptest::{
    Expectation, Server, all_of,
    matchers::{contains, matches, request},
    responders::status_code,
};

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
    writeln!(
        tmp,
        "const __e2eManifestDir = {:?};",
        env!("CARGO_MANIFEST_DIR")
    )
    .unwrap();
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
    let mut command = std::process::Command::new(e2e::actiona_run_bin());
    command.arg(script.path());

    let _web_server = if name == "web.ts" {
        Some(configure_web_server(&mut command))
    } else {
        None
    };

    command.assert()
}

fn configure_web_server(command: &mut std::process::Command) -> Server {
    let server = Server::run();
    let image_fixture_path = format!(
        "{}/../core/test-data/Crown_icon_transparent.png",
        env!("CARGO_MANIFEST_DIR")
    );
    let image_bytes = std::fs::read(&image_fixture_path).unwrap_or_else(|error| {
        panic!("failed to read web image fixture {image_fixture_path}: {error}")
    });

    server.expect(
        Expectation::matching(request::method_path("GET", "/text"))
            .respond_with(status_code(200).body("hello")),
    );

    server.expect(
        Expectation::matching(request::method_path("GET", "/image.png")).respond_with(
            status_code(200)
                .append_header("Content-Type", "image/png")
                .body(image_bytes.clone()),
        ),
    );

    server.expect(
        Expectation::matching(request::method_path("GET", "/binary")).respond_with(
            status_code(200)
                .append_header("Content-Type", "application/octet-stream")
                .append_header("Content-Length", image_bytes.len().to_string())
                .body(image_bytes.clone()),
        ),
    );

    server.expect(
        Expectation::matching(all_of![
            request::method_path("POST", "/auth"),
            request::headers(contains((
                "authorization",
                "Basic dXNlcjpwYXNzd29yZA==".to_string()
            )))
        ])
        .respond_with(status_code(200).body("hello")),
    );

    server.expect(
        Expectation::matching(request::method_path("GET", "/progress")).respond_with(
            status_code(200)
                .append_header("Content-Type", "application/octet-stream")
                .append_header("Content-Length", image_bytes.len().to_string())
                .body(image_bytes.clone()),
        ),
    );

    server.expect(
        Expectation::matching(request::method_path("GET", "/download-file")).respond_with(
            status_code(200)
                .append_header("Content-Type", "image/png")
                .append_header(
                    "Content-Disposition",
                    r#"attachment; filename="example.png""#,
                )
                .body(image_bytes.clone()),
        ),
    );

    server.expect(
        Expectation::matching(all_of![
            request::method_path("POST", "/upload-text"),
            request::headers(contains(("content-type", matches("multipart/form-data")))),
            request::body(matches("name=\"title\"")),
            request::body(matches("hello multipart")),
        ])
        .respond_with(status_code(200).body("ok")),
    );

    server.expect(
        Expectation::matching(all_of![
            request::method_path("POST", "/upload-bytes"),
            request::headers(contains(("content-type", matches("multipart/form-data")))),
            request::body(matches("name=\"payload\"")),
            request::body(matches("hello-bytes")),
        ])
        .respond_with(status_code(200).body("ok")),
    );

    server.expect(
        Expectation::matching(all_of![
            request::method_path("POST", "/upload-file"),
            request::headers(contains(("content-type", matches("multipart/form-data")))),
            request::body(matches("name=\"file\"")),
            request::body(matches("filename=\"multipart_test.txt\"")),
            request::body(matches("file-content")),
        ])
        .respond_with(status_code(200).body("ok")),
    );

    command.env("ACTIONA4_E2E_WEB_TEXT_URL", server.url("/text").to_string());
    command.env(
        "ACTIONA4_E2E_WEB_IMAGE_URL",
        server.url("/image.png").to_string(),
    );
    command.env(
        "ACTIONA4_E2E_WEB_BINARY_URL",
        server.url("/binary").to_string(),
    );
    command.env("ACTIONA4_E2E_WEB_AUTH_URL", server.url("/auth").to_string());
    command.env(
        "ACTIONA4_E2E_WEB_PROGRESS_URL",
        server.url("/progress").to_string(),
    );
    command.env(
        "ACTIONA4_E2E_WEB_DOWNLOAD_FILE_URL",
        server.url("/download-file").to_string(),
    );
    command.env(
        "ACTIONA4_E2E_WEB_MULTIPART_TEXT_URL",
        server.url("/upload-text").to_string(),
    );
    command.env(
        "ACTIONA4_E2E_WEB_MULTIPART_BYTES_URL",
        server.url("/upload-bytes").to_string(),
    );
    command.env(
        "ACTIONA4_E2E_WEB_MULTIPART_FILE_URL",
        server.url("/upload-file").to_string(),
    );

    server
}
