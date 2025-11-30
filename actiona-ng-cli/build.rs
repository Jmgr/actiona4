fn main() {
    tauri_build::build();

    built::write_built_file().expect("Failed to acquire build-time information");
}
