fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=selection.rc");
        println!("cargo:rerun-if-changed=..\\..\\core\\icons\\icon.ico");
    }

    built::write_built_file().expect("Failed to acquire build-time information");

    #[cfg(windows)]
    {
        embed_resource::compile("selection.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to compile selection resources");

        build_support::embed_windows_manifest();
    }
}
