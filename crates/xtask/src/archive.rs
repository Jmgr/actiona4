use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::{DateTime as ChronoDateTime, Datelike, Timelike, Utc};
use color_eyre::{Result, eyre::eyre};
use installer_tools::package::packaged_files;
use zip::{CompressionMethod, DateTime, ZipWriter, write::SimpleFileOptions};

use crate::{util::remove_file_if_exists, workspace::WorkspacePackageInfo};

struct ArchiveFile {
    source_path: PathBuf,
    destination_name: &'static str,
}

struct ArchiveEntry {
    destination_name: &'static str,
    contents: Vec<u8>,
    last_modified_time: DateTime,
}

pub async fn build_archive(
    workspace_root: &Path,
    workspace_package_info: &WorkspacePackageInfo,
) -> Result<()> {
    let archive_path = workspace_root.join("target").join(format!(
        "actiona-run-{}-x86_64-portable.zip",
        workspace_package_info.version
    ));
    let archive_entries = read_archive_entries(workspace_root).await?;
    remove_file_if_exists(&archive_path).await?;
    write_zip_archive(&archive_path, archive_entries)?;

    Ok(())
}

fn archive_files(workspace_root: &Path) -> Vec<ArchiveFile> {
    packaged_files()
        .iter()
        .filter(|packaged_file| packaged_file.include_in_archive)
        .map(|packaged_file| ArchiveFile {
            source_path: workspace_root.join(packaged_file.source_path),
            destination_name: packaged_file.destination_name,
        })
        .collect()
}

async fn read_archive_entries(workspace_root: &Path) -> Result<Vec<ArchiveEntry>> {
    let mut archive_entries = Vec::new();

    for archive_file in archive_files(workspace_root) {
        if !archive_file.source_path.is_file() {
            return Err(eyre!(
                "File not found: {}",
                archive_file.source_path.display()
            ));
        }

        let metadata = tokio::fs::metadata(&archive_file.source_path).await?;
        archive_entries.push(ArchiveEntry {
            destination_name: archive_file.destination_name,
            contents: tokio::fs::read(&archive_file.source_path).await?,
            last_modified_time: zip_datetime_from_system_time(metadata.modified()?),
        });
    }

    Ok(archive_entries)
}

fn write_zip_archive(archive_path: &Path, archive_entries: Vec<ArchiveEntry>) -> Result<()> {
    let archive_file = File::create(archive_path)?;
    let mut zip_writer = ZipWriter::new(archive_file);
    let base_file_options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    for archive_entry in archive_entries {
        let file_options = base_file_options.last_modified_time(archive_entry.last_modified_time);
        zip_writer.start_file(archive_entry.destination_name, file_options)?;
        zip_writer.write_all(&archive_entry.contents)?;
    }

    zip_writer.finish()?;

    Ok(())
}

fn zip_datetime_from_system_time(system_time: SystemTime) -> DateTime {
    let datetime: ChronoDateTime<Utc> = system_time.into();
    let clamped_year = datetime.year().clamp(1980, 2107) as u16;
    let (month, day, hour, minute, second) = if clamped_year == datetime.year() as u16 {
        (
            datetime.month() as u8,
            datetime.day() as u8,
            datetime.hour() as u8,
            datetime.minute() as u8,
            datetime.second() as u8,
        )
    } else if clamped_year == 1980 {
        (1, 1, 0, 0, 0)
    } else {
        (12, 31, 23, 59, 58)
    };

    DateTime::from_date_and_time(clamped_year, month, day, hour, minute, second)
        .expect("clamped ZIP timestamp should always be valid")
}
