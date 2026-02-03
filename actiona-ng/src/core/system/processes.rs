use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::Arc,
};

#[cfg(windows)]
use color_eyre::eyre;
use color_eyre::{Report, Result};
use derive_more::Display;
use derive_where::derive_where;
use parking_lot::Mutex;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, instrument};

#[cfg(unix)]
use super::platform::linux::ProcessSignal;
//#[cfg(windows)]
//use super::platform::win::ProcessSignal;
use crate::{
    core::system::storage::DiskUsage,
    types::{
        ByteCount, DurationUnit, OptionalPath, OptionalPid, OptionalSystemString, OptionalTaskList,
        OptionalThreadKind, OptionalU32, OptionalUSize, OptionalUidUnit, OsStringList, Percent,
        SystemTimeUnit,
        display::{DisplayFields, display_map},
        pid::Pid,
    },
};

/// TODO: Linux only
#[derive(Clone, Copy, Debug, Display)]
pub enum Signal {
    /// `SIGHUP` — “hang up”. Traditionally sent when a terminal disconnects.
    /// Commonly repurposed by daemons to mean “reload configuration”.
    Hup,

    /// `SIGINT` — “interrupt” (like pressing Ctrl-C). Asks the process to stop.
    /// Processes can handle/ignore it.
    Int,

    /// `SIGQUIT` — “quit” (like Ctrl-\\ on many systems). Similar to `SIGINT`
    /// but often triggers a core dump for diagnostics.
    Quit,

    /// `SIGTERM` — “terminate”. The *polite* way to ask a process to exit
    /// cleanly; gives it a chance to shut down.
    Term,

    /// `SIGKILL` — “kill immediately”. Cannot be caught, blocked, or ignored.
    /// Use as a last resort when `SIGTERM` fails.
    Kill,

    /// `SIGSTOP` — stop/suspend execution immediately (cannot be caught/ignored).
    /// Kernel-enforced stop (not the same as `TSTP`). Resume with `SIGCONT`.
    Stop,

    /// `SIGTSTP` — terminal stop (like pressing Ctrl-Z). Job-control friendly,
    /// can be caught/ignored. Resume with `SIGCONT`.
    Tstp,

    /// `SIGCONT` — continue a process that’s stopped (by `STOP`/`TSTP`/`TTIN`/`TTOU`).
    /// Has no effect if the process isn’t stopped.
    Cont,

    /// `SIGTTIN` — background process attempted to read from its controlling TTY.
    /// Typically stops the process (job control). Rarely sent manually but
    /// included due to common job-control semantics.
    Ttin,

    /// `SIGTTOU` — background process attempted to write to its controlling TTY.
    /// Typically stops the process (job control). Symmetric to `TTIN`.
    Ttou,

    /// `SIGWINCH` — window size change. Often used to notify TUI apps that the
    /// terminal size changed; safe to nudge full-screen apps to reflow.
    Winch,

    /// `SIGUSR1` — user-defined signal #1. App-specific semantic (reload, rotate logs, etc.).
    /// Check the target app’s docs.
    Usr1,

    /// `SIGUSR2` — user-defined signal #2. App-specific semantic, like `USR1`.
    Usr2,
}

#[derive(Clone, Copy, Debug, Display)]
pub enum Status {
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
    Unknown(u32),
}

impl From<sysinfo::ProcessStatus> for Status {
    fn from(value: sysinfo::ProcessStatus) -> Self {
        use Status::*;
        match value {
            sysinfo::ProcessStatus::Idle => Idle,
            sysinfo::ProcessStatus::Run => Run,
            sysinfo::ProcessStatus::Sleep => Sleep,
            sysinfo::ProcessStatus::Stop => Stop,
            sysinfo::ProcessStatus::Zombie => Zombie,
            sysinfo::ProcessStatus::Tracing => Tracing,
            sysinfo::ProcessStatus::Dead => Dead,
            sysinfo::ProcessStatus::Wakekill => Wakekill,
            sysinfo::ProcessStatus::Waking => Waking,
            sysinfo::ProcessStatus::Parked => Parked,
            sysinfo::ProcessStatus::LockBlocked => LockBlocked,
            sysinfo::ProcessStatus::UninterruptibleDiskSleep => UninterruptibleDiskSleep,
            sysinfo::ProcessStatus::Suspended => Suspended,
            sysinfo::ProcessStatus::Unknown(status) => Unknown(status),
        }
    }
}

#[derive(Clone, Debug, Display)]
pub enum ThreadKind {
    Kernel,
    Userland,
}

impl From<sysinfo::ThreadKind> for ThreadKind {
    fn from(value: sysinfo::ThreadKind) -> Self {
        use ThreadKind::*;
        match value {
            sysinfo::ThreadKind::Kernel => Kernel,
            sysinfo::ThreadKind::Userland => Userland,
        }
    }
}

#[derive(Debug)]
pub struct Process {
    name: OptionalSystemString,
    cmd: OsStringList,
    exe: OptionalPath,
    pid: Pid,
    env: OsStringList,
    cwd: OptionalPath,
    root: OptionalPath,
    memory: ByteCount,
    virtual_memory: ByteCount,
    parent: OptionalPid,
    status: Status,
    start_time: SystemTimeUnit,
    run_time: DurationUnit,             // dyn
    cpu_usage: Percent,                 // dyn
    accumulated_cpu_time: DurationUnit, // dyn
    disk_usage: DiskUsage,              // dyn
    user_id: OptionalUidUnit,
    effective_user_id: OptionalUidUnit, // Linux only
    group_id: OptionalU32,              // Linux only
    effective_group_id: OptionalU32,    // Linux only
    session_id: OptionalU32,
    tasks: OptionalTaskList,         // Linux only
    thread_kind: OptionalThreadKind, // Linux only
    exists: bool,
    open_files: OptionalUSize,
    open_files_limit: OptionalUSize,
}

impl TryFrom<&sysinfo::Process> for Process {
    type Error = Report;

    fn try_from(value: &sysinfo::Process) -> Result<Self> {
        Ok(Self {
            name: value.name().into(),
            cmd: value.cmd().into(),
            exe: value.exe().into(),
            pid: value.pid().try_into()?,
            env: value.environ().into(),
            cwd: value.cwd().into(),
            root: value.root().into(),
            memory: value.memory().into(),
            virtual_memory: value.virtual_memory().into(),
            parent: value.parent().map(|pid| pid.try_into()).transpose()?.into(),
            status: value.status().into(),
            start_time: SystemTimeUnit::from_unix_epoch(value.start_time()),
            run_time: DurationUnit::from_secs(value.run_time()),
            cpu_usage: value.cpu_usage().into(),
            accumulated_cpu_time: DurationUnit::from_secs(value.accumulated_cpu_time()),
            disk_usage: value.disk_usage().into(),
            user_id: value.user_id().cloned().into(),
            effective_user_id: value.effective_user_id().cloned().into(),
            group_id: value.group_id().map(|gid| *gid).into(),
            effective_group_id: value.effective_group_id().map(|gid| *gid).into(),
            session_id: value.session_id().map(|pid| pid.as_u32()).into(),
            tasks: value
                .tasks()
                .map(|tasks| tasks.iter().map(|pid| pid.as_u32()).collect::<HashSet<_>>())
                .into(),
            thread_kind: value.thread_kind().into(),
            exists: value.exists(),
            open_files: value.open_files().into(),
            open_files_limit: value.open_files_limit().into(),
        })
    }
}

impl TryFrom<sysinfo::Process> for Process {
    type Error = Report;

    fn try_from(value: sysinfo::Process) -> Result<Self> {
        (&value).try_into()
    }
}

impl Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            DisplayFields::default()
                .display_if_some("name", &self.name)
                .display("cmd", &self.cmd)
                .display_if_some("exe", &self.exe)
                .display("pid", self.pid)
                .display("env", &self.env)
                .display_if_some("cwd", &self.cwd)
                .display_if_some("root", &self.root)
                .display("memory", self.memory)
                .display("virtual_memory", self.virtual_memory)
                .display_if_some("parent", &self.parent)
                .display("status", self.status)
                .display("start_time", self.start_time)
                .display("run_time", self.run_time)
                .display("cpu_usage", self.cpu_usage)
                .display("accumulated_cpu_time", self.accumulated_cpu_time)
                .display("disk_usage", self.disk_usage)
                .display_if_some("user_id", &self.user_id)
                .display_if_some("effective_user_id", &self.effective_user_id)
                .display_if_some("group_id", &self.group_id)
                .display_if_some("effective_group_id", &self.effective_group_id)
                .display_if_some("session_id", &self.session_id)
                .display_if_some("tasks", &self.tasks)
                .display_if_some("thread_kind", &self.thread_kind)
                .display("exists", self.exists)
                .display_if_some("open_files", &self.open_files)
                .display_if_some("open_files_limit", &self.open_files_limit)
                .finish(f)
        } else {
            DisplayFields::default()
                .display_if_some("name", &self.name)
                .display_if_some("exe", &self.exe)
                .display("pid", self.pid)
                .display_if_some("cwd", &self.cwd)
                .display("memory", self.memory)
                .display_if_some("parent", &self.parent)
                .display("status", self.status)
                .display("start_time", self.start_time)
                .display("run_time", self.run_time)
                .display("cpu_usage", self.cpu_usage)
                .display_if_some("user_id", &self.user_id)
                .display("exists", self.exists)
                .finish(f)
        }
    }
}

impl Process {
    #[must_use]
    pub const fn name(&self) -> &OptionalSystemString {
        &self.name
    }

    #[must_use]
    pub const fn cmd(&self) -> &OsStringList {
        &self.cmd
    }

    #[must_use]
    pub const fn exe(&self) -> &OptionalPath {
        &self.exe
    }

    #[must_use]
    pub const fn pid(&self) -> Pid {
        self.pid
    }

    #[must_use]
    pub const fn env(&self) -> &OsStringList {
        &self.env
    }

    #[must_use]
    pub const fn cwd(&self) -> &OptionalPath {
        &self.cwd
    }

    #[must_use]
    pub const fn root(&self) -> &OptionalPath {
        &self.root
    }

    #[must_use]
    pub const fn memory(&self) -> ByteCount {
        self.memory
    }

    #[must_use]
    pub const fn virtual_memory(&self) -> ByteCount {
        self.virtual_memory
    }

    #[must_use]
    pub const fn parent(&self) -> OptionalPid {
        self.parent
    }

    #[must_use]
    pub const fn status(&self) -> &Status {
        &self.status
    }

    #[must_use]
    pub const fn start_time(&self) -> SystemTimeUnit {
        self.start_time
    }

    /// dyn
    #[must_use]
    pub const fn run_time(&self) -> DurationUnit {
        self.run_time
    }

    /// dyn
    #[must_use]
    pub const fn cpu_usage(&self) -> Percent {
        self.cpu_usage
    }

    /// dyn
    #[must_use]
    pub const fn accumulated_cpu_time(&self) -> DurationUnit {
        self.accumulated_cpu_time
    }

    /// dyn
    #[must_use]
    pub const fn disk_usage(&self) -> DiskUsage {
        self.disk_usage
    }

    #[must_use]
    pub const fn user_id(&self) -> &OptionalUidUnit {
        &self.user_id
    }

    /// Linux only
    #[must_use]
    pub const fn effective_user_id(&self) -> &OptionalUidUnit {
        &self.effective_user_id
    }

    /// Linux only
    #[must_use]
    pub const fn group_id(&self) -> OptionalU32 {
        self.group_id
    }

    /// Linux only
    #[must_use]
    pub const fn effective_group_id(&self) -> OptionalU32 {
        self.effective_group_id
    }

    #[must_use]
    pub const fn session_id(&self) -> OptionalU32 {
        self.session_id
    }

    /// Linux only
    #[must_use]
    pub const fn tasks(&self) -> &OptionalTaskList {
        &self.tasks
    }

    /// Linux only
    #[must_use]
    pub const fn thread_kind(&self) -> &OptionalThreadKind {
        &self.thread_kind
    }

    #[must_use]
    pub const fn exists(&self) -> bool {
        self.exists
    }

    #[must_use]
    pub const fn open_files(&self) -> OptionalUSize {
        self.open_files
    }

    #[must_use]
    pub const fn open_files_limit(&self) -> OptionalUSize {
        self.open_files_limit
    }
}

#[derive_where(Debug)]
pub struct Processes {
    #[derive_where(skip)]
    system: Arc<Mutex<sysinfo::System>>,

    #[derive_where(skip)]
    task_tracker: TaskTracker,
}

impl Display for Processes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let processes = self.processes().map_err(|err| {
            error!("fetching processes failed: {err}");

            std::fmt::Error
        })?;
        DisplayFields::default()
            .display("processes", display_map(&processes))
            .finish(f)
    }
}

impl Processes {
    #[instrument(name = "processes", skip_all)]
    pub async fn new(task_tracker: TaskTracker) -> Result<Self> {
        let system = task_tracker.spawn_blocking(sysinfo::System::new).await?;

        Ok(Self {
            system: Arc::new(Mutex::new(system)),
            task_tracker,
        })
    }

    pub async fn refresh_processes(
        &self,
        rescan: bool,
        tasks: bool,
    ) -> Result<HashMap<Pid, Process>> {
        let system = self.system.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut system = system.lock();
                system.refresh_processes_specifics(
                    ProcessesToUpdate::All,
                    rescan,
                    if tasks {
                        ProcessRefreshKind::everything()
                    } else {
                        ProcessRefreshKind::everything().without_tasks()
                    },
                );
                system
                    .processes()
                    .iter()
                    .map(|(pid, process)| {
                        let pid = Pid::try_from(*pid)?;
                        let process = process.try_into()?;

                        Ok((pid, process))
                    })
                    .collect::<Result<HashMap<_, _>>>()
            })
            .await??;
        Ok(result)
    }

    pub async fn refresh_process(&self, process: &Process, tasks: bool) -> Result<Option<Process>> {
        let process_id = process.pid.into();
        let system = self.system.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut system = system.lock();
                system.refresh_processes_specifics(
                    ProcessesToUpdate::Some(&[process_id]),
                    false,
                    if tasks {
                        ProcessRefreshKind::everything()
                    } else {
                        ProcessRefreshKind::everything().without_tasks()
                    },
                );
                system
                    .process(process_id)
                    .map(Process::try_from)
                    .transpose()
            })
            .await??;
        Ok(result)
    }

    pub fn processes(&self) -> Result<HashMap<Pid, Process>> {
        let system = self.system.lock();
        system
            .processes()
            .iter()
            .map(|(pid, process)| {
                let pid = Pid::try_from(*pid)?;
                let process = process.try_into()?;

                Ok((pid, process))
            })
            .collect::<Result<HashMap<_, _>>>()
    }

    /// Linux only
    pub fn send_signal(&self, pid: Pid, signal: Signal) -> Result<()> {
        #[cfg(unix)]
        {
            ProcessSignal::send_signal(pid, signal)
        }

        #[cfg(windows)]
        {
            _ = pid;
            _ = signal;
            Err(eyre!("signals are not supported on Windows"))
        }
    }

    /// Linux only
    pub async fn send_signal_and_wait(
        &self,
        pid: Pid,
        signal: Signal,
        cancellation_token: CancellationToken,
    ) -> Result<Option<i32>> {
        #[cfg(unix)]
        {
            ProcessSignal::send_signal_and_wait(pid, signal, cancellation_token).await
        }

        #[cfg(windows)]
        {
            _ = pid;
            _ = signal;
            _ = cancellation_token;
            Err(eyre!("signals are not supported on Windows"))
        }
    }

    pub async fn from_pid(&self, pid: Pid) -> Result<Option<Process>> {
        let process_id = pid.into();
        let system = self.system.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut system = system.lock();
                system.refresh_processes_specifics(
                    ProcessesToUpdate::Some(&[process_id]),
                    false,
                    ProcessRefreshKind::everything().without_tasks(),
                );
                system
                    .process(process_id)
                    .map(Process::try_from)
                    .transpose()
            })
            .await??;
        Ok(result)
    }

    // TODO: send signals to processes: process_signal
}
