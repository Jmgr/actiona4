use std::{env, error::Error, fmt::Write as _, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let out_dir = env::var("OUT_DIR")?;
    let scripts_dir = Path::new(&manifest_dir).join("scripts");

    println!("cargo:rerun-if-changed={}", scripts_dir.display());

    let mut entries: Vec<_> = fs::read_dir(&scripts_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            name.ends_with(".ts") && !name.ends_with(".d.ts") && name != "helpers.ts"
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut out = String::from("#[cfg(test)]\nmod tests {\n    use super::common;\n\n");

    for entry in entries {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        let Some(stem) = name.strip_suffix(".ts") else {
            continue;
        };

        _ = write!(
            out,
            "    #[test]\n    fn {stem}() {{\n        common::run(\"{name}\").success();\n    }}\n\n"
        );
    }

    out.push('}');

    fs::write(Path::new(&out_dir).join("generated_tests.rs"), out)?;
    Ok(())
}
