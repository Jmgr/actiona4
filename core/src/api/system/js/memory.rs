use macros::{js_class, js_methods, platform};
use rquickjs::{Ctx, JsLifetime, Result, atom::PredefinedAtom, class::Trace};

use crate::{
    IntoJsResult,
    api::{
        js::classes::{HostClass, register_host_class},
        system::memory::{CGroupLimits, Memory, MemoryUsage},
    },
    runtime::WithUserData,
    types::display::display_with_type,
};
/// Memory metrics.
///
/// ```ts
/// const usage = await system.memory.usage();
/// const swap = await system.memory.swapUsage();
///
/// println(formatBytes(usage.used), formatBytes(swap.used));
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsMemory {
    inner: Memory,
}

impl<'js> HostClass<'js> for JsMemory {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_host_class::<JsMemoryUsage>(ctx)?;
        register_host_class::<JsCGroupLimits>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsMemory {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsMemory {
    /// @skip
    #[must_use]
    pub const fn new(inner: Memory) -> Self {
        Self { inner }
    }
}

#[js_methods]
impl JsMemory {
    /// Memory usage
    pub async fn usage<'js>(&self, ctx: Ctx<'js>) -> Result<JsMemoryUsage> {
        self.inner
            .refresh_memory_usage()
            .await
            .map(JsMemoryUsage::from)
            .into_js_result(&ctx)
    }

    /// Swap usage
    pub async fn swap_usage<'js>(&self, ctx: Ctx<'js>) -> Result<JsMemoryUsage> {
        self.inner
            .refresh_swap_usage()
            .await
            .map(JsMemoryUsage::from)
            .into_js_result(&ctx)
    }

    /// CGroup limits
    #[platform(only = "linux")]
    #[get]
    pub fn cgroup_limits(&self, ctx: Ctx<'_>) -> Result<Option<JsCGroupLimits>> {
        ctx.user_data().require_linux(&ctx)?;
        Ok(self.inner.cgroup_limits().map(JsCGroupLimits::from))
    }

    /// Returns a string representation of this memory.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Memory", &self.inner)
    }
}

/// A memory usage snapshot.
///
/// ```ts
/// const usage = await system.memory.usage();
/// println(
///   formatBytes(usage.used),
///   formatBytes(usage.free),
///   formatBytes(usage.available),
///   formatBytes(usage.total),
/// );
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsMemoryUsage {
    inner: MemoryUsage,
}

impl<'js> HostClass<'js> for JsMemoryUsage {}

impl<'js> Trace<'js> for JsMemoryUsage {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<MemoryUsage> for JsMemoryUsage {
    fn from(value: MemoryUsage) -> Self {
        Self { inner: value }
    }
}

#[js_methods]
impl JsMemoryUsage {
    /// Used
    #[get]
    #[must_use]
    pub fn used(&self) -> u64 {
        *self.inner.used()
    }

    /// Free
    #[get]
    #[must_use]
    pub fn free(&self) -> u64 {
        *self.inner.free()
    }

    /// Available
    #[get]
    #[must_use]
    pub fn available(&self) -> u64 {
        *self.inner.available()
    }

    /// Total
    #[get]
    #[must_use]
    pub fn total(&self) -> u64 {
        *self.inner.total()
    }

    /// Returns a string representation of this memory usage.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("MemoryUsage", &self.inner)
    }
}

/// CGroup memory and swap limits.
///
/// ```ts
/// const limits = system.memory.cgroupLimits;
/// if (limits) {
///   println(
///     formatBytes(limits.totalMemory),
///     formatBytes(limits.freeMemory),
///     formatBytes(limits.freeSwap),
///   );
/// }
/// ```
///
/// CGroup limits
#[platform(only = "linux")]
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsCGroupLimits {
    inner: CGroupLimits,
}

impl<'js> HostClass<'js> for JsCGroupLimits {}

impl<'js> Trace<'js> for JsCGroupLimits {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<CGroupLimits> for JsCGroupLimits {
    fn from(value: CGroupLimits) -> Self {
        Self { inner: value }
    }
}

#[js_methods]
impl JsCGroupLimits {
    /// Total memory
    #[get]
    #[must_use]
    pub fn total_memory(&self) -> u64 {
        *self.inner.total_memory()
    }

    /// Free memory
    #[get]
    #[must_use]
    pub fn free_memory(&self) -> u64 {
        *self.inner.free_memory()
    }

    /// Free swap
    #[get]
    #[must_use]
    pub fn free_swap(&self) -> u64 {
        *self.inner.free_swap()
    }

    /// RSS
    #[get]
    #[must_use]
    pub fn rss(&self) -> u64 {
        *self.inner.rss()
    }

    /// Returns a string representation of these cgroup limits.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("CGroupLimits", &self.inner)
    }
}
