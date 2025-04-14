use std::{
    fs,
    io::{self, Read},
    path::Path,
    sync::LazyLock,
};

use color_eyre::{Result, eyre::Context};
use flate2::read::GzDecoder;
use tracing::warn;

const TSCONFIG: &str = include_str!("../assets/tsconfig.json");
const INDEX_DTS_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/index.d.ts.gz"));
static INDEX_DTS: LazyLock<String> = LazyLock::new(|| {
    let mut decoder = GzDecoder::new(INDEX_DTS_GZ);
    let mut text = String::new();
    decoder
        .read_to_string(&mut text)
        .expect("Failed to decode embedded index.d.ts.gz");
    text
});

const STARTER_SCRIPT: &str = r#"// Welcome to Actiona!
// Run this script with: actiona-run script.ts (or actiona-run run script.ts)

println("Hello from Actiona!");
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
    let index_dts = INDEX_DTS.as_str();
    let needs_write = match fs::read_to_string(&dts_path) {
        Ok(existing) => existing != index_dts,
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Err(error) => {
            warn!(
                script_path = %script_path.display(),
                path = %dts_path.display(),
                error = %error,
                "failed to read index.d.ts while checking if it needs an update"
            );
            false
        }
    };
    // Don't create index.d.ts if it doesn't exist; that's what `init` is for.

    if needs_write {
        fs::write(&dts_path, index_dts).context("updating index.d.ts")?;
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
    let index_dts = INDEX_DTS.as_str();
    let action = if dts_path.exists() {
        let existing = fs::read_to_string(&dts_path).context("reading existing index.d.ts")?;
        if existing == index_dts {
            eprintln!("Skipped index.d.ts (already up to date)");
            return Ok(());
        }
        "Updated"
    } else {
        "Created"
    };

    fs::write(&dts_path, index_dts).context("writing index.d.ts")?;
    eprintln!("{action} {}", dts_path.display());

    Ok(())
}

fn write_starter_script(path: &Path) -> Result<()> {
    // Skip if any .ts file already exists (excluding .d.ts declaration files)
    let mut has_ts_files = false;
    for entry in fs::read_dir(path).context("reading project directory")? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warn!(
                    project_dir = %path.display(),
                    error = %error,
                    "failed to read an entry while scanning for existing .ts files"
                );
                continue;
            }
        };

        let entry_path = entry.path();
        if entry_path.extension().is_some_and(|ext| ext == "ts")
            && !entry_path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().ends_with(".d.ts"))
        {
            has_ts_files = true;
            break;
        }
    }

    if has_ts_files {
        return Ok(());
    }

    let script_path = path.join("script.ts");
    fs::write(&script_path, STARTER_SCRIPT).context("writing script.ts")?;
    eprintln!("Created {}", script_path.display());

    Ok(())
}
