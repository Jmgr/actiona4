use human_units::si::Prefix;
use rquickjs::{
    Ctx, Exception, JsLifetime, Result,
    class::Trace,
    prelude::{Func, Opt},
};
use tokio_util::task::TaskTracker;

use crate::core::{
    js::classes::{SingletonClass, register_host_class},
    system::{
        System,
        js::{
            cpu::JsCpu, hardware::JsHardware, memory::JsMemory, network::JsNetwork, os::JsOs,
            storage::JsStorage,
        },
    },
};

pub mod cpu;
pub mod hardware;
pub mod memory;
pub mod network;
pub mod os;
pub mod processes;
pub mod storage;

/// System
/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "System")]
pub struct JsSystem {
    inner: System,
}

impl SingletonClass<'_> for JsSystem {
    fn register_dependencies(ctx: &Ctx<'_>) -> Result<()> {
        register_host_class::<JsCpu>(ctx)?;
        register_host_class::<JsHardware>(ctx)?;
        register_host_class::<JsMemory>(ctx)?;
        register_host_class::<JsNetwork>(ctx)?;
        register_host_class::<JsOs>(ctx)?;
        register_host_class::<JsStorage>(ctx)?;

        ctx.globals()
            .set("formatFrequency", Func::from(format_frequency))?;
        ctx.globals()
            .set("formatPercent", Func::from(format_percent))?;
        ctx.globals().set("formatBytes", Func::from(format_bytes))?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsSystem {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsSystem {
    /// @skip
    pub async fn new(task_tracker: TaskTracker) -> super::Result<Self> {
        Ok(Self {
            inner: System::new(task_tracker).await?,
        })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsSystem {
    /// Cpu information
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn cpu(&self) -> JsCpu {
        JsCpu::new(self.inner.cpu())
    }

    /// Hardware information
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn hardware(&self) -> JsHardware {
        JsHardware::new(self.inner.hardware())
    }

    /// Memory information
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn memory(&self) -> JsMemory {
        JsMemory::new(self.inner.memory())
    }

    /// Network information
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn network(&self) -> JsNetwork {
        JsNetwork::new(self.inner.network())
    }

    /// Os information
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn os(&self) -> JsOs {
        JsOs::new(self.inner.os())
    }

    /// Storage information
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn storage(&self) -> JsStorage {
        JsStorage::new(self.inner.storage())
    }
}

pub fn format_frequency(ctx: Ctx<'_>, frequency: u64) -> Result<String> {
    Ok(
        human_units::si::Frequency::try_with_si_prefix(frequency, Prefix::None)
            .map_err(|_| Exception::throw_range(&ctx, "out of range"))?
            .to_string(),
    )
}

pub fn format_percent(ctx: Ctx<'_>, percent: f64, precision: Opt<u32>) -> Result<String> {
    let precision = precision
        .0
        .unwrap_or(2)
        .try_into()
        .map_err(|_| Exception::throw_range(&ctx, "out of range"))?;

    Ok(format!("{percent:.precision$}%")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string())
}

#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    human_units::Size(bytes).to_string()
}
