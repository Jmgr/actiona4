use std::{env, fs, path::PathBuf};

const PLACEHOLDER_PAGE: &str = "<!doctype html>
<html lang=\"en\">
    <head><title>Actiona 4</title></head>
    <body>
        <p>This build of the editor does not include its user interface.</p>
        <p>Build it with <code>cargo make release-editor</code>.</p>
    </body>
</html>
";

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=editor.rc");
        println!("cargo:rerun-if-changed=../core/icons/icon.ico");

        embed_resource::compile("editor.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to compile editor resources");
    }

    // Only release builds embed the web UI; debug builds load it from `trunk serve`.
    if env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_none() {
        select_webroot();
    }

    built::write_built_file().expect("Failed to acquire build-time information");
}

/// Points `EDITOR_WEBROOT`, the directory release builds embed, at `dist/`. Only `trunk build`
/// creates `dist/`, so when it is missing a placeholder page is embedded instead: a plain
/// `cargo build --release` of the workspace still works, and packaging builds the UI first.
fn select_webroot() {
    let dist_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is set"))
            .join("dist");
    // Also reruns when `dist/` is created or rebuilt, so the real UI replaces the placeholder.
    println!("cargo:rerun-if-changed={}", dist_dir.display());

    let webroot = if dist_dir.join("index.html").is_file() {
        dist_dir
    } else {
        println!(
            "cargo:warning=crates/editor/dist is missing, so this build embeds a placeholder \
             instead of the editor UI. Build the UI with `cargo make build-editor-ui-release`."
        );

        let placeholder_dir =
            PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set")).join("webroot");
        fs::create_dir_all(&placeholder_dir).expect("failed to create the placeholder webroot");
        fs::write(placeholder_dir.join("index.html"), PLACEHOLDER_PAGE)
            .expect("failed to write the placeholder webroot");
        placeholder_dir
    };

    println!("cargo:rustc-env=EDITOR_WEBROOT={}", webroot.display());
}
