fn main() {
    // Task dialogs need Common Controls 6, which the test executables request in their manifest.
    #[cfg(windows)]
    build_support::embed_windows_manifest();
}
