use std::sync::Arc;

use rquickjs::{Ctx, JsLifetime, Result, atom::PredefinedAtom, class::Trace};

use crate::{
    IntoJsResult,
    core::{
        js::classes::{HostClass, register_host_class},
        system::memory::{CGroupLimits, Memory, MemoryUsage},
    },
};

/// Memory metrics.
///
/// ```ts
/// const usage = await system.memory.usage();
/// const swap = await system.memory.swapUsage();
///
/// console.log(formatBytes(usage.used), formatBytes(swap.used));
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Memory")]
pub struct JsMemory {
    inner: Arc<Memory>,
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
    pub const fn new(inner: Arc<Memory>) -> Self {
        Self { inner }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
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

    // TODO: @platforms does not work on properties?
    /// CGroup limits
    /// @platforms =linux
    /// @get
    #[qjs(get)]
    pub fn cgroup_limits(&self) -> Option<JsCGroupLimits> {
        self.inner.cgroup_limits().map(JsCGroupLimits::from)
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

/// A memory usage snapshot.
///
/// ```ts
/// const usage = await system.memory.usage();
/// console.log(
///   formatBytes(usage.used),
///   formatBytes(usage.free),
///   formatBytes(usage.available),
///   formatBytes(usage.total),
/// );
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "MemoryUsage")]
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

#[rquickjs::methods(rename_all = "camelCase")]
impl JsMemoryUsage {
    /// Used
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn used(&self) -> u64 {
        *self.inner.used()
    }

    /// Free
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn free(&self) -> u64 {
        *self.inner.free()
    }

    /// Available
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn available(&self) -> u64 {
        *self.inner.available()
    }

    /// Total
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn total(&self) -> u64 {
        *self.inner.total()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

/// CGroup memory and swap limits.
///
/// ```ts
/// const limits = system.memory.cgroupLimits;
/// if (limits) {
///   console.log(
///     formatBytes(limits.totalMemory),
///     formatBytes(limits.freeMemory),
///     formatBytes(limits.freeSwap),
///   );
/// }
/// ```
///
/// CGroup limits
/// @platforms =linux
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "CGroupLimits")]
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

#[rquickjs::methods(rename_all = "camelCase")]
impl JsCGroupLimits {
    /// Total memory
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn total_memory(&self) -> u64 {
        *self.inner.total_memory()
    }

    /// Free memory
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn free_memory(&self) -> u64 {
        *self.inner.free_memory()
    }

    /// Free swap
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn free_swap(&self) -> u64 {
        *self.inner.free_swap()
    }

    /// RSS
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn rss(&self) -> u64 {
        *self.inner.rss()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}
