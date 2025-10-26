use std::{collections::HashMap, sync::Arc};

use derive_more::Display;
use macros::{FromSerde, IntoSerde};
use rquickjs::{Ctx, JsLifetime, Object, Result, atom::PredefinedAtom, class::Trace, prelude::Opt};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    IntoJsResult,
    core::{
        js::{
            classes::{HostClass, register_enum, register_host_class},
            date::date_from_system_time,
        },
        system::{
            js::storage::JsDiskUsage,
            processes::{Process, Processes, Status},
        },
    },
};

/// Processes
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Processes")]
pub struct JsProcesses {
    inner: Arc<Processes>,
}

impl<'js> HostClass<'js> for JsProcesses {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_host_class::<JsProcess>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsProcesses {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsProcesses {
    /// @skip
    #[must_use]
    pub const fn new(inner: Arc<Processes>) -> Self {
        Self { inner }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsProcesses {
    /// Processes
    pub async fn processes<'js>(
        &self,
        ctx: Ctx<'js>,
        rescan: Opt<bool>,
    ) -> Result<HashMap<u32, JsProcess>> {
        let rescan = rescan.0.unwrap_or(true);
        Ok(self
            .inner
            .refresh_processes(rescan, false) // We don't return threads (tasks) for now
            .await
            .into_js_result(&ctx)?
            .into_iter()
            .map(|(pid, process)| (pid.into(), process.into()))
            .collect::<HashMap<_, _>>())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

// Process
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Process")]
pub struct JsProcess {
    inner: Process,
}

impl<'js> HostClass<'js> for JsProcess {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_enum::<JsProcessStatus>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsProcess {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<Process> for JsProcess {
    fn from(value: Process) -> Self {
        Self { inner: value }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsProcess {
    /// Name
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.inner.name().as_deref()
    }

    /// Cmd
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn cmd(&self) -> &[String] {
        self.inner.cmd()
    }

    /// Exe
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn exe(&self) -> Option<String> {
        self.inner
            .exe()
            .as_ref()
            .map(|exe| exe.to_string_lossy().to_string())
    }

    /// Pid
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn pid(&self) -> u32 {
        self.inner.pid().into()
    }

    /// Env
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn env(&self) -> &[String] {
        self.inner.env()
    }

    /// Cwd
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn cwd(&self) -> Option<String> {
        self.inner
            .cwd()
            .as_ref()
            .map(|exe| exe.to_string_lossy().to_string())
    }

    /// Root
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn root(&self) -> Option<String> {
        self.inner
            .root()
            .as_ref()
            .map(|exe| exe.to_string_lossy().to_string())
    }

    /// Memory
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn memory(&self) -> u64 {
        *self.inner.memory()
    }

    /// Virtual memory
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn virtual_memory(&self) -> u64 {
        *self.inner.virtual_memory()
    }

    /// Parent
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn parent(&self) -> Option<u32> {
        self.inner.parent().map(|pid| pid.into())
    }

    /// Status
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn status(&self) -> JsProcessStatus {
        (*self.inner.status()).into()
    }

    /// Start time
    /// @get
    #[qjs(get)]
    pub fn start_time<'js>(&self, ctx: Ctx<'js>) -> Result<Object<'js>> {
        date_from_system_time(&ctx, &self.inner.start_time())
    }

    /// Run time
    /// @get
    #[qjs(get)]
    pub fn run_time(&self, ctx: Ctx<'_>) -> Result<u64> {
        u64::try_from((*self.inner.run_time()).as_millis()).into_js_result(&ctx)
    }

    /// CPU usage
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn cpu_usage(&self) -> f32 {
        *self.inner.cpu_usage()
    }

    /// Accumulated CPU time
    /// @get
    #[qjs(get)]
    pub fn accumulated_cpu_time(&self, ctx: Ctx<'_>) -> Result<u64> {
        u64::try_from((*self.inner.accumulated_cpu_time()).as_millis()).into_js_result(&ctx)
    }

    /// Disk usage
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn disk_usage(&self) -> JsDiskUsage {
        self.inner.disk_usage().into()
    }

    /// User ID
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn user_id(&self) -> Option<String> {
        self.inner.user_id().as_ref().map(|id| id.to_string())
    }

    /// Effective user ID
    /// @get
    /// @platforms =linux
    #[qjs(get)]
    #[must_use]
    pub fn effective_user_id(&self) -> Option<String> {
        self.inner
            .effective_user_id()
            .as_ref()
            .map(|id| id.to_string())
    }

    /// Group ID
    /// @get
    /// @platforms =linux
    #[qjs(get)]
    #[must_use]
    pub fn group_id(&self) -> Option<u32> {
        *self.inner.group_id()
    }

    /// Effective group ID
    /// @get
    /// @platforms =linux
    #[qjs(get)]
    #[must_use]
    pub fn effective_group_id(&self) -> Option<u32> {
        *self.inner.effective_group_id()
    }

    /// Session ID
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn session_id(&self) -> Option<u32> {
        *self.inner.session_id()
    }

    /// Exists
    /// @get
    #[qjs(get)]
    #[must_use]
    pub const fn exists(&self) -> bool {
        self.inner.exists()
    }

    /// Open files
    /// @get
    #[qjs(get)]
    pub fn open_files(&self, ctx: Ctx<'_>) -> Result<Option<u64>> {
        self.inner
            .open_files()
            .map(u64::try_from)
            .transpose()
            .into_js_result(&ctx)
    }

    /// Open files limit
    /// @get
    #[qjs(get)]
    pub fn open_files_limit(&self, ctx: Ctx<'_>) -> Result<Option<u64>> {
        self.inner
            .open_files_limit()
            .map(u64::try_from)
            .transpose()
            .into_js_result(&ctx)
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        self.inner.to_string()
    }
}

/// Process status
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
pub enum JsProcessStatus {
    Idle,
    Run,
    Sleep,
    Stop,
    Zombie,
    Tracing,
    Dead,
    Wakekill,
    Waking,
    Parked,
    LockBlocked,
    UninterruptibleDiskSleep,
    Unknown,
}

impl From<Status> for JsProcessStatus {
    fn from(value: Status) -> Self {
        match value {
            Status::Idle => Self::Idle,
            Status::Run => Self::Run,
            Status::Sleep => Self::Sleep,
            Status::Stop => Self::Stop,
            Status::Zombie => Self::Zombie,
            Status::Tracing => Self::Tracing,
            Status::Dead => Self::Dead,
            Status::Wakekill => Self::Wakekill,
            Status::Waking => Self::Waking,
            Status::Parked => Self::Parked,
            Status::LockBlocked => Self::LockBlocked,
            Status::UninterruptibleDiskSleep => Self::UninterruptibleDiskSleep,
            Status::Unknown(_) => Self::Unknown,
        }
    }
}
