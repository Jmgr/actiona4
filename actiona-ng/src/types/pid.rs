use std::num::{NonZeroI32, NonZeroU32};

use color_eyre::{
    Report, Result,
    eyre::{OptionExt, bail, eyre},
};
use derive_more::{Constructor, Display, From, Into};

/// Process ID
#[derive(Clone, Constructor, Copy, Debug, Display, Eq, From, Hash, Into, PartialEq)]
#[repr(transparent)]
pub struct Pid(NonZeroU32);

impl From<Pid> for u32 {
    fn from(value: Pid) -> Self {
        value.0.get()
    }
}

impl TryFrom<u32> for Pid {
    type Error = Report;

    fn try_from(value: u32) -> Result<Self> {
        Ok(Self(
            NonZeroU32::new(value).ok_or_eyre("process ID has to be non-zero")?,
        ))
    }
}

impl TryFrom<i32> for Pid {
    type Error = Report;

    fn try_from(value: i32) -> Result<Self> {
        if value < 0 {
            bail!("process ID cannot be negative: {value}");
        }

        let value = u32::try_from(value)?;
        value.try_into()
    }
}

impl TryFrom<NonZeroI32> for Pid {
    type Error = Report;

    fn try_from(value: NonZeroI32) -> Result<Self> {
        i32::from(value).try_into()
    }
}

impl TryFrom<sysinfo::Pid> for Pid {
    type Error = Report;

    fn try_from(value: sysinfo::Pid) -> Result<Self> {
        value.as_u32().try_into()
    }
}

impl From<Pid> for sysinfo::Pid {
    fn from(value: Pid) -> Self {
        Self::from_u32(value.0.into())
    }
}

impl TryFrom<Pid> for i32 {
    type Error = Report;

    fn try_from(value: Pid) -> Result<Self> {
        let value = u32::from(value.0);
        Ok(Self::try_from(value)?)
    }
}

#[cfg(unix)]
impl TryFrom<rustix::thread::Pid> for Pid {
    type Error = Report;

    fn try_from(value: rustix::thread::Pid) -> Result<Self> {
        value.as_raw_nonzero().try_into()
    }
}

#[cfg(unix)]
#[allow(unsafe_code)]
impl TryFrom<Pid> for rustix::thread::Pid {
    type Error = Report;

    fn try_from(value: Pid) -> Result<Self> {
        let value = i32::try_from(u32::from(value.0))
            .map_err(|_| eyre!("process ID does not fit in a i32: {value}"))?;
        // SAFETY: we know the process ID is non-zero because we store it as a NonZeroU32
        Ok(unsafe { Self::from_raw_unchecked(value) })
    }
}
