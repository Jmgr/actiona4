use std::env;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=editor.rc");
        println!("cargo:rerun-if-changed=../core/icons/icon.ico");

        embed_resource::compile("editor.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to compile editor resources");
    }

    built::write_built_file().expect("Failed to acquire build-time information");
}
