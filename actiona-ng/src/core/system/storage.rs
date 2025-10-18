use std::{
    fmt::Display,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use eyre::Result;
use itertools::Itertools;
use sysinfo::DiskKind;
use tokio_util::task::TaskTracker;
use tracing::instrument;

use crate::types::{ByteCount, DisplayFields, OptionalSystemString, display_list};

#[derive(Debug)]
pub struct IoStats {
    total: ByteCount,
    delta: ByteCount,
}

impl Display for IoStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("total", self.total)
            .display("delta", self.delta)
            .finish(f)
    }
}

#[derive(Debug)]
pub struct DiskUsage {
    written: IoStats,
    read: IoStats,
}

impl Display for DiskUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("written", &self.written)
            .display("read", &self.read)
            .finish(f)
    }
}

impl From<sysinfo::DiskUsage> for DiskUsage {
    fn from(value: sysinfo::DiskUsage) -> Self {
        Self {
            written: IoStats {
                total: value.total_written_bytes.into(),
                delta: value.written_bytes.into(),
            },
            read: IoStats {
                total: value.total_read_bytes.into(),
                delta: value.read_bytes.into(),
            },
        }
    }
}

#[derive(Debug)]
pub struct Disk {
    kind: DiskKind,
    name: OptionalSystemString,
    file_system: OptionalSystemString,
    mount_point: PathBuf,
    total_space: ByteCount,
    available_space: ByteCount,
    is_removable: bool,
    is_read_only: bool,
    usage: DiskUsage,
}

impl Display for Disk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display_if_some("name", &self.name)
            .display("kind", self.kind)
            .display_if_some("file_system", &self.file_system)
            .display("mount_point", self.mount_point.display())
            .display("available_space", self.available_space)
            .display("total_space", self.total_space)
            .display("is_removable", self.is_removable)
            .display("is_read_only", self.is_read_only)
            .display("usage", &self.usage)
            .finish(f)
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
            usage: value.usage().into(),
        }
    }
}

#[derive_where::derive_where(Debug)]
pub struct Storage {
    #[derive_where(skip)]
    sysinfo_disks: Arc<Mutex<sysinfo::Disks>>,

    #[derive_where(skip)]
    task_tracker: TaskTracker,
}

impl Display for Storage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("disks", display_list(&self.disks()))
            .finish(f)
    }
}

impl Storage {
    #[instrument(name = "storage", skip_all)]
    pub async fn new(task_tracker: TaskTracker) -> Result<Self> {
        let sysinfo_disks = task_tracker
            .spawn_blocking(sysinfo::Disks::new_with_refreshed_list)
            .await?;

        Ok(Self {
            sysinfo_disks: Arc::new(Mutex::new(sysinfo_disks)),
            task_tracker,
        })
    }

    pub async fn refresh_disks(&self, rescan: bool) -> Result<Vec<Disk>> {
        let sysinfo_disks = self.sysinfo_disks.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut sysinfo_disks = sysinfo_disks.lock().unwrap();
                sysinfo_disks.refresh(rescan);
                sysinfo_disks.list().iter().map(Disk::from).collect_vec()
            })
            .await?;
        Ok(result)
    }

    pub async fn refresh_disk(&self, disk: &Disk) -> Result<Option<Disk>> {
        let disk_id = disk.mount_point.clone();
        let sysinfo_disks = self.sysinfo_disks.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut sysinfo_disks = sysinfo_disks.lock().unwrap();
                let disk = sysinfo_disks
                    .list_mut()
                    .iter_mut()
                    .find(|disk| disk.mount_point() == disk_id)?;
                disk.refresh();
                Some(Disk::from(&*disk))
            })
            .await?;
        Ok(result)
    }

    pub fn disks(&self) -> Vec<Disk> {
        let sysinfo_disks = self.sysinfo_disks.lock().unwrap();
        sysinfo_disks.list().iter().map(Disk::from).collect_vec()
    }
}
