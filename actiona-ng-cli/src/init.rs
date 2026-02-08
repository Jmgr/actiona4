use std::{fs, path::Path};

use color_eyre::{Result, eyre::Context};

const TSCONFIG: &str = include_str!("../assets/tsconfig.json");
const INDEX_DTS: &str = include_str!("../assets/index.d.ts");

const STARTER_SCRIPT: &str = r#"// Welcome to Actiona!
// Run this script with: actiona-ng-cli run script.ts

console.log("Hello from Actiona!");
"#;

pub fn run(path: &Path) -> Result<()> {
    fs::create_dir_all(path).context("creating project directory")?;

    write_tsconfig(path)?;
    write_index_dts(path)?;
    write_starter_script(path)?;

    Ok(())
}

/// Ensures `index.d.ts` in the script's directory is up to date with the embedded version.
/// Called automatically before running a script.
pub fn ensure_index_dts(script_path: &Path) -> Result<()> {
    let dir = script_path.parent().unwrap_or_else(|| Path::new("."));

    let dts_path = dir.join("index.d.ts");

    let needs_write = match fs::read_to_string(&dts_path) {
        Ok(existing) => existing != INDEX_DTS,
        Err(_) => false, // Don't create index.d.ts if it doesn't exist — that's what `init` is for
    };

    if needs_write {
        fs::write(&dts_path, INDEX_DTS).context("updating index.d.ts")?;
        eprintln!("Updated {}", dts_path.display());
    }

    Ok(())
}

fn write_tsconfig(path: &Path) -> Result<()> {
    let tsconfig_path = path.join("tsconfig.json");

    if tsconfig_path.exists() {
        eprintln!("Skipped tsconfig.json (already exists)");
    } else {
        fs::write(&tsconfig_path, TSCONFIG).context("writing tsconfig.json")?;
        eprintln!("Created {}", tsconfig_path.display());
    }

    Ok(())
}

fn write_index_dts(path: &Path) -> Result<()> {
    let dts_path = path.join("index.d.ts");

    let action = if dts_path.exists() {
        let existing = fs::read_to_string(&dts_path).context("reading existing index.d.ts")?;
        if existing == INDEX_DTS {
            eprintln!("Skipped index.d.ts (already up to date)");
            return Ok(());
        }
        "Updated"
    } else {
        "Created"
    };

    fs::write(&dts_path, INDEX_DTS).context("writing index.d.ts")?;
    eprintln!("{action} {}", dts_path.display());

    Ok(())
}

fn write_starter_script(path: &Path) -> Result<()> {
    // Skip if any .ts file already exists (excluding .d.ts declaration files)
    let has_ts_files = fs::read_dir(path)
        .context("reading project directory")?
        .filter_map(|e| e.ok())
        .any(|e| {
            let path = e.path();
            path.extension().is_some_and(|ext| ext == "ts")
                && !path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().ends_with(".d.ts"))
        });

    if has_ts_files {
        return Ok(());
    }

    let script_path = path.join("script.ts");
    fs::write(&script_path, STARTER_SCRIPT).context("writing script.ts")?;
    eprintln!("Created {}", script_path.display());

    Ok(())
}
