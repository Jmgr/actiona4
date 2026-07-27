fn main() {
    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=opencv.rc");
        println!("cargo:rerun-if-changed=..\\..\\core\\icons\\icon.ico");
    }

    built::write_built_file().expect("Failed to acquire build-time information");

    #[cfg(windows)]
    {
        embed_resource::compile("opencv.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to compile opencv extension resources");

        build_support::embed_windows_manifest();
    }
}
