use std::{ffi::OsStr, fmt::Display, path::Path};

use color_eyre::Result;
use system_shutdown::{
    force_logout, force_reboot, force_shutdown, hibernate, logout, reboot, shutdown, sleep,
};
use tokio::join;
use tokio_util::task::TaskTracker;
use tracing::instrument;

use crate::{
    api::system::{
        cpu::Cpu, hardware::Hardware, memory::Memory, network::Network, os::Os,
        processes::Processes, storage::Storage,
    },
    types::display::DisplayFields,
};

pub mod cpu;
pub mod hardware;
pub mod js;
pub mod memory;
pub mod network;
pub mod os;
pub mod platform;
pub mod processes;
pub mod storage;

#[derive(Clone, Debug)]
pub struct System {
    cpu: Cpu,
    memory: Memory,
    os: Os,
    network: Network,
    hardware: Hardware,
    storage: Storage,
    processes: Processes,
}

impl Display for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("cpu", self.cpu())
            .display("memory", self.memory())
            .display("os", self.os())
            .display("network", self.network())
            .display("hardware", self.hardware())
            .display("storage", self.storage())
            .display("processes", self.processes())
            .finish(f)
    }
}

impl System {
    #[instrument(name = "system", skip_all)]
    pub async fn new(task_tracker: TaskTracker) -> Result<Self> {
        let (cpu, os, network, storage, memory, hardware, processes) = join!(
            Cpu::new(task_tracker.clone()),
            Os::new(task_tracker.clone()),
            Network::new(task_tracker.clone()),
            Storage::new(task_tracker.clone()),
            Memory::new(task_tracker.clone()),
            Hardware::new(task_tracker.clone()),
            Processes::new(task_tracker.clone()),
        );

        Ok(Self {
            cpu: cpu?,
            memory: memory?,
            os: os?,
            network: network?,
            hardware: hardware?,
            storage: storage?,
            processes: processes?,
        })
    }

    #[must_use]
    pub fn cpu(&self) -> Cpu {
        self.cpu.clone()
    }

    #[must_use]
    pub fn memory(&self) -> Memory {
        self.memory.clone()
    }

    #[must_use]
    pub fn os(&self) -> Os {
        self.os.clone()
    }

    #[must_use]
    pub fn network(&self) -> Network {
        self.network.clone()
    }

    #[must_use]
    pub fn hardware(&self) -> Hardware {
        self.hardware.clone()
    }

    #[must_use]
    pub fn storage(&self) -> Storage {
        self.storage.clone()
    }

    #[must_use]
    pub fn processes(&self) -> Processes {
        self.processes.clone()
    }

    pub fn shutdown(force: bool) -> Result<()> {
        if force {
            force_shutdown()?;
        } else {
            shutdown()?;
        }

        Ok(())
    }

    pub fn reboot(force: bool) -> Result<()> {
        if force {
            force_reboot()?;
        } else {
            reboot()?;
        }

        Ok(())
    }

    pub fn logout(force: bool) -> Result<()> {
        if force {
            force_logout()?;
        } else {
            logout()?;
        }

        Ok(())
    }

    pub fn hibernate() -> Result<()> {
        hibernate()?;

        Ok(())
    }

    pub fn sleep() -> Result<()> {
        sleep()?;

        Ok(())
    }

    pub fn open(path: &OsStr, with: Option<&str>) -> Result<()> {
        if let Some(with) = with {
            open::with_detached(path, with)?;
        } else {
            open::that_detached(path)?;
        }

        Ok(())
    }

    pub fn open_path(path: &Path, with: Option<&str>) -> Result<()> {
        if with.is_none() {
            _ = path.metadata()?;
        }

        Self::open(path.as_os_str(), with)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Runtime;

    #[test]
    #[ignore]
    fn test_cpu_usage() {
        Runtime::test(async move |runtime| {
            let system = System::new(runtime.task_tracker()).await.unwrap();

            assert!(*system.cpu().refresh_global_usage().await.unwrap() > 0.)
        });
    }
}
