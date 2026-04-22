use std::sync::OnceLock;

fn actiona_run_bin_path() -> std::path::PathBuf {
    // current_exe → target/{profile}/deps/<test-exe>
    // parent()    → target/{profile}/deps/
    // parent()    → target/{profile}/
    let mut path = std::env::current_exe()
        .expect("cannot determine test binary path")
        .parent()
        .expect("no parent")
        .parent()
        .expect("no grandparent")
        .to_path_buf();

    path.push(if cfg!(windows) {
        "actiona-run.exe"
    } else {
        "actiona-run"
    });

    path
}

fn ensure_actiona_run_bin_exists(path: &std::path::Path) {
    static BUILD_ONCE: OnceLock<()> = OnceLock::new();

    if path.exists() {
        return;
    }

    BUILD_ONCE.get_or_init(|| {
        let status = std::process::Command::new("cargo")
            .args(["build", "-p", "run", "--bin", "actiona-run"])
            .status()
            .expect("failed to spawn `cargo build` for actiona-run");

        assert!(
            status.success(),
            "`cargo build -p run --bin actiona-run` failed with status {status}"
        );
    });
}

/// Return the path to the actiona-run binary built by cargo.
///
/// Walks up from the running test binary (target/debug/deps/<exe>) to
/// target/debug/ and appends the binary name. If the binary is missing,
/// build it on demand so `cargo test` works without a separate pre-build step.
pub fn actiona_run_bin() -> std::path::PathBuf {
    let path = actiona_run_bin_path();
    ensure_actiona_run_bin_exists(&path);
    path
}
