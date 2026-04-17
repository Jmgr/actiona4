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

        let content = fs::read_to_string(entry.path()).unwrap();
        let guard = content
            .lines()
            .next()
            .and_then(|line| line.strip_prefix("// @guard: "))
            .unwrap_or("")
            .trim();

        out.push_str(&render_test(stem, &name, guard));
    }

    fs::write(Path::new(&out_dir).join("generated_tests.rs"), out).unwrap();
}

fn render_test(stem: &str, name: &str, guard: &str) -> String {
    let body = match guard {
        "e2e::require_not_windows!();" => format!(
            "    #[cfg(windows)]\n    {{\n        println!(\"skipping: not supported on Windows\");\n    }}\n    #[cfg(not(windows))]\n    {{\n        common::run(\"{name}\").success();\n    }}\n"
        ),
        "e2e::require_windows!();" => format!(
            "    #[cfg(not(windows))]\n    {{\n        println!(\"skipping: requires Windows\");\n    }}\n    #[cfg(windows)]\n    {{\n        common::run(\"{name}\").success();\n    }}\n"
        ),
        "" => format!("    common::run(\"{name}\").success();\n"),
        _ => format!("    {guard}\n    common::run(\"{name}\").success();\n"),
    };

    format!("#[test]\nfn {stem}() {{\n{body}}}\n\n")
}
