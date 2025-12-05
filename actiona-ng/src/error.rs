use std::{num::TryFromIntError, time::SystemTimeError};

use strum::EnumIs;
use thiserror::Error;

use crate::{IntoJSError, core::clipboard};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Clipboard error: {0}")]
    ClipboardError(clipboard::Error),

    #[error(transparent)]
    CommonError(CommonError),
}

impl Error {
    pub fn is_cancelled(&self) -> bool {
        if let Self::CommonError(err) = self {
            return err.is_cancelled();
        }

        false
    }
}

impl IntoJSError for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, EnumIs)]
pub enum CommonError {
    #[error("Unsupported on this platform: {0}")]
    UnsupportedPlatform(String),

    #[error("Unsupported: {0}")]
    Unsupported(String),

    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error("Unexpected error")]
    Unexpected,

    #[error("Cancelled")]
    Cancelled,

    #[error("ArrayBuffer is detached")]
    DetachedArrayBuffer,
}

impl IntoJSError for CommonError {}

impl IntoJSError for TryFromIntError {}

impl IntoJSError for SystemTimeError {}
