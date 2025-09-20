use core::fmt::Display;
use std::{ffi::OsStr, marker::PhantomData, ops::Deref};

use humansize::BINARY;

#[repr(transparent)]
#[derive(Debug, Default)]
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
    pub fn none() -> Self {
        Self(None)
    }
}

#[derive(Debug)]
pub struct DegreesCelsiusTag;
pub type DegreesCelsius = Unit<f64, DegreesCelsiusTag>;

impl Display for DegreesCelsius {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} °C", self.0)
    }
}

impl From<f32> for DegreesCelsius {
    fn from(value: f32) -> Self {
        DegreesCelsius::from(value as f64)
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
        ByteCount::from(value as u64)
    }
}

pub type OptionalByteCount = OptionalUnit<ByteCount>;

impl From<Option<u32>> for OptionalByteCount {
    fn from(value: Option<u32>) -> Self {
        Self(value.map(|v| v.into()))
    }
}

pub type OptionalString = OptionalUnit<String>;

impl From<Option<&str>> for OptionalString {
    fn from(value: Option<&str>) -> Self {
        Self(match value.map(|s| s.trim()) {
            Some("Default string")
            | Some("To be filled by O.E.M.")
            | Some("System Product Name")
            | Some("Not Specified") => None,
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s.to_string()),
            None => None,
        })
    }
}

impl From<Option<String>> for OptionalString {
    fn from(value: Option<String>) -> Self {
        Self::from(value.as_deref())
    }
}

impl From<&OsStr> for OptionalString {
    fn from(value: &OsStr) -> Self {
        Self::from(value.to_str())
    }
}
