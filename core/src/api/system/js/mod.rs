use std::path::Path;

use human_units::{FormatSize, si::Prefix};
use macros::{js_class, js_methods};
use rquickjs::{
    Ctx, Exception, JsLifetime, Result,
    atom::PredefinedAtom,
    class::Trace,
    prelude::{Func, Opt},
};
use tokio_util::task::TaskTracker;
use tracing::instrument;

use crate::{
    IntoJsResult,
    api::{
        js::classes::{SingletonClass, register_host_class, registration_target},
        system::{
            System,
            js::{
                cpu::JsCpu, hardware::JsHardware, memory::JsMemory, network::JsNetwork, os::JsOs,
                processes::JsProcesses, storage::JsStorage,
            },
        },
    },
    types::display::display_with_type,
};
pub mod cpu;
pub mod hardware;
pub mod memory;
pub mod network;
pub mod os;
pub mod processes;
pub mod storage;

/// System information and power/session operations.
///
/// ```ts
/// const cpuUsage = await system.cpu.usage();
/// const memory = await system.memory.usage();
///
/// println(formatPercent(cpuUsage), formatBytes(memory.used));
/// ```
///
/// ```ts
/// const interfaces = await system.network.listInterfaces();
/// println(`interfaces: ${interfaces.length}`);
/// ```
/// @singleton
#[derive(Debug, JsLifetime)]
#[js_class]
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
        register_host_class::<JsProcesses>(ctx)?;
        register_host_class::<JsStorage>(ctx)?;

        let target = registration_target(ctx);
        target.set("formatFrequency", Func::from(format_frequency))?;
        target.set("formatPercent", Func::from(format_percent))?;
        target.set("formatBytes", Func::from(format_bytes))?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsSystem {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsSystem {
    /// @skip
    #[instrument(skip_all)]
    pub async fn new(task_tracker: TaskTracker) -> super::Result<Self> {
        Ok(Self {
            inner: System::new(task_tracker).await?,
        })
    }
}

#[js_methods]
impl JsSystem {
    /// Cpu information
    #[get]
    #[must_use]
    pub fn cpu(&self) -> JsCpu {
        JsCpu::new(self.inner.cpu())
    }

    /// Hardware information
    #[get]
    #[must_use]
    pub fn hardware(&self) -> JsHardware {
        JsHardware::new(self.inner.hardware())
    }

    /// Memory information
    #[get]
    #[must_use]
    pub fn memory(&self) -> JsMemory {
        JsMemory::new(self.inner.memory())
    }

    /// Network information
    #[get]
    #[must_use]
    pub fn network(&self) -> JsNetwork {
        JsNetwork::new(self.inner.network())
    }

    /// Os information
    #[get]
    #[must_use]
    pub fn os(&self) -> JsOs {
        JsOs::new(self.inner.os())
    }

    /// Processes information
    #[get]
    #[must_use]
    pub fn processes(&self) -> JsProcesses {
        JsProcesses::new(self.inner.processes())
    }

    /// Storage information
    #[get]
    #[must_use]
    pub fn storage(&self) -> JsStorage {
        JsStorage::new(self.inner.storage())
    }

    pub fn shutdown(&self, ctx: Ctx<'_>, force: Opt<bool>) -> Result<()> {
        let force = force.unwrap_or_default();

        System::shutdown(force).into_js_result(&ctx)?;

        Ok(())
    }

    pub fn reboot(&self, ctx: Ctx<'_>, force: Opt<bool>) -> Result<()> {
        let force = force.unwrap_or_default();

        System::reboot(force).into_js_result(&ctx)?;

        Ok(())
    }

    pub fn logout(&self, ctx: Ctx<'_>, force: Opt<bool>) -> Result<()> {
        let force = force.unwrap_or_default();

        System::logout(force).into_js_result(&ctx)?;

        Ok(())
    }

    pub fn hibernate(&self, ctx: Ctx<'_>) -> Result<()> {
        System::hibernate().into_js_result(&ctx)?;

        Ok(())
    }

    pub fn sleep(&self, ctx: Ctx<'_>) -> Result<()> {
        System::sleep().into_js_result(&ctx)?;

        Ok(())
    }

    pub fn open(&self, ctx: Ctx<'_>, path: String, with_app: Opt<String>) -> Result<()> {
        System::open(path.as_ref(), with_app.as_deref()).into_js_result(&ctx)?;

        Ok(())
    }

    pub fn open_path(&self, ctx: Ctx<'_>, path: String, with_app: Option<String>) -> Result<()> {
        System::open_path(Path::new(&path), with_app.as_deref()).into_js_result(&ctx)?;

        Ok(())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("System", &self.inner)
    }
}

/// Formats a frequency value in Hz using SI prefixes.
///
/// ```ts
/// formatFrequency(40000);    // "40 kHz"
/// formatFrequency(3400000);  // "3.4 MHz"
/// ```
pub fn format_frequency(ctx: Ctx<'_>, frequency: u64) -> Result<String> {
    Ok(
        human_units::si::Frequency::try_with_si_prefix(frequency, Prefix::None)
            .map_err(|_| Exception::throw_range(&ctx, "out of range"))?
            .to_string(),
    )
}

/// Formats a percentage value and appends `%`.
///
/// ```ts
/// formatPercent(50);          // "50%"
/// formatPercent(50.005);      // "50.01%"
/// formatPercent(12.3456, 1);  // "12.3%"
/// ```
pub fn format_percent(ctx: Ctx<'_>, percent: f64, precision: Opt<u32>) -> Result<String> {
    let precision = precision
        .0
        .unwrap_or(2)
        .try_into()
        .map_err(|_| Exception::throw_range(&ctx, "out of range"))?;

    let mut s = format!("{percent:.precision$}");
    if s.contains('.') {
        s = s.trim_end_matches('0').trim_end_matches('.').to_string();
    }

    Ok(format!("{s}%"))
}

/// Formats a byte size using human-readable units.
///
/// ```ts
/// formatBytes(42000);        // "42 kB"
/// formatBytes(1048576);      // "1.05 MB"
/// ```
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    bytes.format_size().to_string()
}

#[cfg(test)]
mod tests {
    use crate::runtime::Runtime;

    #[test]
    fn test_format_frequency() {
        Runtime::test_with_script_engine(async |script_engine| {
            assert_eq!(
                script_engine
                    .eval::<String>("formatFrequency(40000)")
                    .await
                    .unwrap(),
                "40 kHz"
            );
        });
    }

    #[test]
    fn test_format_percent() {
        Runtime::test_with_script_engine(async |script_engine| {
            assert_eq!(
                script_engine
                    .eval::<String>("formatPercent(50)")
                    .await
                    .unwrap(),
                "50%"
            );

            assert_eq!(
                script_engine
                    .eval::<String>("formatPercent(50.005)")
                    .await
                    .unwrap(),
                "50.01%"
            );
        });
    }

    #[test]
    fn test_format_bytes() {
        Runtime::test_with_script_engine(async |script_engine| {
            assert_eq!(
                script_engine
                    .eval::<String>("formatBytes(42000)")
                    .await
                    .unwrap(),
                "41 KiB"
            );
        });
    }
}
