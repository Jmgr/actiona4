use std::{
    collections::HashSet,
    fmt::Display,
    sync::{Arc, Mutex},
    thread::sleep,
};

use color_eyre::{Result, eyre::eyre};
use derive_where::derive_where;
use itertools::Itertools;
use sysinfo::{CpuRefreshKind, RefreshKind};
use tokio_util::task::TaskTracker;
use tracing::instrument;

use crate::types::{
    Frequency, Percent,
    display::{DisplayFields, display_list},
};

#[derive(Debug)]
pub struct CpuCore {
    index: usize,
    name: String,
    vendor: String,
    brand: String,
    usage: Percent,
    frequency: Frequency,
}

impl CpuCore {
    #[must_use]
    pub fn new(cpu: &sysinfo::Cpu, index: usize) -> Self {
        Self {
            index,
            name: cpu.name().to_string(),
            vendor: cpu.vendor_id().to_string(),
            brand: cpu.brand().to_string(),
            usage: cpu.cpu_usage().into(),
            frequency: cpu.frequency().into(),
        }
    }

    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn vendor_id(&self) -> &str {
        &self.vendor
    }

    #[must_use]
    pub fn brand(&self) -> &str {
        &self.brand
    }

    #[must_use]
    pub const fn usage(&self) -> &Percent {
        &self.usage
    }

    #[must_use]
    pub const fn frequency(&self) -> &Frequency {
        &self.frequency
    }
}

impl Display for CpuCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("name", &self.name)
            .display("vendor", &self.vendor)
            .display("brand", &self.brand)
            .display("usage", self.usage)
            .display("frequency", self.frequency)
            .finish(f)
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
struct CpuVariant {
    vendor: String,
    brand: String,
}

impl Display for CpuVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("vendor", &self.vendor)
            .display("brand", &self.brand)
            .finish(f)
    }
}

#[derive_where(Debug)]
pub struct Cpu {
    #[derive_where(skip)]
    system: Arc<Mutex<sysinfo::System>>,

    physical_core_count: Option<usize>,
    architecture: String,
    cpu_variants: Vec<CpuVariant>,

    #[derive_where(skip)]
    task_tracker: TaskTracker,
}

impl Display for Cpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            DisplayFields::default()
                .display("architecture", &self.architecture)
                .display_if_some("physical_core_count", &self.physical_core_count)
                .display("cores", display_list(&self.cores()))
                .finish(f)
        } else {
            let mut fields = DisplayFields::default()
                .display("architecture", &self.architecture)
                .display("core_count", self.cores().len())
                .display_if_some("physical_core_count", &self.physical_core_count);

            if let Some(variant) = self.cpu_variants.first()
                && self.cpu_variants.len() == 1
            {
                fields = fields.display("vendor", &variant.vendor);
                fields = fields.display("brand", &variant.brand);
            } else {
                fields = fields.display("variants", display_list(&self.cpu_variants));
            }

            fields.finish(f)
        }
    }
}

impl Cpu {
    #[instrument(name = "cpu", skip_all)]
    pub async fn new(task_tracker: TaskTracker) -> Result<Self> {
        let system = task_tracker
            .spawn_blocking(move || {
                sysinfo::System::new_with_specifics(
                    RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_frequency()),
                )
            })
            .await?;

        let cpu_variants = system
            .cpus()
            .iter()
            .map(|cpu| CpuVariant {
                vendor: cpu.vendor_id().to_string(),
                brand: cpu.brand().to_string(),
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect_vec();

        Ok(Self {
            system: Arc::new(Mutex::new(system)),
            physical_core_count: sysinfo::System::physical_core_count(),
            architecture: sysinfo::System::cpu_arch(),
            cpu_variants,
            task_tracker,
        })
    }

    pub async fn refresh_global_usage(&self) -> Result<Percent> {
        let local_system = self.system.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut system = local_system.lock().unwrap();
                system.refresh_cpu_usage();

                sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

                system.refresh_cpu_usage();
                system.global_cpu_usage().into()
            })
            .await?;
        Ok(result)
    }

    #[must_use]
    pub fn global_usage(&self) -> Percent {
        let system = self.system.lock().unwrap();
        system.global_cpu_usage().into()
    }

    pub async fn refresh_core_usage(&self, core: &CpuCore) -> Result<Percent> {
        self.refresh_core_usage_by_index(core.index).await
    }

    pub async fn refresh_core_usage_by_index(&self, index: usize) -> Result<Percent> {
        let local_system = self.system.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut system = local_system.lock().unwrap();
                if index >= system.cpus().len() {
                    return Err(eyre!("invalid index"));
                }

                system.refresh_cpu_usage();

                sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

                system.refresh_cpu_usage();
                Ok(system.cpus()[index].cpu_usage().into())
            })
            .await??;
        Ok(result)
    }

    pub async fn refresh_frequencies(&self) -> Result<Vec<CpuCore>> {
        let local_system = self.system.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut system = local_system.lock().unwrap();
                system.refresh_cpu_frequency();
                system
                    .cpus()
                    .iter()
                    .enumerate()
                    .map(|(index, cpu)| CpuCore::new(cpu, index))
                    .collect_vec()
            })
            .await?;
        Ok(result)
    }

    #[must_use]
    pub fn cores(&self) -> Vec<CpuCore> {
        let system = self.system.lock().unwrap();
        system
            .cpus()
            .iter()
            .enumerate()
            .map(|(index, cpu)| CpuCore::new(cpu, index))
            .collect_vec()
    }

    #[must_use]
    pub const fn physical_core_count(&self) -> Option<usize> {
        self.physical_core_count
    }

    #[must_use]
    pub fn architecture(&self) -> &str {
        &self.architecture
    }
}
