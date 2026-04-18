/// Return the path to the actiona-run binary built by cargo.
///
/// Walks up from the running test binary (target/debug/deps/<exe>) to
/// target/debug/ and appends the binary name.
pub fn actiona_run_bin() -> std::path::PathBuf {
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
