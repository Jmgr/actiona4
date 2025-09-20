use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use itertools::Itertools;
use tokio::time::sleep;
use tokio_util::task::TaskTracker;

pub type LastRefresh = Arc<tokio::sync::Mutex<Option<Instant>>>;

const MINIMUM_CPU_FREQUENCY_UPDATE_INTERVAL: Duration = Duration::from_millis(10);

#[derive_where::derive_where(Debug)]
pub struct Cpu {
    #[derive_where(skip)]
    system: Arc<Mutex<sysinfo::System>>,

    #[derive_where(skip)]
    last_usage_refresh: LastRefresh,

    cores: Vec<CpuCore>,
    physical_core_count: Option<usize>,
    architecture: String,
    task_tracker: TaskTracker,
}

impl Cpu {
    pub fn new(system: Arc<Mutex<sysinfo::System>>, task_tracker: TaskTracker) -> Self {
        let last_usage_refresh = Arc::new(tokio::sync::Mutex::new(None));
        let cores = {
            let system_guard = system.lock().unwrap();

            system_guard
                .cpus()
                .iter()
                .enumerate()
                .map(|(index, cpu)| {
                    CpuCore::new(
                        cpu,
                        index,
                        system.clone(),
                        task_tracker.clone(),
                        last_usage_refresh.clone(),
                    )
                })
                .collect_vec()
        };

        Self {
            system,
            last_usage_refresh,
            cores,
            physical_core_count: sysinfo::System::physical_core_count(),
            architecture: sysinfo::System::cpu_arch(),
            task_tracker,
        }
    }

    pub async fn usage(&self) -> f32 {
        cpu_usage_operation(
            self.last_usage_refresh.clone(),
            self.system.clone(),
            self.task_tracker.clone(),
            |system| system.global_cpu_usage(),
        )
        .await
    }

    pub fn cores(&self) -> &Vec<CpuCore> {
        &self.cores
    }

    pub fn physical_core_count(&self) -> Option<usize> {
        self.physical_core_count
    }

    pub fn architecture(&self) -> &str {
        &self.architecture
    }
}

#[derive_where::derive_where(Debug)]
pub struct CpuCore {
    #[derive_where(skip)]
    system: Arc<Mutex<sysinfo::System>>,

    #[derive_where(skip)]
    last_usage_refresh: LastRefresh,

    #[derive_where(skip)]
    last_frequency_refresh: LastRefresh,

    index: usize,
    name: String,
    vendor_id: String,
    brand: String,
    task_tracker: TaskTracker,
}

pub(crate) async fn cpu_usage_operation<F, R>(
    last_usage_refresh: LastRefresh,
    system: Arc<Mutex<sysinfo::System>>,
    task_tracker: TaskTracker,
    operation: F,
) -> R
where
    F: FnOnce(&sysinfo::System) -> R,
{
    let mut last_usage_refresh = last_usage_refresh.lock().await;

    if let Some(last_refresh) = *last_usage_refresh {
        if last_refresh.elapsed() < sysinfo::MINIMUM_CPU_UPDATE_INTERVAL {
            let system = system.lock().unwrap();
            return operation(&*system);
        }
    }

    let local_system = system.clone();
    task_tracker
        .spawn_blocking(move || {
            let mut system = local_system.lock().unwrap();
            system.refresh_cpu_usage();
        })
        .await
        .unwrap();

    sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL).await;

    let local_system = system.clone();
    task_tracker
        .spawn_blocking(move || {
            let mut system = local_system.lock().unwrap();
            system.refresh_cpu_usage();
        })
        .await
        .unwrap();

    {
        let system = system.lock().unwrap();
        *last_usage_refresh = Some(Instant::now());
        operation(&*system)
    }
}

impl CpuCore {
    pub fn new(
        cpu: &sysinfo::Cpu,
        index: usize,
        system: Arc<Mutex<sysinfo::System>>,
        task_tracker: TaskTracker,
        last_usage_refresh: LastRefresh,
    ) -> Self {
        Self {
            system,
            task_tracker,
            last_usage_refresh,
            last_frequency_refresh: Arc::new(tokio::sync::Mutex::new(None)),
            index,
            name: cpu.name().to_string(),
            vendor_id: cpu.vendor_id().to_string(),
            brand: cpu.brand().to_string(),
        }
    }

    pub async fn usage(&self) -> f32 {
        cpu_usage_operation(
            self.last_usage_refresh.clone(),
            self.system.clone(),
            self.task_tracker.clone(),
            |system| system.cpus()[self.index].cpu_usage(),
        )
        .await
    }

    pub async fn frequency(&self) -> u64 {
        let mut last_frequency_refresh = self.last_frequency_refresh.lock().await;

        let mut system = self.system.lock().unwrap();

        if let Some(last_refresh) = *last_frequency_refresh {
            if last_refresh.elapsed() >= MINIMUM_CPU_FREQUENCY_UPDATE_INTERVAL {
                system.refresh_cpu_frequency();

                *last_frequency_refresh = Some(Instant::now());
            }
        }

        system.cpus()[self.index].frequency()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn vendor_id(&self) -> &str {
        &self.vendor_id
    }

    pub fn brand(&self) -> &str {
        &self.brand
    }
}
