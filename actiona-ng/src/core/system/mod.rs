use std::sync::{Arc, Mutex};

use crate::core::system::{
    cpu::Cpu, hardware::Hardware, memory::Memory, motherboard::Motherboard, network::Network,
    os::Os,
};

pub mod cpu;
pub mod hardware;
pub mod memory;
pub mod motherboard;
pub mod network;
pub mod os;

#[derive(Debug)]
pub struct System {
    cpu: Arc<Cpu>,
    memory: Arc<Memory>,
    motherboard: Arc<Motherboard>,
    os: Arc<Os>,
    network: Arc<Network>,
    hardware: Arc<Hardware>,
}

impl System {
    pub fn new(system: Arc<Mutex<sysinfo::System>>) -> Self {
        {
            let mut system_guard = system.lock().unwrap();
            system_guard.refresh_all();
        }

        Self {
            cpu: Arc::new(Cpu::new(system.clone())),
            memory: Arc::new(Memory::new(system.clone())),
            motherboard: Arc::new(Motherboard::default()),
            os: Arc::new(Os::default()),
            network: Arc::new(Network::default()),
            hardware: Arc::new(Hardware::default()),
        }
    }

    pub fn cpu(&self) -> Arc<Cpu> {
        self.cpu.clone()
    }

    pub fn memory(&self) -> Arc<Memory> {
        self.memory.clone()
    }

    pub fn motherboard(&self) -> Arc<Motherboard> {
        self.motherboard.clone()
    }

    pub fn os(&self) -> Arc<Os> {
        self.os.clone()
    }

    pub fn network(&self) -> Arc<Network> {
        self.network.clone()
    }

    pub fn hardware(&self) -> Arc<Hardware> {
        self.hardware.clone()
    }

    // TODO: processes, disk
}

pub(crate) fn normalize_string<S: AsRef<str>>(s: Option<S>) -> Option<String> {
    match s.as_ref().map(|s| s.as_ref().trim()) {
        Some("Default string")
        | Some("To be filled by O.E.M.")
        | Some("System Product Name")
        | Some("Not Specified") => None,
        Some(s) if s.is_empty() => None,
        Some(s) => Some(s.to_string()),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::Runtime;

    #[test]
    fn test_cpu_usage() {
        Runtime::test(async move |runtime| {
            let system = System::new(runtime.system());

            assert!(system.cpu().usage().await > 0.)
        });
    }

    #[test]
    fn test_name() {
        Runtime::test(async move |runtime| {
            let system = System::new(runtime.system());

            println!("{:#?}", system);

            println!("memory: {}", system.memory.usage());
            println!("swap: {}", system.memory.swap_usage());
            println!("uptime: {}", humantime::format_duration(system.os.uptime()));
            println!(
                "boot_time: {}",
                humantime::format_rfc3339(system.os.boot_time())
            );
            println!("cgroup limits: {:?}", system.memory().cgroup_limits());
            println!("hostname: {:?}", system.network().hostname());
            for (name, interface) in system.network().interfaces() {
                println!("{name}: {interface}");
            }
            println!("hardware: {:?}", system.hardware());
            println!("motherboard: {:?}", system.motherboard());
            println!("users: {:#?}", system.os().users());
            println!("groups: {:#?}", system.os().groups());
            println!("components: {:#?}", system.hardware().components());
        });
    }
}
