use std::{fmt::Display, sync::Arc};

use eyre::Result;
use tokio::join;
use tokio_util::task::TaskTracker;
use tracing::instrument;

use crate::{
    core::system::{
        cpu::Cpu, hardware::Hardware, memory::Memory, network::Network, os::Os,
        processes::Processes, storage::Storage,
    },
    types::display::DisplayFields,
};

pub mod cpu;
pub mod hardware;
pub mod memory;
pub mod network;
pub mod os;
pub mod platform;
pub mod processes;
pub mod storage;

#[derive(Debug)]
pub struct System {
    cpu: Arc<Cpu>,
    memory: Arc<Memory>,
    os: Arc<Os>,
    network: Arc<Network>,
    hardware: Arc<Hardware>,
    storage: Arc<Storage>,
    processes: Arc<Processes>,
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
            cpu: Arc::new(cpu?),
            memory: Arc::new(memory?),
            os: Arc::new(os?),
            network: Arc::new(network?),
            hardware: Arc::new(hardware?),
            storage: Arc::new(storage?),
            processes: Arc::new(processes?),
        })
    }

    #[must_use]
    pub fn cpu(&self) -> Arc<Cpu> {
        self.cpu.clone()
    }

    #[must_use]
    pub fn memory(&self) -> Arc<Memory> {
        self.memory.clone()
    }

    #[must_use]
    pub fn os(&self) -> Arc<Os> {
        self.os.clone()
    }

    #[must_use]
    pub fn network(&self) -> Arc<Network> {
        self.network.clone()
    }

    #[must_use]
    pub fn hardware(&self) -> Arc<Hardware> {
        self.hardware.clone()
    }

    #[must_use]
    pub fn storage(&self) -> Arc<Storage> {
        self.storage.clone()
    }

    #[must_use]
    pub fn processes(&self) -> Arc<Processes> {
        self.processes.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::{process::Command, time::sleep};
    use tracing_subscriber::{EnvFilter, fmt, fmt::format::FmtSpan, prelude::*};

    use super::*;
    use crate::{core::system::processes::Signal, runtime::Runtime};

    #[test]
    fn test_cpu_usage() {
        Runtime::test(async move |runtime| {
            let system = System::new(runtime.task_tracker()).await.unwrap();

            assert!(*system.cpu().refresh_global_usage().await.unwrap() > 0.)
        });
    }

    #[test]
    fn test_signal() {
        Runtime::test(async move |runtime| {
            let system = System::new(runtime.task_tracker()).await.unwrap();

            let child = Command::new("gnome-calculator")
                .spawn()
                .expect("failed to spawn xeyes");

            let pid = child.id().unwrap().try_into().unwrap();

            println!("pid: {pid}");

            sleep(Duration::from_secs(3)).await;

            let result = system
                .processes()
                .send_signal_and_wait(pid, Signal::Term, runtime.cancellation_token())
                .await
                .unwrap();
            println!("result: {result:?}");

            sleep(Duration::from_secs(3)).await;

            /*
            let process = system.processes().from_pid(pid).await.unwrap().unwrap();
            system
                .processes()
                .send_signal_and_wait(&process)
                .await
                .unwrap();
            */
        });
    }

    #[test]
    fn init_test() {
        Runtime::test(async move |runtime| {
            let console_layer = console_subscriber::spawn(); // serves to tokio-console
            let fmt_layer = fmt::layer()
                .with_test_writer()
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE); // TODO

            let filter =
                EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"));

            tracing_subscriber::registry()
                .with(filter)
                .with(console_layer)
                .with(fmt_layer)
                .init();

            let _system = System::new(runtime.task_tracker()).await.unwrap();
        });
    }
}
