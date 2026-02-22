use std::{
    borrow::Cow,
    fmt::Debug,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

#[cfg(linux)]
use arboard::{ClearExtLinux, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use arboard::{Get, ImageData, Set};
use color_eyre::{Result, eyre::eyre};
use derive_more::Display;
use image::RgbaImage;
use itertools::Itertools;
use macros::{FromSerde, IntoSerde};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use strum::EnumIter;
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;
use tracing::instrument;
#[cfg(windows)]
use windows::Win32::System::DataExchange::GetClipboardSequenceNumber;

use crate::{api::image::Image, error::CommonError};

pub mod js;

/// Sentinel error for "content not available" — used for `downcast_ref` matching.
#[derive(Debug, thiserror::Error)]
#[error("content not available (incorrect format or empty clipboard)")]
pub struct ContentNotAvailable;

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
/// @expand
pub enum ClipboardMode {
    #[default]
    /// `ClipboardMode.Clipboard`
    Clipboard,

    /// @platforms =linux
    /// `ClipboardMode.Selection`
    Selection,
}

pub enum ClipboardData {
    Text(String),
    Image(ImageData<'static>),
    Html(String),
    FileList(Vec<PathBuf>),
}

#[derive(Clone, Copy, Debug)]
pub struct WaitForChangedOptions {
    pub mode: Option<ClipboardMode>,
    pub interval: Duration,
}

impl Default for WaitForChangedOptions {
    fn default() -> Self {
        Self {
            mode: None,
            interval: Duration::from_millis(200),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ClipboardSnapshot {
    Empty,
    Hash(u64),
}

#[derive(Clone)]
pub struct Clipboard {
    inner: Arc<Mutex<arboard::Clipboard>>,
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn clipboard_sequence_number() -> Option<u32> {
    let sequence_number = unsafe { GetClipboardSequenceNumber() };
    if sequence_number == 0 {
        None
    } else {
        Some(sequence_number)
    }
}

impl Clipboard {
    #[instrument(skip_all)]
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
            _ = mode;

            function(inner.set())
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
            _ = mode;

            function(inner.get())
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
            return Err(ContentNotAvailable.into());
        })
    }

    fn snapshot(&self, mode: Option<ClipboardMode>) -> Result<ClipboardSnapshot> {
        match self.save(mode) {
            Ok(data) => {
                let mut hasher = DefaultHasher::new();

                match data {
                    ClipboardData::Text(text) => {
                        0u8.hash(&mut hasher);
                        text.hash(&mut hasher);
                    }
                    ClipboardData::Image(image) => {
                        1u8.hash(&mut hasher);
                        image.width.hash(&mut hasher);
                        image.height.hash(&mut hasher);
                        image.bytes.hash(&mut hasher);
                    }
                    ClipboardData::Html(html) => {
                        2u8.hash(&mut hasher);
                        html.hash(&mut hasher);
                    }
                    ClipboardData::FileList(files) => {
                        3u8.hash(&mut hasher);
                        files.hash(&mut hasher);
                    }
                }

                Ok(ClipboardSnapshot::Hash(hasher.finish()))
            }
            Err(err) if err.downcast_ref::<ContentNotAvailable>().is_some() => {
                Ok(ClipboardSnapshot::Empty)
            }
            Err(err) => Err(err),
        }
    }

    pub async fn wait_for_changed(
        &self,
        options: WaitForChangedOptions,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        if options.interval.is_zero() {
            return Err(eyre!("unsupported: interval cannot be zero"));
        }

        #[cfg(windows)]
        if let Some(initial_sequence_number) = clipboard_sequence_number() {
            loop {
                select! {
                    _ = cancellation_token.cancelled() => {
                        return Err(CommonError::Cancelled.into());
                    }
                    _ = sleep(options.interval) => {}
                }

                match clipboard_sequence_number() {
                    Some(current_sequence_number) => {
                        if current_sequence_number != initial_sequence_number {
                            return Ok(());
                        }
                    }
                    // Sequence number is temporarily unavailable: fall back to content snapshots.
                    None => break,
                }
            }
        }

        let initial_snapshot = self.snapshot(options.mode)?;

        loop {
            select! {
                _ = cancellation_token.cancelled() => {
                    return Err(CommonError::Cancelled.into());
                }
                _ = sleep(options.interval) => {}
            }

            let current_snapshot = self.snapshot(options.mode)?;
            if current_snapshot != initial_snapshot {
                return Ok(());
            }
        }
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
        self.set(|set| set.text(text), mode)?;

        Ok(())
    }

    pub fn get_text(&self, mode: Option<ClipboardMode>) -> Result<String> {
        let text = self.get(|get| get.text(), mode)?;

        Ok(text)
    }

    pub fn set_image(&self, image: Image, mode: Option<ClipboardMode>) -> Result<()> {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let width: usize = width.try_into()?;
        let height: usize = height.try_into()?;
        let bytes = Cow::Owned(image.into_raw());

        self.set(
            |set| {
                set.image(ImageData {
                    width,
                    height,
                    bytes,
                })
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
        .ok_or_else(|| eyre!("format conversion failure"))?;

        Ok(Image::from_rgba8(img))
    }

    pub fn set_file_list(
        &self,
        file_list: &[impl AsRef<Path>],
        mode: Option<ClipboardMode>,
    ) -> Result<()> {
        self.set(|set| set.file_list(file_list), mode)?;

        Ok(())
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
            _ = mode;
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
