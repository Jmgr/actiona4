use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex},
};

use color_eyre::Result;
use derive_where::derive_where;
use itertools::Itertools;
use tokio_util::task::TaskTracker;
use tracing::instrument;

use crate::types::{
    DurationUnit, OptionalSystemString, SystemTimeUnit, UidUnit,
    display::{DisplayFields, display_list, display_map},
};

#[derive(Debug)]
pub struct Group {
    name: String,
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("name", &self.name)
            .finish(f)
    }
}

impl From<&sysinfo::Group> for Group {
    fn from(value: &sysinfo::Group) -> Self {
        Self {
            name: value.name().to_string(),
        }
    }
}

impl From<sysinfo::Group> for Group {
    fn from(value: sysinfo::Group) -> Self {
        (&value).into()
    }
}

impl Group {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug)]
pub struct User {
    name: String,
    group_id: Option<u32>,
    groups: Vec<u32>,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("name", &self.name)
            .display_if_some("group_id", &self.group_id)
            .display("groups", display_list(&self.groups))
            .finish(f)
    }
}

impl From<&sysinfo::User> for User {
    fn from(value: &sysinfo::User) -> Self {
        Self {
            name: value.name().to_string(),
            #[cfg(windows)]
            group_id: None,
            #[cfg(not(windows))]
            group_id: Some(*value.group_id()),
            groups: value
                .groups()
                .into_iter()
                .map(|group| **group.id())
                .collect_vec(),
        }
    }
}

impl User {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn group_id(&self) -> Option<u32> {
        self.group_id
    }

    #[must_use]
    pub fn groups(&self) -> &[u32] {
        &self.groups
    }
}

#[derive_where(Debug)]
pub struct Os {
    #[derive_where(skip)]
    users: Arc<Mutex<sysinfo::Users>>,

    #[derive_where(skip)]
    groups: Arc<Mutex<sysinfo::Groups>>,

    name: OptionalSystemString,
    kernel_version: OptionalSystemString,
    version: OptionalSystemString,
    long_version: OptionalSystemString,
    distribution_id: String,
    distribution_id_like: Vec<String>,
    kernel_long_version: String,

    #[derive_where(skip)]
    task_tracker: TaskTracker,
}

impl Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            DisplayFields::default()
                .display_if_some("name", &self.name)
                .display("uptime", self.uptime())
                .display("boot_time", self.boot_time())
                .display_if_some("open_files_limit", &self.open_files_limit())
                .display_if_some("kernel_version", &self.kernel_version)
                .display("kernel_long_version", &self.kernel_long_version)
                .display_if_some("version", &self.version)
                .display_if_some("long_version", &self.long_version)
                .display("distribution_id", &self.distribution_id)
                .display(
                    "distribution_id_like",
                    display_list(&self.distribution_id_like),
                )
                .display("users", display_map(&self.users()))
                .display("groups", display_map(&self.groups()))
                .finish(f)
        } else {
            DisplayFields::default()
                .display_if_some("name", &self.name)
                .display("uptime", self.uptime())
                .display("boot_time", self.boot_time())
                .display_if_some("kernel_version", &self.kernel_version)
                .display("kernel_long_version", &self.kernel_long_version)
                .display_if_some("version", &self.version)
                .display_if_some("long_version", &self.long_version)
                .display("distribution_id", &self.distribution_id)
                .display(
                    "distribution_id_like",
                    display_list(&self.distribution_id_like),
                )
                .finish(f)
        }
    }
}

impl Os {
    #[instrument(name = "os", skip_all)]
    pub async fn new(task_tracker: TaskTracker) -> Result<Self> {
        let (users, groups) = task_tracker
            .spawn_blocking(|| {
                (
                    sysinfo::Users::new_with_refreshed_list(),
                    sysinfo::Groups::new_with_refreshed_list(),
                )
            })
            .await?;

        Ok(Self {
            users: Arc::new(Mutex::new(users)),
            groups: Arc::new(Mutex::new(groups)),
            name: sysinfo::System::name().into(),
            kernel_version: sysinfo::System::kernel_version().into(),
            version: sysinfo::System::os_version().into(),
            long_version: sysinfo::System::long_os_version().into(),
            distribution_id: sysinfo::System::distribution_id(),
            distribution_id_like: sysinfo::System::distribution_id_like(),
            kernel_long_version: sysinfo::System::kernel_long_version(),
            task_tracker,
        })
    }

    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    #[must_use]
    pub fn kernel_version(&self) -> Option<&str> {
        self.kernel_version.as_deref()
    }

    #[must_use]
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    #[must_use]
    pub fn long_version(&self) -> Option<&str> {
        self.long_version.as_deref()
    }

    #[must_use]
    pub fn distribution_id(&self) -> &str {
        &self.distribution_id
    }

    #[must_use]
    pub const fn distribution_id_like(&self) -> &Vec<String> {
        &self.distribution_id_like
    }

    #[must_use]
    pub fn kernel_long_version(&self) -> &str {
        &self.kernel_long_version
    }

    #[must_use]
    pub fn uptime(&self) -> DurationUnit {
        DurationUnit::from_secs(sysinfo::System::uptime())
    }

    #[must_use]
    pub fn boot_time(&self) -> SystemTimeUnit {
        SystemTimeUnit::from_unix_epoch(sysinfo::System::boot_time())
    }

    #[must_use]
    pub fn open_files_limit(&self) -> Option<usize> {
        sysinfo::System::open_files_limit()
    }

    pub async fn refresh_users(&self) -> Result<HashMap<UidUnit, User>> {
        let users = self.users.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut users = users.lock().unwrap();
                users.refresh();

                users
                    .list()
                    .iter()
                    .map(|user| (user.id().clone().into(), user.into()))
                    .collect()
            })
            .await?;

        Ok(result)
    }

    #[must_use]
    pub fn users(&self) -> HashMap<UidUnit, User> {
        let users = self.users.lock().unwrap();
        users
            .list()
            .iter()
            .map(|user| (user.id().clone().into(), user.into()))
            .collect()
    }

    pub async fn refresh_groups(&self) -> Result<HashMap<u32, Group>> {
        let groups = self.groups.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut groups = groups.lock().unwrap();
                groups.refresh();

                groups
                    .list()
                    .iter()
                    .map(|group| (**group.id(), group.into()))
                    .collect()
            })
            .await?;

        Ok(result)
    }

    #[must_use]
    pub fn groups(&self) -> HashMap<u32, Group> {
        let groups = self.groups.lock().unwrap();
        groups
            .list()
            .iter()
            .map(|group| (**group.id(), group.into()))
            .collect()
    }
}
