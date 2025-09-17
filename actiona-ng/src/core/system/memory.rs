use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use humansize::{BINARY, format_size};
use sysinfo::MemoryRefreshKind;

#[derive(Debug)]
pub struct MemoryUsage {
    used: u64,
    free: u64,
    available: u64,
    total: u64,
}

impl Display for MemoryUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(used: {}, free: {}, available: {}, total: {})",
            format_size(self.used, BINARY),
            format_size(self.free, BINARY),
            format_size(self.available, BINARY),
            format_size(self.total, BINARY)
        )
    }
}

impl MemoryUsage {
    pub fn used(&self) -> u64 {
        self.used
    }

    pub fn free(&self) -> u64 {
        self.free
    }

    pub fn available(&self) -> u64 {
        self.available
    }

    pub fn total(&self) -> u64 {
        self.total
    }
}

#[derive(Debug)]
pub struct CGroupLimits {
    total_memory: u64,
    free_memory: u64,
    free_swap: u64,
    rss: u64,
}

impl From<sysinfo::CGroupLimits> for CGroupLimits {
    fn from(value: sysinfo::CGroupLimits) -> Self {
        Self {
            total_memory: value.total_memory,
            free_memory: value.free_memory,
            free_swap: value.free_swap,
            rss: value.rss,
        }
    }
}

impl Display for CGroupLimits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(total memory: {}, free memory: {}, free swap: {}, rss: {})",
            format_size(self.total_memory, BINARY),
            format_size(self.free_memory, BINARY),
            format_size(self.free_swap, BINARY),
            format_size(self.rss, BINARY)
        )
    }
}

impl CGroupLimits {
    pub fn total_memory(&self) -> u64 {
        self.total_memory
    }

    pub fn free_memory(&self) -> u64 {
        self.free_memory
    }

    pub fn free_swap(&self) -> u64 {
        self.free_swap
    }

    pub fn rss(&self) -> u64 {
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
            used: system_guard.used_memory(),
            free: system_guard.free_memory(),
            available: system_guard.available_memory(),
            total: system_guard.total_memory(),
        }
    }

    /// Note that "available" is the same as "free" for swap.
    pub fn swap_usage(&self) -> MemoryUsage {
        let mut system_guard = self.system.lock().unwrap();
        system_guard.refresh_memory_specifics(MemoryRefreshKind::nothing().with_swap());

        MemoryUsage {
            used: system_guard.used_swap(),
            free: system_guard.free_swap(),
            available: system_guard.free_swap(), // We use "free" here
            total: system_guard.total_swap(),
        }
    }

    /// Note: only works on Linux
    pub fn cgroup_limits(&self) -> Option<CGroupLimits> {
        let system_guard = self.system.lock().unwrap();
        system_guard.cgroup_limits().map(|limits| limits.into())
    }
}
