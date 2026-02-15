use std::{num::TryFromIntError, time::SystemTimeError};

use strum::EnumIs;
use thiserror::Error;

use crate::IntoJSError;

#[derive(Debug, EnumIs, Error)]
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
