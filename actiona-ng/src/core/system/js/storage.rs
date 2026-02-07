use std::sync::Arc;

use derive_more::Display;
use itertools::Itertools;
use macros::{FromJsObject, FromSerde, IntoSerde};
use rquickjs::{Ctx, JsLifetime, Result, atom::PredefinedAtom, class::Trace, prelude::Opt};
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use sysinfo::DiskKind;

use crate::{
    IntoJsResult,
    core::{
        js::classes::{HostClass, register_enum, register_host_class},
        system::{
            Storage,
            storage::{Disk, DiskUsage, IoStats},
        },
    },
};

/// Storage devices and disk usage information.
///
/// ```ts
/// const disks = await system.storage.listDisks();
/// console.log(disks.length);
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Storage")]
pub struct JsStorage {
    inner: Arc<Storage>,
}

impl<'js> HostClass<'js> for JsStorage {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_host_class::<JsDisk>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsStorage {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsStorage {
    /// @skip
    #[must_use]
    pub const fn new(inner: Arc<Storage>) -> Self {
        Self { inner }
    }
}

/// List disks options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct ListDisksOptions {
    /// Rescan
    /// @default `true`
    pub rescan: bool,
}

impl Default for ListDisksOptions {
    fn default() -> Self {
        Self { rescan: true }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsStorage {
    /// Disks
    pub async fn list_disks<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<ListDisksOptions>,
    ) -> Result<Vec<JsDisk>> {
        let options = options.unwrap_or_default();
        Ok(self
            .inner
            .refresh_disks(options.rescan)
            .await
            .into_js_result(&ctx)?
            .into_iter()
            .map(JsDisk::from)
            .collect_vec())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

/// A disk device.
///
/// ```ts
/// const disks = await system.storage.listDisks();
/// const disk = disks[0];
/// if (disk) {
///   console.log(disk.name, disk.kind, disk.mountPoint);
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Disk")]
pub struct JsDisk {
    inner: Disk,
}

impl<'js> HostClass<'js> for JsDisk {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_host_class::<JsDiskUsage>(ctx)?;
        register_enum::<JsDiskKind>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsDisk {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<Disk> for JsDisk {
    fn from(value: Disk) -> Self {
        Self { inner: value }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsDisk {
    /// Kind
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn kind(&self) -> JsDiskKind {
        self.inner.kind().into()
    }

    /// Name
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.inner.name().as_deref()
    }

    /// File system
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn file_system(&self) -> Option<&str> {
        self.inner.file_system().as_deref()
    }

    /// Mount point
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn mount_point(&self) -> String {
        self.inner.mount_point().to_string_lossy().to_string()
    }

    /// Total space
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn total_space(&self) -> u64 {
        *self.inner.total_space()
    }

    /// Available space
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn available_space(&self) -> u64 {
        *self.inner.available_space()
    }

    /// Is removable
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn is_removable(&self) -> bool {
        self.inner.is_removable()
    }

    /// Is read-only
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        self.inner.is_read_only()
    }

    /// Usage
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn usage(&self) -> JsDiskUsage {
        self.inner.usage().into()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

/// Disk kind values.
///
/// ```ts
/// const disks = await system.storage.listDisks();
/// const disk = disks[0];
/// if (disk && disk.kind === DiskKind.SSD) {
///   console.log("SSD");
/// }
/// ```
///
/// Disk kind
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    Hash,
    IntoSerde,
    PartialEq,
    Serialize,
)]
pub enum JsDiskKind {
    /// Hard disk drive
    HDD,

    /// Solid-state drive
    SSD,

    /// Unknown drive kind
    Unknown,
}

impl From<DiskKind> for JsDiskKind {
    fn from(value: DiskKind) -> Self {
        match value {
            DiskKind::HDD => Self::HDD,
            DiskKind::SSD => Self::SSD,
            DiskKind::Unknown(_) => Self::Unknown,
        }
    }
}

/// Disk I/O statistics (bytes).
///
/// ```ts
/// const disks = await system.storage.listDisks();
/// const disk = disks[0];
/// if (disk) {
///   console.log(disk.usage.read.total, disk.usage.written.delta);
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "IoStats")]
pub struct JsIoStats {
    inner: IoStats,
}

impl<'js> HostClass<'js> for JsIoStats {}

impl<'js> Trace<'js> for JsIoStats {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<IoStats> for JsIoStats {
    fn from(value: IoStats) -> Self {
        Self { inner: value }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsIoStats {
    /// Total
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn total(&self) -> u64 {
        *self.inner.total()
    }

    /// Delta
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn delta(&self) -> u64 {
        *self.inner.delta()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

/// Read/write usage for a disk.
///
/// ```ts
/// const disks = await system.storage.listDisks();
/// const disk = disks[0];
/// if (disk) {
///   console.log(disk.usage.read.total, disk.usage.written.total);
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "DiskUsage")]
pub struct JsDiskUsage {
    inner: DiskUsage,
}

impl<'js> HostClass<'js> for JsDiskUsage {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_host_class::<JsIoStats>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsDiskUsage {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<DiskUsage> for JsDiskUsage {
    fn from(value: DiskUsage) -> Self {
        Self { inner: value }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsDiskUsage {
    /// Written
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn written(&self) -> JsIoStats {
        self.inner.written().into()
    }

    /// Read
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn read(&self) -> JsIoStats {
        self.inner.read().into()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}
