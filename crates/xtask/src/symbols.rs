use std::{
    fs,
    fs::File,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
};

use color_eyre::{Result, eyre::eyre};
use tempfile::TempDir;
use zip::{ZipArchive, result::ZipError};

#[cfg(unix)]
const BINARY_NAMES: &[&str] = &["actiona-run", "selection"];

#[cfg(windows)]
const BINARY_NAMES: &[&str] = &["actiona-run", "actiona-runw", "selection"];

#[cfg(unix)]
const STACKWALK_MODULE_NAME: &str = "actiona-run";

#[cfg(windows)]
const STACKWALK_MODULE_NAME: &str = "actiona-run.exe";

struct SymbolFunction {
    start_offset: u64,
    end_offset: u64,
    name: String,
}

struct DumpMetadata {
    version: Option<String>,
    git_sha: Option<String>,
}

struct PreparedDumpArtifact {
    dump_path: PathBuf,
    metadata: Option<DumpMetadata>,
    _temp_dir: Option<TempDir>,
}

pub fn generate_symbols(workspace_root: &Path) -> Result<()> {
    let release_dir = workspace_root.join("target").join("release");
    let symbols_dir = workspace_root.join("target").join("symbols");
    fs::create_dir_all(&symbols_dir)?;

    for binary_name in BINARY_NAMES {
        let binary_path = binary_path(&release_dir, binary_name);
        let sym_path = symbols_dir.join(format!("{binary_name}.sym"));

        run_dump_syms(&binary_path, &sym_path)?;

        #[cfg(unix)]
        run_strip(&binary_path)?;
    }

    Ok(())
}

pub fn symbolicate(workspace_root: &Path, dump_path: &Path) -> Result<()> {
    let sym_path = workspace_root
        .join("target")
        .join("symbols")
        .join("actiona-run.sym");

    if !sym_path.exists() {
        return Err(eyre!(
            "Symbol file not found at {}. Run `cargo make symbols` first.",
            sym_path.display()
        ));
    }

    let symbol_functions = load_symbol_functions(&sym_path)?;
    let prepared_dump = prepare_dump_artifact(dump_path)?;
    print_dump_metadata(prepared_dump.metadata.as_ref());

    let output = Command::new("minidump-stackwalk")
        .arg(&prepared_dump.dump_path)
        .arg(&sym_path)
        .output()
        .map_err(|error| eyre!("Failed to run minidump-stackwalk: {error}"))?;

    if !output.stderr.is_empty() {
        io::stderr().write_all(&output.stderr)?;
    }

    let stackwalk_output = String::from_utf8_lossy(&output.stdout);
    let symbolicated_output = symbolize_stackwalk_output(&stackwalk_output, &symbol_functions);
    print!("{symbolicated_output}");
    if !symbolicated_output.ends_with('\n') {
        println!();
    }
    io::stdout().flush()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(eyre!(
            "minidump-stackwalk exited with status {}",
            output.status
        ))
    }
}

fn prepare_dump_artifact(dump_path: &Path) -> Result<PreparedDumpArtifact> {
    if is_zip_archive(dump_path) {
        return prepare_dump_archive(dump_path);
    }

    Ok(PreparedDumpArtifact {
        dump_path: dump_path.to_path_buf(),
        metadata: read_legacy_dump_metadata(dump_path)?,
        _temp_dir: None,
    })
}

fn is_zip_archive(dump_path: &Path) -> bool {
    dump_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
}

fn prepare_dump_archive(archive_path: &Path) -> Result<PreparedDumpArtifact> {
    let archive_file = File::open(archive_path)?;
    let mut zip_archive = ZipArchive::new(archive_file)?;
    let temp_dir = TempDir::new()?;
    let dump_entry_name = find_dump_entry_name(&mut zip_archive)?;
    let dump_file_name = Path::new(&dump_entry_name)
        .file_name()
        .ok_or_else(|| eyre!("dump entry {dump_entry_name} has no file name"))?;
    let extracted_dump_path = temp_dir.path().join(dump_file_name);
    let metadata = read_archive_metadata(&mut zip_archive)?;

    {
        let mut dump_entry = zip_archive.by_name(&dump_entry_name)?;
        let mut extracted_dump = File::create(&extracted_dump_path)?;
        io::copy(&mut dump_entry, &mut extracted_dump)?;
    }

    Ok(PreparedDumpArtifact {
        dump_path: extracted_dump_path,
        metadata,
        _temp_dir: Some(temp_dir),
    })
}

fn find_dump_entry_name(zip_archive: &mut ZipArchive<File>) -> Result<String> {
    for index in 0..zip_archive.len() {
        let entry_name = {
            let file = zip_archive.by_index(index)?;
            file.name().to_owned()
        };

        if entry_name.ends_with(".dmp") {
            return Ok(entry_name);
        }
    }

    Err(eyre!("zip archive does not contain a .dmp file"))
}

fn read_archive_metadata(zip_archive: &mut ZipArchive<File>) -> Result<Option<DumpMetadata>> {
    let mut metadata_file = match zip_archive.by_name("metadata.json") {
        Ok(metadata_file) => metadata_file,
        Err(ZipError::FileNotFound) => return Ok(None),
        Err(error) => return Err(error.into()),
    };

    let mut metadata_contents = String::new();
    metadata_file.read_to_string(&mut metadata_contents)?;

    parse_json_metadata(&metadata_contents)
}

fn dump_metadata_path(dump_path: &Path) -> PathBuf {
    let file_name = dump_path
        .file_name()
        .map(|name| format!("{}.meta", name.to_string_lossy()))
        .unwrap_or_else(|| "actiona-run.dmp.meta".to_owned());

    dump_path.with_file_name(file_name)
}

fn read_legacy_dump_metadata(dump_path: &Path) -> Result<Option<DumpMetadata>> {
    let metadata_path = dump_metadata_path(dump_path);

    if !metadata_path.exists() {
        return Ok(None);
    }

    let metadata = fs::read_to_string(&metadata_path)?;
    let mut version = None;
    let mut git_sha = None;

    for line in metadata.lines() {
        if let Some(value) = line.strip_prefix("version=") {
            version = Some(value.to_owned());
        } else if let Some(value) = line.strip_prefix("git_sha=") {
            git_sha = Some(value.to_owned());
        }
    }

    Ok(Some(DumpMetadata { version, git_sha }))
}

fn parse_json_metadata(metadata_contents: &str) -> Result<Option<DumpMetadata>> {
    let metadata_value: serde_json::Value = serde_json::from_str(metadata_contents)?;
    let version = metadata_value
        .get("version")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned);
    let git_sha = metadata_value
        .get("git_sha")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned);

    if version.is_none() && git_sha.is_none() {
        Ok(None)
    } else {
        Ok(Some(DumpMetadata { version, git_sha }))
    }
}

fn print_dump_metadata(metadata: Option<&DumpMetadata>) {
    let Some(metadata) = metadata else {
        return;
    };

    if let Some(version) = metadata.version.as_deref() {
        if let Some(git_sha) = metadata.git_sha.as_deref() {
            println!("Version: {version} ({git_sha})");
        } else {
            println!("Version: {version}");
        }
    } else if let Some(git_sha) = metadata.git_sha.as_deref() {
        println!("Git: {git_sha}");
    }
}

fn load_symbol_functions(sym_path: &Path) -> Result<Vec<SymbolFunction>> {
    let symbol_file = fs::read_to_string(sym_path)?;
    let mut symbol_functions = Vec::new();

    for line in symbol_file.lines() {
        if !line.starts_with("FUNC ") {
            continue;
        }

        let mut parts = line.splitn(5, ' ');
        let _record_type = parts.next();
        let start_offset = parts.next().and_then(parse_hex_u64);
        let size = parts.next().and_then(parse_hex_u64);
        let _parameter_size = parts.next();
        let name = parts.next();

        let (Some(start_offset), Some(size), Some(name)) = (start_offset, size, name) else {
            continue;
        };

        symbol_functions.push(SymbolFunction {
            start_offset,
            end_offset: start_offset.saturating_add(size),
            name: name.to_owned(),
        });
    }

    Ok(symbol_functions)
}

fn parse_hex_u64(value: &str) -> Option<u64> {
    u64::from_str_radix(value, 16).ok()
}

fn symbolize_stackwalk_output(
    stackwalk_output: &str,
    symbol_functions: &[SymbolFunction],
) -> String {
    let mut symbolicated_lines = Vec::new();

    for line in stackwalk_output.lines() {
        symbolicated_lines.push(symbolize_stackwalk_line(line, symbol_functions));
    }

    symbolicated_lines.join("\n")
}

fn symbolize_stackwalk_line(line: &str, symbol_functions: &[SymbolFunction]) -> String {
    let frame_marker = format!("{STACKWALK_MODULE_NAME} + 0x");
    let Some(marker_index) = line.find(&frame_marker) else {
        return line.to_owned();
    };

    let offset_start = marker_index + frame_marker.len();
    let offset_text = line[offset_start..].trim();
    let Some(frame_offset) = parse_hex_u64(offset_text) else {
        return line.to_owned();
    };

    let Some(symbol_function) = symbol_functions.iter().find(|symbol_function| {
        frame_offset >= symbol_function.start_offset && frame_offset < symbol_function.end_offset
    }) else {
        return line.to_owned();
    };

    let function_offset = frame_offset.saturating_sub(symbol_function.start_offset);
    let prefix = &line[..marker_index];

    format!(
        "{prefix}{STACKWALK_MODULE_NAME}!{} + 0x{function_offset:x}",
        symbol_function.name,
    )
}

#[cfg(unix)]
fn binary_path(release_dir: &Path, binary_name: &str) -> PathBuf {
    release_dir.join(binary_name)
}

#[cfg(windows)]
fn binary_path(release_dir: &Path, binary_name: &str) -> PathBuf {
    release_dir.join(format!("{binary_name}.exe"))
}

fn run_dump_syms(binary_path: &Path, sym_path: &Path) -> Result<()> {
    let output = Command::new("dump_syms")
        .arg(binary_path)
        .output()
        .map_err(|error| eyre!("Failed to run dump_syms: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(eyre!(
            "dump_syms failed for {}: {stderr}",
            binary_path.display()
        ));
    }

    fs::write(sym_path, &output.stdout)?;

    Ok(())
}

#[cfg(unix)]
fn run_strip(binary_path: &Path) -> Result<()> {
    let status = Command::new("strip")
        .arg(binary_path)
        .status()
        .map_err(|error| eyre!("Failed to run strip: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(eyre!("strip failed for {}", binary_path.display()))
    }
}
