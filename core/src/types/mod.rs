use core::fmt::Display;
use std::{
    collections::HashSet,
    ffi::{OsStr, OsString},
    marker::PhantomData,
    ops::Deref,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use humansize::BINARY;
use itertools::Itertools;
use sysinfo::Uid;

use crate::{api::system::processes::ThreadKind, types::pid::Pid};

pub mod convert;
pub mod display;
pub mod input;
pub mod ops;
pub mod pid;
pub mod si32;
pub mod su32;
pub mod try_traits;

#[repr(transparent)]
#[derive(Debug, Default, Eq, Hash, PartialEq)]
pub struct Unit<T, Tag>(T, PhantomData<Tag>);

impl<T: Clone, Tag> Clone for Unit<T, Tag> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}
impl<T: Copy, Tag> Copy for Unit<T, Tag> {}

impl<T, Tag> From<T> for Unit<T, Tag> {
    fn from(value: T) -> Self {
        Self(value, PhantomData)
    }
}
impl<T, Tag> Deref for Unit<T, Tag> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

#[repr(transparent)]
#[derive(Debug, Default)]
pub struct OptionalUnit<T>(Option<T>);

impl<T: Clone> Clone for OptionalUnit<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<T: Copy> Copy for OptionalUnit<T> {}

impl<T: Clone + Display> Display for OptionalUnit<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Some(v) => write!(f, "{}", v),
            None => write!(f, "<NONE>"),
        }
    }
}

impl<T> Deref for OptionalUnit<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> OptionalUnit<T> {
    pub const fn none() -> Self {
        Self(None)
    }
}

#[derive(Debug)]
pub struct DegreesCelsiusTag;
pub type DegreesCelsius = Unit<f64, DegreesCelsiusTag>;

impl Display for DegreesCelsius {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.0}°C", self.0)
    }
}

impl From<f32> for DegreesCelsius {
    fn from(value: f32) -> Self {
        Self::from(f64::from(value))
    }
}

pub type OptionalDegreesCelsius = OptionalUnit<DegreesCelsius>;

impl From<Option<f32>> for OptionalDegreesCelsius {
    fn from(value: Option<f32>) -> Self {
        Self(value.map(|v| v.into()))
    }
}

#[derive(Debug)]
pub struct ByteCountTag;
pub type ByteCount = Unit<u64, ByteCountTag>;

impl Display for ByteCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", humansize::format_size(self.0, BINARY))
    }
}

impl From<u32> for ByteCount {
    fn from(value: u32) -> Self {
        Self::from(u64::from(value))
    }
}

pub type OptionalByteCount = OptionalUnit<ByteCount>;

impl From<Option<u32>> for OptionalByteCount {
    fn from(value: Option<u32>) -> Self {
        Self(value.map(|v| v.into()))
    }
}

pub type OptionalSystemString = OptionalUnit<String>;

impl From<Option<&str>> for OptionalSystemString {
    fn from(value: Option<&str>) -> Self {
        Self(match value.map(|s| s.trim()) {
            Some("Default string")
            | Some("To be filled by O.E.M.")
            | Some("System Product Name")
            | Some("Not Specified") => None,
            None | Some("") => None,
            Some(s) => Some(s.to_string()),
        })
    }
}

impl From<Option<String>> for OptionalSystemString {
    fn from(value: Option<String>) -> Self {
        Self::from(value.as_deref())
    }
}

impl From<&OsStr> for OptionalSystemString {
    fn from(value: &OsStr) -> Self {
        Self(Some(value.to_string_lossy().to_string()))
    }
}

#[derive(Debug)]
pub struct PercentTag;
pub type Percent = Unit<f32, PercentTag>;

impl Display for Percent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.0)
    }
}

pub type OptionalPercent = OptionalUnit<Percent>;

#[derive(Debug)]
pub struct FrequencyTag;
pub type Frequency = Unit<u64, FrequencyTag>;

impl Display for Frequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} Hz", self.0)
    }
}

pub type OptionalFrequency = OptionalUnit<Frequency>;

#[derive(Debug)]
pub struct OsStringListTag;
pub type OsStringList = Unit<Vec<String>, OsStringListTag>;

impl Display for OsStringList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.iter().join(", "))
    }
}

impl From<&[OsString]> for OsStringList {
    fn from(value: &[OsString]) -> Self {
        value
            .iter()
            .map(|cmd| cmd.to_string_lossy().into_owned())
            .collect_vec()
            .into()
    }
}

#[derive(Debug)]
pub struct PathTag;
pub type Path = Unit<PathBuf, PathTag>;

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl From<&std::path::Path> for Path {
    fn from(value: &std::path::Path) -> Self {
        value.to_path_buf().into()
    }
}

pub type OptionalPath = OptionalUnit<Path>;

impl From<Option<&std::path::Path>> for OptionalPath {
    fn from(value: Option<&std::path::Path>) -> Self {
        Self(value.map(|path| path.into()))
    }
}

pub type OptionalU32 = OptionalUnit<u32>;

impl From<Option<u32>> for OptionalU32 {
    fn from(value: Option<u32>) -> Self {
        Self(value)
    }
}

pub type OptionalPid = OptionalUnit<Pid>;

impl From<Option<Pid>> for OptionalPid {
    fn from(value: Option<Pid>) -> Self {
        Self(value)
    }
}

pub type OptionalUSize = OptionalUnit<usize>;

impl From<Option<usize>> for OptionalUSize {
    fn from(value: Option<usize>) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct TaskListTag;
pub type TaskList = Unit<HashSet<u32>, TaskListTag>;

impl Display for TaskList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.iter().join(", "))
    }
}

pub type OptionalTaskList = OptionalUnit<TaskList>;

impl From<Option<HashSet<u32>>> for OptionalTaskList {
    fn from(value: Option<HashSet<u32>>) -> Self {
        Self(value.map(|path| path.into()))
    }
}

pub type OptionalThreadKind = OptionalUnit<ThreadKind>;

impl From<Option<sysinfo::ThreadKind>> for OptionalThreadKind {
    // TODO: automate this?
    fn from(value: Option<sysinfo::ThreadKind>) -> Self {
        Self(value.map(|path| path.into()))
    }
}

#[derive(Debug)]
pub struct SystemTimeUnitTag;
pub type SystemTimeUnit = Unit<SystemTime, SystemTimeUnitTag>;

impl SystemTimeUnit {
    #[must_use]
    pub fn from_unix_epoch(secs: u64) -> Self {
        (UNIX_EPOCH + Duration::from_secs(secs)).into()
    }
}

impl Display for SystemTimeUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", humantime::format_rfc3339(self.0))
    }
}

#[derive(Debug)]
pub struct DurationUnitTag;
pub type DurationUnit = Unit<Duration, DurationUnitTag>;

impl Display for DurationUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", humantime::format_duration(self.0))
    }
}

impl DurationUnit {
    #[must_use]
    pub fn from_secs(secs: u64) -> Self {
        Duration::from_secs(secs).into()
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct UidTag;
pub type UidUnit = Unit<Uid, UidTag>;

impl Display for UidUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0) // TODO
    }
}

pub type OptionalUidUnit = OptionalUnit<UidUnit>;

impl From<Option<Uid>> for OptionalUidUnit {
    // TODO: automate this?
    fn from(value: Option<Uid>) -> Self {
        Self(value.map(|uid| uid.into()))
    }
}
