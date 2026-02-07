use std::sync::Arc;

use itertools::Itertools;
use rquickjs::{Ctx, JsLifetime, Result, atom::PredefinedAtom, class::Trace};

use crate::{
    IntoJsResult,
    core::{
        js::classes::HostClass,
        system::cpu::{Cpu, CpuCore},
    },
};

/// CPU metrics and topology.
///
/// ```ts
/// const globalUsage = await system.cpu.usage();
/// const core0Usage = await system.cpu.coreUsage(0);
/// const freqs = await system.cpu.frequencies();
///
/// console.log(
///   system.cpu.logicalCoreCount,
///   formatPercent(globalUsage),
///   formatPercent(core0Usage),
///   formatFrequency(freqs[0]),
/// );
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Cpu")]
pub struct JsCpu {
    inner: Arc<Cpu>,
    cores: Vec<CpuCore>,
}

impl<'js> HostClass<'js> for JsCpu {}

impl<'js> Trace<'js> for JsCpu {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsCpu {
    /// @skip
    #[must_use]
    pub fn new(inner: Arc<Cpu>) -> Self {
        let cores = inner.cores();

        Self { inner, cores }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsCpu {
    pub async fn usage(&self, ctx: Ctx<'_>) -> Result<f32> {
        Ok(*self
            .inner
            .refresh_global_usage()
            .await
            .into_js_result(&ctx)?)
    }

    pub async fn core_usage(&self, ctx: Ctx<'_>, logical_core_index: u64) -> Result<f32> {
        let index = usize::try_from(logical_core_index).into_js_result(&ctx)?;
        Ok(*self
            .inner
            .refresh_core_usage_by_index(index)
            .await
            .into_js_result(&ctx)?)
    }

    /// @readonly
    pub async fn frequencies<'js>(&self, ctx: Ctx<'js>) -> Result<Vec<u64>> {
        let cores = self
            .inner
            .refresh_frequencies()
            .await
            .into_js_result(&ctx)?;

        Ok(cores
            .into_iter()
            .map(|core| **core.frequency())
            .collect_vec())
    }

    /// Logical core count
    /// @get
    #[qjs(get)]
    pub fn logical_core_count(&self, ctx: Ctx<'_>) -> Result<u64> {
        u64::try_from(self.cores.len()).into_js_result(&ctx)
    }

    /// Physical core count
    /// @get
    #[qjs(get)]
    pub fn physical_core_count(&self, ctx: Ctx<'_>) -> Result<Option<u64>> {
        self.inner
            .physical_core_count()
            .map(u64::try_from)
            .transpose()
            .into_js_result(&ctx)
    }

    /// Architecture
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn architecture(&self) -> &str {
        self.inner.architecture()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::Runtime;

    #[test]
    fn test_core_count() {
        Runtime::test_with_script_engine(async |script_engine| {
            assert!(
                script_engine
                    .eval::<u64>("system.cpu.physicalCoreCount")
                    .await
                    .unwrap()
                    > 0
            );

            assert!(
                script_engine
                    .eval::<u64>("system.cpu.logicalCoreCount")
                    .await
                    .unwrap()
                    > 0
            );
        });
    }
}
