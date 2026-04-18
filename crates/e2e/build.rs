use std::{env, fs, path::Path};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let scripts_dir = Path::new(&manifest_dir).join("scripts");

    println!("cargo:rerun-if-changed={}", scripts_dir.display());

    let mut entries: Vec<_> = fs::read_dir(&scripts_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.ends_with(".ts") && !name.ends_with(".d.ts") && name != "helpers.ts"
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut out = String::new();

    for entry in entries {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        let stem = name.strip_suffix(".ts").unwrap();

        out.push_str(&format!(
            "#[test]\nfn {stem}() {{\n    common::run(\"{name}\").success();\n}}\n\n"
        ));
    }

    fs::write(Path::new(&out_dir).join("generated_tests.rs"), out).unwrap();
}
