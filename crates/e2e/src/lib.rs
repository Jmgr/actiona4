use std::sync::OnceLock;

fn target_profile_dir() -> std::path::PathBuf {
    // current_exe -> target/{profile}/deps/<test-exe>
    // parent()    -> target/{profile}/deps/
    // parent()    -> target/{profile}/
    std::env::current_exe()
        .expect("cannot determine test binary path")
        .parent()
        .expect("no parent")
        .parent()
        .expect("no grandparent")
        .to_path_buf()
}

fn actiona_run_bin_path() -> std::path::PathBuf {
    let mut path = target_profile_dir();

    path.push(if cfg!(windows) {
        "actiona-run.exe" // TODO: we can't use an env var for this?
    } else {
        "actiona-run"
    });

    path
}

fn selection_extension_bin_path() -> std::path::PathBuf {
    let mut path = target_profile_dir();

    path.push(if cfg!(windows) {
        "extension-selection.exe" //TODO
    } else {
        "extension-selection"
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

fn ensure_selection_extension_bin_exists(path: &std::path::Path) {
    static BUILD_ONCE: OnceLock<()> = OnceLock::new();

    if path.exists() {
        return;
    }

    BUILD_ONCE.get_or_init(|| {
        let status = std::process::Command::new("cargo")
            .args(["build", "-p", "selection", "--bin", "extension-selection"])
            .status()
            .expect("failed to spawn `cargo build` for extension-selection");

        assert!(
            status.success(),
            "`cargo build -p selection --bin extension-selection` failed with status {status}"
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

/// Return the path to the selection extension binary built by cargo.
///
/// This is intentionally used only by manual/ignored e2e tests that exercise
/// interactive overlay selection.
pub fn selection_extension_bin() -> std::path::PathBuf {
    let path = selection_extension_bin_path();
    ensure_selection_extension_bin_exists(&path);
    path
}
