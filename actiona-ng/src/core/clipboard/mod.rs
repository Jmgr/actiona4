use std::{borrow::Cow, fmt::Debug, num::TryFromIntError};

#[cfg(linux)]
use arboard::{ClearExtLinux, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use arboard::{Get, ImageData, Set};
use derive_more::Display;
use eyre::Report;
use image::{DynamicImage, RgbaImage};
use itertools::Itertools;
use macros::ExposeEnum;
use rquickjs::{JsLifetime, class::Trace};
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

#[derive(Clone, Copy, Debug, Default, Display, Eq, ExposeEnum, JsLifetime, PartialEq, Trace)]
#[rquickjs::class]
pub enum ClipboardMode {
    #[default]
    Clipboard,

    /// @platforms =linux
    Selection,
}

pub struct Clipboard {
    inner: arboard::Clipboard,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: arboard::Clipboard::new()?,
        })
    }

    fn set(&'_ mut self, mode: Option<ClipboardMode>) -> Set<'_> {
        let mode = mode.unwrap_or_default();
        let inner = &mut self.inner;

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
    }

    fn get(&'_ mut self, mode: Option<ClipboardMode>) -> Get<'_> {
        let mode = mode.unwrap_or_default();
        let inner = &mut self.inner;

        #[cfg(linux)]
        if mode == ClipboardMode::Selection {
            inner.get().clipboard(LinuxClipboardKind::Primary)
        } else {
            inner.get()
        }

        #[cfg(not(linux))]
        {
            let _ = mode;
            inner.get()
        }
    }

    pub async fn set_text<'a, T: Into<Cow<'a, str>>>(
        &mut self,
        text: T,
        mode: Option<ClipboardMode>,
    ) -> Result<()> {
        let mode = mode.unwrap_or_default();
        let inner = &mut self.inner;

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

    pub async fn get_text(&mut self, mode: Option<ClipboardMode>) -> Result<String> {
        let text = self.get(mode).text()?;

        Ok(text)
    }

    pub async fn set_image(&mut self, image: Image, mode: Option<ClipboardMode>) -> Result<()> {
        let image = image.to_rgba8().into_owned();
        let (width, height) = image.dimensions();
        let bytes = Cow::Owned(image.into_raw());

        self.set(mode).image(ImageData {
            width: width.try_into()?,
            height: height.try_into()?,
            bytes,
        })?;

        Ok(())
    }

    pub async fn get_image(&mut self, mode: Option<ClipboardMode>) -> Result<Image> {
        let image = self.get(mode).image()?;

        let img = RgbaImage::from_vec(
            image.width.try_into()?,
            image.height.try_into()?,
            image.bytes.to_vec(),
        )
        .unwrap();

        Ok(DynamicImage::ImageRgba8(img).into())
    }

    pub async fn get_file_list(&mut self, mode: Option<ClipboardMode>) -> Result<Vec<String>> {
        let result = self.get(mode).file_list()?;

        let result = result
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect_vec();

        Ok(result)
    }

    pub async fn set_html(
        &mut self,
        html: String,
        alt_text: Option<String>,
        mode: Option<ClipboardMode>,
    ) -> Result<()> {
        self.set(mode).html(html, alt_text)?;

        Ok(())
    }

    pub async fn get_html(&mut self, mode: Option<ClipboardMode>) -> Result<String> {
        let html = self.get(mode).html()?;

        Ok(html)
    }

    pub async fn clear(&mut self, mode: Option<ClipboardMode>) -> Result<()> {
        let mode = mode.unwrap_or_default();
        let inner = &mut self.inner;

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
