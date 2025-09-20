use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use sysinfo::MemoryRefreshKind;

use crate::types::ByteCount;

#[derive(Debug)]
pub struct MemoryUsage {
    used: ByteCount,
    free: ByteCount,
    available: ByteCount,
    total: ByteCount,
}

impl Display for MemoryUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(used: {}, free: {}, available: {}, total: {})",
            self.used, self.free, self.available, self.total,
        )
    }
}

impl MemoryUsage {
    pub fn used(&self) -> ByteCount {
        self.used
    }

    pub fn free(&self) -> ByteCount {
        self.free
    }

    pub fn available(&self) -> ByteCount {
        self.available
    }

    pub fn total(&self) -> ByteCount {
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
        write!(
            f,
            "(total memory: {}, free memory: {}, free swap: {}, rss: {})",
            self.total_memory, self.free_memory, self.free_swap, self.rss,
        )
    }
}

impl CGroupLimits {
    pub fn total_memory(&self) -> ByteCount {
        self.total_memory
    }

    pub fn free_memory(&self) -> ByteCount {
        self.free_memory
    }

    pub fn free_swap(&self) -> ByteCount {
        self.free_swap
    }

    pub fn rss(&self) -> ByteCount {
        self.rss
    }
}

#[derive_where::derive_where(Debug)]
pub struct Memory {
    #[derive_where(skip)]
    system: Arc<Mutex<sysinfo::System>>,
}

impl Memory {
    pub fn new(system: Arc<Mutex<sysinfo::System>>) -> Self {
        Self { system }
    }

    pub fn usage(&self) -> MemoryUsage {
        let mut system_guard = self.system.lock().unwrap();
        system_guard.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());

        MemoryUsage {
            used: system_guard.used_memory().into(),
            free: system_guard.free_memory().into(),
            available: system_guard.available_memory().into(),
            total: system_guard.total_memory().into(),
        }
    }

    /// Note that "available" is the same as "free" for swap.
    pub fn swap_usage(&self) -> MemoryUsage {
        let mut system_guard = self.system.lock().unwrap();
        system_guard.refresh_memory_specifics(MemoryRefreshKind::nothing().with_swap());

        MemoryUsage {
            used: system_guard.used_swap().into(),
            free: system_guard.free_swap().into(),
            available: system_guard.free_swap().into(), // We use "free" here
            total: system_guard.total_swap().into(),
        }
    }

    /// Note: only works on Linux
    pub fn cgroup_limits(&self) -> Option<CGroupLimits> {
        let system_guard = self.system.lock().unwrap();
        system_guard.cgroup_limits().map(|limits| limits.into())
    }
}
