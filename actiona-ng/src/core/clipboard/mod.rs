use std::fmt::Debug;

use thiserror::Error;

use crate::error::CommonError;

pub mod js;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    CommonError(#[from] CommonError),

    #[error("other error")]
    OtherError,
}

impl From<arboard::Error> for Error {
    fn from(value: arboard::Error) -> Self {
        match value {
            arboard::Error::ContentNotAvailable => todo!(),
            arboard::Error::ClipboardNotSupported => todo!(),
            arboard::Error::ClipboardOccupied => todo!(),
            arboard::Error::ConversionFailure => todo!(),
            arboard::Error::Unknown { description } => CommonError::Unknown(description).into(),
            _ => CommonError::Unexpected.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Clipboard {
    inner: arboard::Clipboard,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: arboard::Clipboard::new()?,
        })
    }
}

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Clipboard").finish()
    }
}
