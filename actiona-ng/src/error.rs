use thiserror::Error;

use crate::{IntoJSError, core::clipboard};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Clipboard error: {0}")]
    ClipboardError(clipboard::Error),

    #[error(transparent)]
    CommonError(CommonError),
}

impl IntoJSError for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
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
