use std::{borrow::Cow, fmt::Debug, num::TryFromIntError, path::PathBuf, sync::Arc};

#[cfg(linux)]
use arboard::{ClearExtLinux, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use arboard::{Get, ImageData, Set};
use derive_more::Display;
use eyre::{Report, eyre};
use image::{DynamicImage, RgbaImage};
use itertools::Itertools;
use macros::{FromSerde, IntoSerde};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use thiserror::Error;

use crate::{core::image::Image, error::CommonError};

pub mod js;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    EyreReport(#[from] Report),

    #[error(transparent)]
    TryFromIntError(#[from] TryFromIntError),

    #[error(transparent)]
    CommonError(#[from] CommonError),

    #[error("content not available (incorrect format or empty clipboard)")]
    ContentNotAvailable,

    #[error("format conversion failure")]
    ConversionFailure,
}

impl From<arboard::Error> for Error {
    fn from(value: arboard::Error) -> Self {
        match value {
            arboard::Error::ContentNotAvailable => Self::ContentNotAvailable,
            arboard::Error::ClipboardNotSupported => {
                CommonError::UnsupportedPlatform("not supported on platform".to_string()).into()
            }
            arboard::Error::ConversionFailure => Self::ConversionFailure,
            arboard::Error::Unknown { description } => CommonError::Unknown(description).into(),
            _ => CommonError::Unexpected.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
pub enum ClipboardMode {
    #[default]
    Clipboard,

    /// @platforms =linux
    Selection,
}

pub enum ClipboardData {
    Text(String),
    Image(ImageData<'static>),
    Html(String),
    FileList(Vec<PathBuf>),
}

#[derive(Clone)]
pub struct Clipboard {
    inner: Arc<Mutex<arboard::Clipboard>>,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: Arc::new(Mutex::new(arboard::Clipboard::new()?)),
        })
    }

    fn set<F, R>(&'_ self, function: F, mode: Option<ClipboardMode>) -> R
    where
        F: FnOnce(Set<'_>) -> R,
    {
        let mode = mode.unwrap_or_default();
        let mut inner = self.inner.lock();

        #[cfg(linux)]
        if mode == ClipboardMode::Selection {
            function(inner.set().clipboard(LinuxClipboardKind::Primary))
        } else {
            function(inner.set())
        }

        #[cfg(not(linux))]
        {
            let _ = function;
            let _ = mode;
        }
    }

    fn get<F, R>(&'_ self, function: F, mode: Option<ClipboardMode>) -> R
    where
        F: FnOnce(Get<'_>) -> R,
    {
        let mode = mode.unwrap_or_default();
        let mut inner = self.inner.lock();

        #[cfg(linux)]
        if mode == ClipboardMode::Selection {
            function(inner.get().clipboard(LinuxClipboardKind::Primary))
        } else {
            function(inner.get())
        }

        #[cfg(not(linux))]
        {
            let _ = function;
            let _ = mode;
        }
    }

    pub fn save(&self, mode: Option<ClipboardMode>) -> Result<ClipboardData> {
        Ok(if let Ok(data) = self.get(|get| get.image(), mode) {
            ClipboardData::Image(data)
        } else if let Ok(data) = self.get(|get| get.file_list(), mode) {
            ClipboardData::FileList(data)
        } else if let Ok(data) = self.get(|get| get.html(), mode) {
            ClipboardData::Html(data)
        } else if let Ok(data) = self.get(|get| get.text(), mode) {
            ClipboardData::Text(data)
        } else {
            return Err(eyre!("unknown clipboard data").into());
        })
    }

    pub fn restore(&self, data: ClipboardData, mode: Option<ClipboardMode>) -> Result<()> {
        self.set(
            |set| match data {
                ClipboardData::Text(text) => set.text(text),
                ClipboardData::Image(image_data) => set.image(image_data),
                ClipboardData::Html(html) => set.html(html, None),
                ClipboardData::FileList(file_list) => set.file_list(&file_list),
            },
            mode,
        )?;
        Ok(())
    }

    pub fn set_text<'a, T: Into<Cow<'a, str>>>(
        &self,
        text: T,
        mode: Option<ClipboardMode>,
    ) -> Result<()> {
        let mode = mode.unwrap_or_default();
        let mut inner = self.inner.lock();

        let clipboard = {
            #[cfg(linux)]
            if mode == ClipboardMode::Selection {
                inner.set().clipboard(LinuxClipboardKind::Primary)
            } else {
                inner.set()
            }

            #[cfg(not(linux))]
            {
                let _ = mode;
                inner.set()
            }
        };

        clipboard.text(text)?;

        Ok(())
    }

    pub fn get_text(&self, mode: Option<ClipboardMode>) -> Result<String> {
        let text = self.get(|get| get.text(), mode)?;

        Ok(text)
    }

    pub fn set_image(&self, image: Image, mode: Option<ClipboardMode>) -> Result<()> {
        let image = image.to_rgba8().into_owned();
        let (width, height) = image.dimensions();
        let bytes = Cow::Owned(image.into_raw());

        self.set(
            |set| {
                set.image(ImageData {
                    width: width.try_into()?,
                    height: height.try_into()?,
                    bytes,
                })?;
                Result::Ok(())
            },
            mode,
        )?;

        Ok(())
    }

    pub fn get_image(&self, mode: Option<ClipboardMode>) -> Result<Image> {
        let image = self.get(|get| get.image(), mode)?;

        let img = RgbaImage::from_vec(
            image.width.try_into()?,
            image.height.try_into()?,
            image.bytes.to_vec(),
        )
        .unwrap();

        Ok(DynamicImage::ImageRgba8(img).into())
    }

    pub fn get_file_list(&self, mode: Option<ClipboardMode>) -> Result<Vec<String>> {
        let result = self.get(|get| get.file_list(), mode)?;

        let result = result
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect_vec();

        Ok(result)
    }

    pub fn set_html(
        &self,
        html: String,
        alt_text: Option<String>,
        mode: Option<ClipboardMode>,
    ) -> Result<()> {
        self.set(|set| set.html(html, alt_text), mode)?;

        Ok(())
    }

    pub fn get_html(&self, mode: Option<ClipboardMode>) -> Result<String> {
        let html = self.get(|get| get.html(), mode)?;

        Ok(html)
    }

    pub fn clear(&self, mode: Option<ClipboardMode>) -> Result<()> {
        let mode = mode.unwrap_or_default();
        let mut inner = self.inner.lock();

        #[cfg(linux)]
        if mode == ClipboardMode::Selection {
            inner.clear_with().clipboard(LinuxClipboardKind::Primary)?;
        } else {
            inner.clear()?;
        }

        #[cfg(not(linux))]
        {
            let _ = mode;
            inner.clear()?;
        }

        Ok(())
    }
}

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Clipboard").finish()
    }
}
