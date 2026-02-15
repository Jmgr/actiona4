use std::process::Command;

fn cargo_bin() -> Command {
    #[cfg(all(windows, feature = "windows-console-bin"))]
    let exe = env!("CARGO_BIN_EXE_actiona-run-console");
    #[cfg(any(not(windows), all(windows, not(feature = "windows-console-bin"))))]
    let exe = env!("CARGO_BIN_EXE_actiona-run");

    Command::new(exe)
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
fn script_path_defaults_to_run_subcommand() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let project_path = dir.path().join("test-project");

    let output = cargo_bin()
        .args(["init", project_path.to_str().unwrap()])
        .output()
        .expect("failed to run init");
    assert!(output.status.success());

    let output = cargo_bin()
        .arg(project_path.join("script.ts").to_str().unwrap())
        .output()
        .expect("failed to run starter script without subcommand");
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
fn globals_mode_registers_on_global_scope() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let script_path = dir.path().join("globals.ts");

    std::fs::write(
        &script_path,
        r#"
// No pragma — globals mode
const checks = [
    ["mouse", typeof mouse, "object"],
    ["keyboard", typeof keyboard, "object"],
    ["Point", typeof Point, "function"],
    ["Key", typeof Key, "function"],
    ["sleep", typeof sleep, "function"],
    ["actiona", typeof actiona, "undefined"],
];

for (const [name, actual, expected] of checks) {
    if (actual !== expected) {
        throw new Error(`${name}: expected ${expected}, got ${actual}`);
    }
}
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .args(["run", script_path.to_str().unwrap()])
        .output()
        .expect("failed to run globals script");
    assert!(
        output.status.success(),
        "globals script failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn no_globals_mode_registers_under_actiona_namespace() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let script_path = dir.path().join("noglobals.ts");

    std::fs::write(
        &script_path,
        r#"//@actiona noglobals
const checks = [
    ["actiona", typeof actiona, "object"],
    ["actiona.mouse", typeof actiona.mouse, "object"],
    ["actiona.keyboard", typeof actiona.keyboard, "object"],
    ["actiona.Point", typeof actiona.Point, "function"],
    ["actiona.Key", typeof actiona.Key, "function"],
    ["actiona.sleep", typeof actiona.sleep, "function"],
    ["mouse", typeof mouse, "undefined"],
    ["keyboard", typeof keyboard, "undefined"],
];

for (const [name, actual, expected] of checks) {
    if (actual !== expected) {
        throw new Error(`${name}: expected ${expected}, got ${actual}`);
    }
}
"#,
    )
    .unwrap();

    let output = cargo_bin()
        .args(["run", script_path.to_str().unwrap()])
        .output()
        .expect("failed to run noglobals script");
    assert!(
        output.status.success(),
        "noglobals script failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
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
