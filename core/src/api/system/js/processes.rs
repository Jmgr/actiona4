use derive_more::Display;
use itertools::Itertools;
use macros::{FromJsObject, FromSerde, IntoSerde};
use rquickjs::{
    Ctx, JsLifetime, Object, Result, Value, atom::PredefinedAtom, class::Trace, prelude::Opt,
};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use crate::{
    IntoJsResult,
    api::{
        ResultExt,
        js::{
            classes::{HostClass, register_enum, register_host_class},
            date::date_from_system_time,
        },
        name::js::JsName,
        process::js::JsSignal,
        system::{
            js::storage::JsDiskUsage,
            processes::{Process, Processes, Status},
        },
    },
    types::display::display_with_type,
};

/// Process listing and inspection.
///
/// ```ts
/// const processes = await system.processes.list();
/// println(processes.length);
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Processes")]
pub struct JsProcesses {
    inner: Processes,
}

impl<'js> HostClass<'js> for JsProcesses {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_host_class::<JsProcessInfo>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsProcesses {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl JsProcesses {
    /// @skip
    #[must_use]
    pub const fn new(inner: Processes) -> Self {
        Self { inner }
    }
}

/// List processes options
/// @options
#[derive(Clone, Copy, Debug, FromJsObject)]
pub struct ListProcessesOptions {
    /// Rescan
    /// @default `true`
    pub rescan: bool,
}

impl Default for ListProcessesOptions {
    fn default() -> Self {
        Self { rescan: true }
    }
}

/// Process search options.
/// @options
#[derive(Debug)]
pub struct JsProcessesFindOptions<'js> {
    /// Match by process ID.
    /// When undefined, any PID is accepted.
    /// @default `undefined`
    pub pid: Option<u32>,

    /// Match by parent process ID.
    /// When undefined, parent PID is not filtered.
    /// @default `undefined`
    pub parent_pid: Option<u32>,

    /// Match by process name.
    /// When undefined, name is not filtered.
    /// @default `undefined`
    pub name: Option<JsName<'js>>,

    /// Match by process status.
    /// When undefined, status is not filtered.
    /// @default `undefined`
    pub status: Option<JsProcessStatus>,

    /// Refresh process list before filtering.
    /// @default `true`
    pub rescan: bool,
}

impl<'js> Default for JsProcessesFindOptions<'js> {
    fn default() -> Self {
        Self {
            pid: None,
            parent_pid: None,
            name: None,
            status: None,
            rescan: true,
        }
    }
}

impl<'js> rquickjs::FromJs<'js> for JsProcessesFindOptions<'js> {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        if value.is_undefined() || value.is_null() {
            return Ok(Self::default());
        }

        let object = value
            .into_object()
            .or_throw_message(ctx, "Expected an object")?;

        let rescan: Option<bool> = object.get("rescan")?;

        Ok(Self {
            pid: object.get("pid")?,
            parent_pid: object.get("parentPid")?,
            name: object.get("name")?,
            status: object.get("status")?,
            rescan: rescan.unwrap_or(true),
        })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsProcesses {
    /// Lists all processes
    /// @readonly
    pub async fn list<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<ListProcessesOptions>,
    ) -> Result<Vec<JsProcessInfo>> {
        let options = options.unwrap_or_default();
        Ok(self
            .inner
            .refresh_processes(options.rescan, false) // We don't return threads (tasks) for now
            .await
            .into_js_result(&ctx)?
            .into_values()
            .map(|process| process.into())
            .collect_vec())
    }

    /// Finds processes matching the provided criteria.
    /// ```ts
    /// const byPid = await system.processes.find({ pid: 12345 });
    /// const byParent = await system.processes.find({ parentPid: 1 });
    /// const byName = await system.processes.find({ name: new Wildcard("my-app*") });
    /// const running = await system.processes.find({ status: ProcessStatus.Run });
    /// const exact = await system.processes.find({ pid: 12345, name: "my-app" });
    /// ```
    /// @readonly
    pub async fn find<'js>(
        &self,
        ctx: Ctx<'js>,
        options: JsProcessesFindOptions<'js>,
    ) -> Result<Vec<JsProcessInfo>> {
        let processes = self
            .inner
            .refresh_processes(options.rescan, false)
            .await
            .into_js_result(&ctx)?;

        let mut matching = Vec::new();
        for process in processes.into_values() {
            if let Some(filter_pid) = options.pid {
                let pid: u32 = process.pid().into();
                if pid != filter_pid {
                    continue;
                }
            }

            if let Some(filter_parent_pid) = options.parent_pid {
                let parent_pid = process.parent().map(|pid| pid.into());
                if parent_pid != Some(filter_parent_pid) {
                    continue;
                }
            }

            if let Some(filter_status) = options.status
                && JsProcessStatus::from(*process.status()) != filter_status
            {
                continue;
            }

            if let Some(filter_name) = options.name.as_ref() {
                let process_name = process.name().as_deref().unwrap_or_default();
                if !filter_name.inner().matches(&ctx, process_name) {
                    continue;
                }
            }

            matching.push(process.into());
        }

        Ok(matching)
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Processes", &self.inner)
    }
}

/// A process information entry.
///
/// ```ts
/// const processes = await system.processes.list();
/// const process = processes[0];
/// if (process) {
///   println(process.pid, process.name, process.status);
/// }
/// ```
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "ProcessInfo")]
pub struct JsProcessInfo {
    inner: Process,
}

impl<'js> HostClass<'js> for JsProcessInfo {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_enum::<JsProcessStatus>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsProcessInfo {
    fn trace<'a>(&self, _tracer: rquickjs::class::Tracer<'a, 'js>) {}
}

impl From<Process> for JsProcessInfo {
    fn from(value: Process) -> Self {
        Self { inner: value }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsProcessInfo {
    /// Name
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.inner.name().as_deref()
    }

    /// Cmd
    /// @get
    /// @readonly
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
    /// @readonly
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

    /// Run time in seconds
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn run_time(&self) -> f64 {
        self.inner.run_time().as_secs_f64()
    }

    /// CPU usage
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn cpu_usage(&self) -> f32 {
        *self.inner.cpu_usage()
    }

    /// Accumulated CPU time in seconds
    /// @get
    #[qjs(get)]
    #[must_use]
    pub fn accumulated_cpu_time(&self) -> f64 {
        self.inner.accumulated_cpu_time().as_secs_f64()
    }

    /// Disk usage
    /// @get
    /// @readonly
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

    /// Kill the process immediately (SIGKILL on Unix, TerminateProcess on Windows).
    ///
    /// ```ts
    /// // Force-stop a specific PID if it is still running.
    /// const targetPid = 12345;
    /// const proc = (await system.processes.find({ pid: targetPid }))[0];
    /// if (proc) await proc.kill();
    /// ```
    pub async fn kill(&self, ctx: Ctx<'_>) -> Result<()> {
        crate::api::process::kill_by_pid(self.inner.pid()).into_js_result(&ctx)
    }

    /// Gracefully terminate the process (SIGTERM on Unix, WM_CLOSE on Windows).
    ///
    /// ```ts
    /// // Ask a specific PID to shut down cleanly.
    /// const targetPid = 12345;
    /// const proc = (await system.processes.find({ pid: targetPid }))[0];
    /// if (proc) await proc.terminate();
    /// ```
    pub async fn terminate(&self, ctx: Ctx<'_>) -> Result<()> {
        crate::api::process::terminate_by_pid(self.inner.pid()).into_js_result(&ctx)
    }

    /// Send a signal to the process.
    ///
    /// ```ts
    /// const targetPid = 12345;
    /// const proc = (await system.processes.find({ pid: targetPid }))[0];
    /// if (proc) await proc.sendSignal(Signal.Term);
    /// ```
    ///
    /// @platforms =linux
    #[cfg(unix)]
    pub async fn send_signal(&self, ctx: Ctx<'_>, signal: JsSignal) -> Result<()> {
        crate::api::process::send_signal(self.inner.pid(), signal.into()).into_js_result(&ctx)
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("ProcessInfo", &self.inner)
    }
}

/// Process status.
///
/// ```ts
/// const processes = await system.processes.list();
/// const process = processes[0];
/// if (process && process.status === ProcessStatus.Run) {
///   println("process is running");
/// }
/// ```
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
    Suspended,
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
            Status::Suspended => Self::Suspended,
            Status::Unknown(_) => Self::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::Runtime;

    #[test]
    fn test_find() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>(
                    r#"
                const processes = await system.processes.list();
                const sample = processes[0];
                if (!sample) {
                    throw new Error("Expected at least one process");
                }

                const byPid = await system.processes.find({ pid: sample.pid, rescan: false });
                if (byPid.length === 0) {
                    throw new Error("pid filter mismatch");
                }

                let pidMatched = false;
                for (const process of byPid) {
                    if (process.pid === sample.pid) {
                        pidMatched = true;
                    }
                }
                if (!pidMatched) {
                    throw new Error("pid filter returned unexpected process set");
                }

                const byStatus = await system.processes.find({ status: sample.status, rescan: false });
                if (byStatus.length === 0) {
                    throw new Error("status filter mismatch");
                }

                if (sample.name !== undefined) {
                    const byName = await system.processes.find({ name: sample.name, rescan: false });
                    if (byName.length === 0) {
                        throw new Error("name filter mismatch");
                    }
                }
                "#,
                )
                .await
                .unwrap();
        });
    }
}
