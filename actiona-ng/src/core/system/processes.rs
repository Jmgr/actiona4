use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::{Arc, Mutex},
};

use eyre::Result;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind};
use tokio_util::task::TaskTracker;
use tracing::instrument;

use crate::{
    core::system::storage::DiskUsage,
    types::{
        ByteCount, DisplayFields, DurationUnit, OptionalPath, OptionalSystemString,
        OptionalTaskList, OptionalThreadKind, OptionalU32, OptionalUSize, OsStringList, Percent,
        SystemTimeUnit, display_map,
    },
};

#[derive(Debug, derive_more::Display)]
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
            sysinfo::ProcessStatus::Unknown(status) => Unknown(status),
        }
    }
}

#[derive(Debug, Clone, derive_more::Display)]
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
    pid: u32,
    env: OsStringList,
    cwd: OptionalPath,
    root: OptionalPath,
    memory: ByteCount,         // dyn
    virtual_memory: ByteCount, // dyn
    parent: OptionalU32,
    status: Status, // dyn
    start_time: SystemTimeUnit,
    run_time: DurationUnit,             // dyn
    cpu_usage: Percent,                 // dyn
    accumulated_cpu_time: DurationUnit, // dyn
    disk_usage: DiskUsage,              // dyn
    user_id: OptionalU32,
    effective_user_id: OptionalU32,  // Linux only
    group_id: OptionalU32,           // Linux only
    effective_group_id: OptionalU32, // Linux only
    session_id: OptionalU32,
    tasks: OptionalTaskList,         // Linux only
    thread_kind: OptionalThreadKind, // Linux only
    exists: bool,
    open_files: OptionalUSize,       // dyn
    open_files_limit: OptionalUSize, // dyn
}

impl From<&sysinfo::Process> for Process {
    fn from(value: &sysinfo::Process) -> Self {
        Self {
            name: value.name().into(),
            cmd: value.cmd().into(),
            exe: value.exe().into(),
            pid: value.pid().as_u32(),
            env: value.environ().into(),
            cwd: value.cwd().into(),
            root: value.root().into(),
            memory: value.memory().into(),
            virtual_memory: value.virtual_memory().into(),
            parent: value.parent().map(|pid| pid.as_u32()).into(),
            status: value.status().into(),
            start_time: SystemTimeUnit::from_unix_epoch(value.start_time()),
            run_time: DurationUnit::from_secs(value.run_time()),
            cpu_usage: value.cpu_usage().into(),
            accumulated_cpu_time: DurationUnit::from_secs(value.accumulated_cpu_time()),
            disk_usage: value.disk_usage().into(),
            user_id: value.user_id().map(|pid| **pid).into(),
            effective_user_id: value.effective_user_id().map(|pid| **pid).into(),
            group_id: value.group_id().map(|pid| *pid).into(),
            effective_group_id: value.effective_group_id().map(|pid| *pid).into(),
            session_id: value.session_id().map(|pid| pid.as_u32()).into(),
            tasks: value
                .tasks()
                .map(|tasks| tasks.iter().map(|pid| pid.as_u32()).collect::<HashSet<_>>())
                .into(),
            thread_kind: value.thread_kind().into(),
            exists: value.exists(),
            open_files: value.open_files().into(),
            open_files_limit: value.open_files_limit().into(),
        }
    }
}

impl From<sysinfo::Process> for Process {
    fn from(value: sysinfo::Process) -> Self {
        (&value).into()
    }
}

impl Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            DisplayFields::default()
                .display_if_some("name", &self.name)
                .display("cmd", &self.cmd)
                .display_if_some("exe", &self.exe)
                .display("pid", &self.pid)
                .display("env", &self.env)
                .display_if_some("cwd", &self.cwd)
                .display_if_some("root", &self.root)
                .display("memory", &self.memory)
                .display("virtual_memory", &self.virtual_memory)
                .display_if_some("parent", &self.parent)
                .display("status", &self.status)
                .display("start_time", &self.start_time)
                .display("run_time", &self.run_time)
                .display("cpu_usage", &self.cpu_usage)
                .display("accumulated_cpu_time", &self.accumulated_cpu_time)
                .display("disk_usage", &self.disk_usage)
                .display_if_some("user_id", &self.user_id)
                .display_if_some("effective_user_id", &self.effective_user_id)
                .display_if_some("group_id", &self.group_id)
                .display_if_some("effective_group_id", &self.effective_group_id)
                .display_if_some("session_id", &self.session_id)
                .display_if_some("tasks", &self.tasks)
                .display_if_some("thread_kind", &self.thread_kind)
                .display("exists", &self.exists)
                .display_if_some("open_files", &self.open_files)
                .display_if_some("open_files_limit", &self.open_files_limit)
                .finish(f)
        } else {
            DisplayFields::default()
                .display_if_some("name", &self.name)
                .display_if_some("exe", &self.exe)
                .display("pid", &self.pid)
                .display_if_some("cwd", &self.cwd)
                .display("memory", &self.memory)
                .display_if_some("parent", &self.parent)
                .display("status", &self.status)
                .display("start_time", &self.start_time)
                .display("run_time", &self.run_time)
                .display("cpu_usage", &self.cpu_usage)
                .display_if_some("user_id", &self.user_id)
                .display("exists", &self.exists)
                .finish(f)
        }
    }
}

#[derive_where::derive_where(Debug)]
pub struct Processes {
    #[derive_where(skip)]
    system: Arc<Mutex<sysinfo::System>>,

    #[derive_where(skip)]
    task_tracker: TaskTracker,
}

impl Display for Processes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("processes", display_map(&self.processes()))
            .finish(f)
    }
}

impl Processes {
    #[instrument(name = "processes", skip_all)]
    pub async fn new(task_tracker: TaskTracker) -> Result<Self> {
        let system = task_tracker
            .spawn_blocking(move || {
                sysinfo::System::new_with_specifics(
                    RefreshKind::nothing()
                        .with_processes(ProcessRefreshKind::everything().without_tasks()),
                )
            })
            .await?;

        Ok(Self {
            system: Arc::new(Mutex::new(system)),
            task_tracker,
        })
    }

    pub async fn refresh_processes(
        &self,
        rescan: bool,
        tasks: bool,
    ) -> Result<HashMap<u32, Process>> {
        let system = self.system.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut system = system.lock().unwrap();
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
                    .map(|(pid, process)| (pid.as_u32(), process.into()))
                    .collect::<HashMap<_, _>>()
            })
            .await?;
        Ok(result)
    }

    pub async fn refresh_process(&self, process: &Process, tasks: bool) -> Result<Option<Process>> {
        let process_id = Pid::from_u32(process.pid);
        let system = self.system.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut system = system.lock().unwrap();
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
                    .map(|process| Process::from(&*process))
            })
            .await?;
        Ok(result)
    }

    pub fn processes(&self) -> HashMap<u32, Process> {
        let system = self.system.lock().unwrap();
        system
            .processes()
            .iter()
            .map(|(pid, process)| (pid.as_u32(), process.into()))
            .collect::<HashMap<_, _>>()
    }
}
