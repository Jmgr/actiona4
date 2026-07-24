use std::{fs::File, io::Write, path::Path, time::SystemTime};

use chrono::{DateTime as ChronoDateTime, Datelike, Timelike, Utc};
use color_eyre::{Result, eyre::eyre};
use installer_tools::package::{PackagedFile, PackagedFilePlatform, packaged_files};
use tokio::fs::metadata;
use zip::{CompressionMethod, DateTime, ZipWriter, write::SimpleFileOptions};

use crate::{
    package_docs::read_packaged_file_contents, util::remove_file_if_exists,
    workspace::WorkspacePackageInfo,
};

struct ArchiveFile {
    packaged_file: PackagedFile,
}

struct ArchiveEntry {
    destination_name: String,
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

fn archive_files(workspace_root: &Path) -> Result<Vec<ArchiveFile>> {
    Ok(packaged_files(workspace_root)?
        .iter()
        .filter(|packaged_file| packaged_file.include_in_archive)
        .map(|packaged_file| ArchiveFile {
            packaged_file: packaged_file.clone(),
        })
        .collect())
}

async fn read_archive_entries(workspace_root: &Path) -> Result<Vec<ArchiveEntry>> {
    let mut archive_entries = Vec::new();

    for archive_file in archive_files(workspace_root)? {
        let source_path = workspace_root.join(&archive_file.packaged_file.source_path);

        if !source_path.is_file() {
            return Err(eyre!("File not found: {}", source_path.display()));
        }

        let metadata = metadata(&source_path).await?;
        let contents =
            read_packaged_file_contents(workspace_root, &archive_file.packaged_file).await?;
        archive_entries.push(ArchiveEntry {
            destination_name: archive_file
                .packaged_file
                .destination_name_for(PackagedFilePlatform::Windows)
                .to_owned(),
            contents,
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
        zip_writer.start_file(&archive_entry.destination_name, file_options)?;
        zip_writer.write_all(&archive_entry.contents)?;
    }

    zip_writer.finish()?;

    Ok(())
}

fn zip_datetime_from_system_time(system_time: SystemTime) -> DateTime {
    let datetime: ChronoDateTime<Utc> = system_time.into();
    let clamped_year = u16::try_from(datetime.year().clamp(1980, 2107))
        .expect("clamped ZIP timestamp year should fit in u16");
    let (month, day, hour, minute, second) = if i32::from(clamped_year) == datetime.year() {
        (
            u8::try_from(datetime.month()).expect("month should fit in u8"),
            u8::try_from(datetime.day()).expect("day should fit in u8"),
            u8::try_from(datetime.hour()).expect("hour should fit in u8"),
            u8::try_from(datetime.minute()).expect("minute should fit in u8"),
            u8::try_from(datetime.second()).expect("second should fit in u8"),
        )
    } else if clamped_year == 1980 {
        (1, 1, 0, 0, 0)
    } else {
        (12, 31, 23, 59, 58)
    };

    DateTime::from_date_and_time(clamped_year, month, day, hour, minute, second)
        .expect("clamped ZIP timestamp should always be valid")
}
