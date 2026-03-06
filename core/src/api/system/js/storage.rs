use derive_more::Display;
use itertools::Itertools;
use macros::{FromJsObject, FromSerde, IntoSerde, js_class, js_methods, options};
use rquickjs::{Ctx, JsLifetime, Result, atom::PredefinedAtom, class::Trace, prelude::Opt};
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use sysinfo::DiskKind;

use crate::{
    IntoJsResult,
    api::{
        js::classes::{HostClass, register_enum, register_host_class},
        system::{
            Storage,
            storage::{Disk, DiskUsage, IoStats},
        },
    },
    types::display::display_with_type,
};

/// Storage devices and disk usage information.
///
/// ```ts
/// const disks = await system.storage.listDisks();
/// println(disks.length);
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsStorage {
    inner: Storage,
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
    pub const fn new(inner: Storage) -> Self {
        Self { inner }
    }
}

/// List disks options
#[options]
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct ListDisksOptions {
    /// Rescan
    #[default(true)]
    pub rescan: bool,
}

#[js_methods]
impl JsStorage {
    /// Disks
    /// @readonly
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
        display_with_type("Storage", &self.inner)
    }
}

/// A disk device.
///
/// ```ts
/// const disks = await system.storage.listDisks();
/// const disk = disks[0];
/// if (disk) {
///   println(
///     disk.name,
///     disk.kind,
///     disk.mountPoint,
///     formatBytes(disk.totalSpace),
///     formatBytes(disk.availableSpace),
///   );
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
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

#[js_methods]
impl JsDisk {
    /// Kind
    #[get]
    #[must_use]
    pub fn kind(&self) -> JsDiskKind {
        self.inner.kind().into()
    }

    /// Name
    #[get]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.inner.name().as_deref()
    }

    /// File system
    #[get]
    #[must_use]
    pub fn file_system(&self) -> Option<&str> {
        self.inner.file_system().as_deref()
    }

    /// Mount point
    #[get]
    #[must_use]
    pub fn mount_point(&self) -> String {
        self.inner.mount_point().to_string_lossy().to_string()
    }

    /// Total space
    #[get]
    #[must_use]
    pub fn total_space(&self) -> u64 {
        *self.inner.total_space()
    }

    /// Available space
    #[get]
    #[must_use]
    pub fn available_space(&self) -> u64 {
        *self.inner.available_space()
    }

    /// Is removable
    #[get]
    #[must_use]
    pub const fn is_removable(&self) -> bool {
        self.inner.is_removable()
    }

    /// Is read-only
    #[get]
    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        self.inner.is_read_only()
    }

    /// Usage
    /// @readonly
    #[get]
    #[must_use]
    pub fn usage(&self) -> JsDiskUsage {
        self.inner.usage().into()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Disk", &self.inner)
    }
}

/// Disk kind values.
///
/// ```ts
/// const disks = await system.storage.listDisks();
/// const disk = disks[0];
/// if (disk && disk.kind === DiskKind.SSD) {
///   println("SSD");
/// }
/// ```
///
/// Disk kind
///
/// @expand
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
#[serde(rename = "DiskKind")]
pub enum JsDiskKind {
    /// Hard disk drive
    /// `DiskKind.HDD`
    HDD,

    /// Solid-state drive
    /// `DiskKind.SSD`
    SSD,

    /// Unknown drive kind
    /// `DiskKind.Unknown`
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
///   println(
///     formatBytes(disk.usage.read.total),
///     formatBytes(disk.usage.written.delta),
///   );
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
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

#[js_methods]
impl JsIoStats {
    /// Total
    #[get]
    #[must_use]
    pub fn total(&self) -> u64 {
        *self.inner.total()
    }

    /// Delta
    #[get]
    #[must_use]
    pub fn delta(&self) -> u64 {
        *self.inner.delta()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("IoStats", self.inner)
    }
}

/// Read/write usage for a disk.
///
/// ```ts
/// const disks = await system.storage.listDisks();
/// const disk = disks[0];
/// if (disk) {
///   println(
///     formatBytes(disk.usage.read.total),
///     formatBytes(disk.usage.written.total),
///   );
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[js_class]
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

#[js_methods]
impl JsDiskUsage {
    /// Written
    /// @readonly
    #[get]
    #[must_use]
    pub fn written(&self) -> JsIoStats {
        self.inner.written().into()
    }

    /// Read
    /// @readonly
    #[get]
    #[must_use]
    pub fn read(&self) -> JsIoStats {
        self.inner.read().into()
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("DiskUsage", self.inner)
    }
}
