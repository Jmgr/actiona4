use std::process::Command;

fn cargo_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_actiona4-run"))
}

#[test]
fn init_creates_project_and_starter_script_runs() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let project_path = dir.path().join("test-project");

    // Run init
    let output = cargo_bin()
        .args(["init", project_path.to_str().unwrap()])
        .output()
        .expect("failed to run init");
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify files exist
    assert!(
        project_path.join("tsconfig.json").exists(),
        "tsconfig.json not created"
    );
    assert!(
        project_path.join("index.d.ts").exists(),
        "index.d.ts not created"
    );
    assert!(
        project_path.join("script.ts").exists(),
        "script.ts not created"
    );

    // Run the starter script
    let output = cargo_bin()
        .args(["run", project_path.join("script.ts").to_str().unwrap()])
        .output()
        .expect("failed to run starter script");
    assert!(
        output.status.success(),
        "starter script failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello from Actiona!"),
        "unexpected output: {stdout}"
    );
}

#[test]
fn init_is_idempotent() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let project_path = dir.path().join("test-project");

    // Run init twice
    let output1 = cargo_bin()
        .args(["init", project_path.to_str().unwrap()])
        .output()
        .expect("failed to run init");
    assert!(output1.status.success());

    let output2 = cargo_bin()
        .args(["init", project_path.to_str().unwrap()])
        .output()
        .expect("failed to run init second time");
    assert!(output2.status.success());

    let stderr = String::from_utf8_lossy(&output2.stderr);
    assert!(
        stderr.contains("Skipped tsconfig.json"),
        "should skip existing tsconfig.json"
    );
    assert!(
        stderr.contains("Skipped index.d.ts"),
        "should skip up-to-date index.d.ts"
    );
    // script.ts already exists so starter script should not be re-created
    assert!(!stderr.contains("Created") || !stderr.contains("script.ts"));
}

#[test]
fn run_auto_updates_outdated_index_dts() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let project_path = dir.path().join("test-project");

    // Init project
    let output = cargo_bin()
        .args(["init", project_path.to_str().unwrap()])
        .output()
        .expect("failed to run init");
    assert!(output.status.success());

    // Tamper with index.d.ts
    std::fs::write(project_path.join("index.d.ts"), "// outdated").unwrap();

    // Run script — should auto-update index.d.ts
    let output = cargo_bin()
        .args(["run", project_path.join("script.ts").to_str().unwrap()])
        .output()
        .expect("failed to run script");
    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Updated"),
        "should report updating index.d.ts"
    );

    // Verify index.d.ts was restored
    let content = std::fs::read_to_string(project_path.join("index.d.ts")).unwrap();
    assert!(
        content.len() > 1000,
        "index.d.ts should be restored to full content"
    );
}
