use std::{fmt::Display, sync::Arc};

use color_eyre::Result;
use derive_where::derive_where;
use parking_lot::Mutex;
use sysinfo::{MemoryRefreshKind, RefreshKind};
use tokio_util::task::TaskTracker;
use tracing::instrument;

use crate::types::{ByteCount, display::DisplayFields};

#[derive(Debug)]
pub struct MemoryUsage {
    used: ByteCount,
    free: ByteCount,
    available: ByteCount,
    total: ByteCount,
}

impl Display for MemoryUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("used", self.used)
            .display("free", self.free)
            .display("available", self.available)
            .display("total", self.total)
            .finish(f)
    }
}

impl MemoryUsage {
    fn new_with_memory(system: &sysinfo::System) -> Self {
        Self {
            used: system.used_memory().into(),
            free: system.free_memory().into(),
            available: system.available_memory().into(),
            total: system.total_memory().into(),
        }
    }

    /// Note that "available" is the same as "free" for swap.
    fn new_with_swap(system: &sysinfo::System) -> Self {
        Self {
            used: system.used_swap().into(),
            free: system.free_swap().into(),
            available: system.free_swap().into(), // We use "free" here
            total: system.total_swap().into(),
        }
    }

    #[must_use]
    pub const fn used(&self) -> ByteCount {
        self.used
    }

    #[must_use]
    pub const fn free(&self) -> ByteCount {
        self.free
    }

    #[must_use]
    pub const fn available(&self) -> ByteCount {
        self.available
    }

    #[must_use]
    pub const fn total(&self) -> ByteCount {
        self.total
    }
}

#[derive(Debug)]
pub struct CGroupLimits {
    total_memory: ByteCount,
    free_memory: ByteCount,
    free_swap: ByteCount,
    rss: ByteCount,
}

impl From<sysinfo::CGroupLimits> for CGroupLimits {
    fn from(value: sysinfo::CGroupLimits) -> Self {
        Self {
            total_memory: value.total_memory.into(),
            free_memory: value.free_memory.into(),
            free_swap: value.free_swap.into(),
            rss: value.rss.into(),
        }
    }
}

impl Display for CGroupLimits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("free_memory", self.free_memory)
            .display("free_swap", self.free_swap)
            .display("rss", self.rss)
            .display("total", self.total_memory)
            .finish(f)
    }
}

impl CGroupLimits {
    #[must_use]
    pub const fn total_memory(&self) -> ByteCount {
        self.total_memory
    }

    #[must_use]
    pub const fn free_memory(&self) -> ByteCount {
        self.free_memory
    }

    #[must_use]
    pub const fn free_swap(&self) -> ByteCount {
        self.free_swap
    }

    #[must_use]
    pub const fn rss(&self) -> ByteCount {
        self.rss
    }
}

#[derive_where(Debug)]
pub struct Memory {
    #[derive_where(skip)]
    system: Arc<Mutex<sysinfo::System>>,

    #[derive_where(skip)]
    task_tracker: TaskTracker,
}

impl Display for Memory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("memory", self.memory_usage())
            .display("swap", self.swap_usage())
            .finish(f)
    }
}

impl Memory {
    #[instrument(name = "memory", skip_all)]
    pub async fn new(task_tracker: TaskTracker) -> Result<Self> {
        let system = task_tracker
            .spawn_blocking(move || {
                sysinfo::System::new_with_specifics(
                    RefreshKind::nothing().with_memory(MemoryRefreshKind::everything()),
                )
            })
            .await?;

        Ok(Self {
            system: Arc::new(Mutex::new(system)),
            task_tracker,
        })
    }

    pub async fn refresh_memory_usage(&self) -> Result<MemoryUsage> {
        let local_system = self.system.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut system_guard = local_system.lock();
                system_guard.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());
                MemoryUsage::new_with_memory(&system_guard)
            })
            .await?;
        Ok(result)
    }

    #[must_use]
    pub fn memory_usage(&self) -> MemoryUsage {
        let system_guard = self.system.lock();

        MemoryUsage::new_with_memory(&system_guard)
    }

    pub async fn refresh_swap_usage(&self) -> Result<MemoryUsage> {
        let local_system = self.system.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut system_guard = local_system.lock();
                system_guard.refresh_memory_specifics(MemoryRefreshKind::nothing().with_swap());
                MemoryUsage::new_with_swap(&system_guard)
            })
            .await?;
        Ok(result)
    }

    #[must_use]
    pub fn swap_usage(&self) -> MemoryUsage {
        let system_guard = self.system.lock();

        MemoryUsage::new_with_swap(&system_guard)
    }

    /// Note: only works on Linux
    pub fn cgroup_limits(&self) -> Option<CGroupLimits> {
        let system_guard = self.system.lock();
        system_guard.cgroup_limits().map(CGroupLimits::from)
    }
}
