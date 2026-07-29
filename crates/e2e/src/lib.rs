use std::{
    collections::BTreeSet,
    env,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};

use parking_lot::Mutex;

const ACTIONA_RUN_OVERRIDE_ENV: &str = "ACTIONA4_E2E_RUNNER";

fn target_profile_dir() -> PathBuf {
    // current_exe -> target/{profile}/deps/<test-exe>
    // parent()    -> target/{profile}/deps/
    // parent()    -> target/{profile}/
    env::current_exe()
        .expect("cannot determine test binary path")
        .parent()
        .expect("no parent")
        .parent()
        .expect("no grandparent")
        .to_path_buf()
}

fn actiona_run_bin_path() -> PathBuf {
    let mut path = target_profile_dir();

    path.push(if cfg!(windows) {
        "actiona-run.exe" // TODO: we can't use an env var for this?
    } else {
        "actiona-run"
    });

    path
}

fn extension_bin_path(name: &str) -> PathBuf {
    let mut path = target_profile_dir();

    path.push(format!("extension-{name}{}", env::consts::EXE_SUFFIX));

    path
}

fn ensure_actiona_run_bin_exists(path: &Path) {
    static BUILD_ONCE: OnceLock<()> = OnceLock::new();

    if path.exists() {
        return;
    }

    BUILD_ONCE.get_or_init(|| {
        let status = Command::new("cargo")
            .args(["build", "-p", "run", "--bin", "actiona-run"])
            .status()
            .expect("failed to spawn `cargo build` for actiona-run");

        assert!(
            status.success(),
            "`cargo build -p run --bin actiona-run` failed with status {status}"
        );
    });
}

fn ensure_extension_bin_exists(name: &str, path: &Path) {
    // Tests run in parallel threads, so remember what has already been built
    // rather than letting every test race on cargo's target-directory lock.
    static BUILT: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

    let mut built = BUILT.lock();
    if !built.insert(name.to_owned()) || path.exists() {
        return;
    }

    let package = format!("extension-{name}");
    let status = Command::new("cargo")
        .args(["build", "-p", &package])
        .status()
        .unwrap_or_else(|error| panic!("failed to spawn `cargo build` for {package}: {error}"));

    assert!(
        status.success(),
        "`cargo build -p {package}` failed with status {status}"
    );
}

/// Return the path to the actiona-run executable under test.
///
/// Uses the executable specified by `ACTIONA4_E2E_RUNNER` when set. Otherwise,
/// walks up from the running test binary (target/debug/deps/<exe>) to
/// target/debug/ and appends the binary name. If the binary is missing, build
/// it on demand so `cargo test` works without a separate pre-build step.
#[must_use]
pub fn actiona_run_bin() -> PathBuf {
    if let Some(path) = env::var_os(ACTIONA_RUN_OVERRIDE_ENV) {
        return PathBuf::from(path);
    }

    let path = actiona_run_bin_path();
    ensure_actiona_run_bin_exists(&path);
    path
}

/// Whether the test process should use a pre-built actiona-run executable.
///
/// This is used to exercise packaged artifacts such as the AppImage, which
/// provide their extension executables alongside the main binary.
#[must_use]
pub fn actiona_run_is_overridden() -> bool {
    env::var_os(ACTIONA_RUN_OVERRIDE_ENV).is_some()
}

/// Return the path to an extension binary built by cargo, building it on
/// demand so `cargo test` works without a separate pre-build step.
///
/// Extensions are discovered next to `actiona-run`, which is where cargo puts
/// them, so making sure they exist is all the wiring the tests need.
#[must_use]
pub fn extension_bin(name: &str) -> PathBuf {
    let path = extension_bin_path(name);
    ensure_extension_bin_exists(name, &path);
    path
}
