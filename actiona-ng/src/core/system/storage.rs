use std::{
    fmt::Display,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use itertools::Itertools;
use sysinfo::DiskKind;

use crate::types::{ByteCount, OptionalString};

#[derive(Debug)]
pub struct Disk {
    kind: DiskKind,
    name: OptionalString,
    file_system: OptionalString,
    mount_point: PathBuf,
    total_space: ByteCount,
    available_space: ByteCount,
    is_removable: bool,
    is_read_only: bool,

    // TODO: split usage
    total_written_bytes: ByteCount,
    written_bytes: ByteCount,
    total_read_bytes: ByteCount,
    read_bytes: ByteCount,
}

impl Display for Disk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(kind: {}, name: {}, file_system: {}, mount_point: {}, total_space: {},
            available_space: {}, is_removable: {}, is_read_only: {}, total_written_bytes: {},
            written_bytes: {}, total_read_bytes: {}, read_bytes: {})",
            self.kind,
            self.name,
            self.file_system,
            self.mount_point.display(),
            self.total_space,
            self.available_space,
            self.is_removable,
            self.is_read_only,
            self.total_written_bytes,
            self.written_bytes,
            self.total_read_bytes,
            self.read_bytes
        )
    }
}

impl From<&sysinfo::Disk> for Disk {
    fn from(value: &sysinfo::Disk) -> Self {
        Self {
            kind: value.kind(),
            name: value.name().into(),
            file_system: value.file_system().into(),
            mount_point: value.mount_point().to_path_buf(),
            total_space: value.total_space().into(),
            available_space: value.available_space().into(),
            is_removable: value.is_removable(),
            is_read_only: value.is_read_only(),
            total_written_bytes: value.usage().total_written_bytes.into(),
            written_bytes: value.usage().written_bytes.into(),
            total_read_bytes: value.usage().total_read_bytes.into(),
            read_bytes: value.usage().read_bytes.into(),
        }
    }
}

#[derive_where::derive_where(Debug)]
pub struct Storage {
    #[derive_where(skip)]
    disks: Arc<Mutex<sysinfo::Disks>>,
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            disks: Arc::new(Mutex::new(sysinfo::Disks::new())),
        }
    }
}

impl Storage {
    pub fn disks(&self) -> Vec<Disk> {
        let mut disks = self.disks.lock().unwrap();
        disks.refresh(true);

        disks.list().iter().map(|disk| disk.into()).collect_vec()
    }
}
