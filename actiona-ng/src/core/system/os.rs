use std::{
    collections::HashMap,
    fmt::Display,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use itertools::Itertools;

use crate::types::OptionalString;

#[derive(Debug)]
pub struct Group {
    name: String,
}

impl Display for Group {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
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

#[derive(Debug)]
pub struct User {
    name: String,
    group_id: u32,
    groups: Vec<u32>,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(name: {}, group_id: {}, groups: [{}])",
            self.name,
            self.group_id,
            self.groups.iter().join(", ")
        )
    }
}

impl From<&sysinfo::User> for User {
    fn from(value: &sysinfo::User) -> Self {
        Self {
            name: value.name().to_string(),
            group_id: *value.group_id(),
            groups: value
                .groups()
                .into_iter()
                .map(|group| **group.id())
                .collect_vec(),
        }
    }
}

#[derive_where::derive_where(Debug)]
pub struct Os {
    #[derive_where(skip)]
    users: Arc<Mutex<sysinfo::Users>>,

    #[derive_where(skip)]
    groups: Arc<Mutex<sysinfo::Groups>>,

    name: OptionalString,
    kernel_version: OptionalString,
    version: OptionalString,
    long_version: OptionalString,
    distribution_id: String,
    distribution_id_like: Vec<String>,
    kernel_long_version: String,
}

impl Default for Os {
    fn default() -> Self {
        Self {
            users: Arc::new(Mutex::new(sysinfo::Users::new())),
            groups: Arc::new(Mutex::new(sysinfo::Groups::new())),
            name: sysinfo::System::name().into(),
            kernel_version: sysinfo::System::kernel_version().into(),
            version: sysinfo::System::os_version().into(),
            long_version: sysinfo::System::long_os_version().into(),
            distribution_id: sysinfo::System::distribution_id(),
            distribution_id_like: sysinfo::System::distribution_id_like(),
            kernel_long_version: sysinfo::System::kernel_long_version(),
        }
    }
}

impl Os {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn kernel_version(&self) -> Option<&str> {
        self.kernel_version.as_deref()
    }

    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    pub fn long_version(&self) -> Option<&str> {
        self.long_version.as_deref()
    }
    pub fn distribution_id(&self) -> &str {
        &self.distribution_id
    }

    pub fn distribution_id_like(&self) -> &Vec<String> {
        &self.distribution_id_like
    }

    pub fn kernel_long_version(&self) -> &str {
        &self.kernel_long_version
    }

    pub fn uptime(&self) -> Duration {
        Duration::from_secs(sysinfo::System::uptime())
    }

    pub fn boot_time(&self) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(sysinfo::System::boot_time())
    }

    pub fn open_files_limit(&self) -> Option<usize> {
        sysinfo::System::open_files_limit()
    }

    pub fn users(&self) -> HashMap<u32, User> {
        let mut users = self.users.lock().unwrap();
        users.refresh();

        users
            .list()
            .iter()
            .map(|user| (**user.id(), user.into()))
            .collect()
    }

    pub fn groups(&self) -> HashMap<u32, Group> {
        let mut groups = self.groups.lock().unwrap();
        groups.refresh();

        groups
            .list()
            .iter()
            .map(|group| (**group.id(), group.into()))
            .collect()
    }
}
