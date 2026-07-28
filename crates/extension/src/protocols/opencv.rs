use std::{
    fmt::{self, Display},
    ops::Deref,
    sync::atomic::{AtomicU64, Ordering},
};

use image::RgbaImage;
use macros::rpc_protocol;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use strum::EnumIs;
use types::{Point, Rect, Size, display::DisplayFields, size};
use uuid::Uuid;

/// Identifies one in-flight find request, so progress reports and cancellation
/// can be routed back to the caller that started it.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RequestId(u64);

impl Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// Hands out unique [`RequestId`]s.
///
/// Lives on the host, which is the only side that starts requests.
#[derive(Debug, Default)]
pub struct RequestIdProvider(AtomicU64);

impl RequestIdProvider {
    #[must_use]
    pub fn next_id(&self) -> RequestId {
        RequestId(self.0.fetch_add(1, Ordering::Relaxed))
    }
}

/// An [`RgbaImage`] that can cross the wire.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RgbaPixels(RgbaImage);

/// The wire shape of [`RgbaPixels`]. Borrowed on the way out, owned on the way
/// in, so serializing does not copy the pixel buffer.
#[derive(Serialize)]
#[serde(rename = "RgbaPixels")]
struct RgbaPixelsRef<'a> {
    width: u32,
    height: u32,
    pixels: &'a [u8],
}

#[derive(Deserialize)]
#[serde(rename = "RgbaPixels")]
struct RgbaPixelsOwned {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Serialize for RgbaPixels {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        RgbaPixelsRef {
            width: self.0.width(),
            height: self.0.height(),
            pixels: self.0.as_raw(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for RgbaPixels {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let RgbaPixelsOwned {
            width,
            height,
            pixels,
        } = RgbaPixelsOwned::deserialize(deserializer)?;

        RgbaImage::from_raw(width, height, pixels)
            .map(Self)
            .ok_or_else(|| {
                de::Error::custom(format!(
                    "RGBA8 buffer does not hold exactly {width}x{height} pixels"
                ))
            })
    }
}

impl Deref for RgbaPixels {
    type Target = RgbaImage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RgbaPixels {
    #[must_use]
    pub fn size(&self) -> Size {
        size(self.0.width(), self.0.height())
    }
}

impl From<RgbaImage> for RgbaPixels {
    fn from(image: RgbaImage) -> Self {
        Self(image)
    }
}

impl From<&RgbaImage> for RgbaPixels {
    fn from(image: &RgbaImage) -> Self {
        Self(image.clone())
    }
}

impl From<RgbaPixels> for RgbaImage {
    fn from(value: RgbaPixels) -> Self {
        value.0
    }
}

/// Identifies one prepared source image owned by the extension.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct SourceHandle(Uuid);

impl SourceHandle {
    #[must_use]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Identifies one prepared template image owned by the extension.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TemplateHandle(Uuid);

impl TemplateHandle {
    #[must_use]
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Everything the extension needs to take one screen capture on the host's
/// behalf.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CaptureSpec {
    /// The screen rectangle to capture, in screen coordinates.
    pub rect: Rect,

    /// Display rectangles to keep; everything in `rect` outside all of them is
    /// blackened. Empty means "keep everything".
    pub blacken_outside: Vec<Rect>,

    /// Whether the X11 MIT-SHM fast path may be used.
    pub use_shm: bool,
}

/// A match returned by a find operation.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Match {
    pub position: Point,
    pub rect: Rect,
    pub score: f64,
}

impl Display for Match {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DisplayFields::default()
            .display("position", self.position)
            .display("rect", self.rect)
            .display("score", self.score)
            .finish(f)
    }
}

impl Match {
    #[must_use]
    pub const fn new(position: Point, rect: Rect, score: f64) -> Self {
        Self {
            position,
            rect,
            score,
        }
    }

    /// Returns a new Match with position and rect offset by the given origin point.
    #[must_use]
    pub fn offset(self, origin: Point) -> Self {
        Self {
            position: self.position + origin,
            rect: Rect {
                top_left: self.rect.top_left + origin,
                ..self.rect
            },
            score: self.score,
        }
    }
}

/// The result of a find request.
///
/// [`Self::UnknownHandle`] is a typed stale-handle signal rather than an error,
/// so the host can tell "the extension restarted, re-upload and retry" apart
/// from a genuine failure without matching on error strings.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum FindOutcome {
    Matches(Vec<Match>),
    UnknownHandle,
}

#[derive(
    Clone, Copy, Debug, Default, Deserialize, strum::Display, EnumIs, Eq, PartialEq, Serialize,
)]
pub enum FindImageStep {
    Capturing,
    #[default]
    Preparing,
    Downscaling,
    Matching,
    Filtering,
    ComputingResults,
    Finished,
}

/// How far along a find request is.
///
/// `progress` is the completion ratio from 0 to 1 covering the whole request.
/// `step_progress` is the completion ratio from 0 to 1 for the step named by
/// `step`. A step that cannot measure itself reports a `step_progress` of 0
/// when it starts and 1 when it ends.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct FindImageProgress {
    pub step: FindImageStep,
    pub progress: f32,
    pub step_progress: f32,
}

impl Display for FindImageProgress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DisplayFields::default()
            .display("step", self.step)
            .display("progress", self.progress)
            .display("step_progress", self.step_progress)
            .finish(f)
    }
}

impl FindImageProgress {
    #[must_use]
    pub const fn new(step: FindImageStep, progress: f32, step_progress: f32) -> Self {
        Self {
            step,
            progress,
            step_progress,
        }
    }

    /// The report for entering `step`, before it has done anything.
    #[must_use]
    pub const fn started(step: FindImageStep, progress: f32) -> Self {
        Self::new(step, progress, 0.0)
    }
}

/// Find image template options.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct FindImageTemplateOptions {
    /// Use color matching.
    pub use_colors: bool,

    /// Use OpenCL/OpenCV's GPU-capable path for template matching.
    pub enable_gpu: bool,

    /// Use template transparency.
    pub use_transparency: bool,

    /// Matching threshold.
    /// Values are between 0 (worst) to 1 (best).
    pub match_threshold: f32,

    /// Radius to consider proximity (in pixels).
    pub non_maximum_suppression_radius: Option<i32>,

    /// How many times should the source image and the template be downscaled?
    pub downscale: u64,
}

impl Default for FindImageTemplateOptions {
    fn default() -> Self {
        Self {
            use_colors: true,
            enable_gpu: false,
            use_transparency: true,
            match_threshold: 0.8,
            non_maximum_suppression_radius: Some(10),
            downscale: 0,
        }
    }
}

#[rpc_protocol]
#[derive(Debug)]
pub trait OpenCVProtocol {
    /// Uploads an image and prepares it as a search source.
    #[host_call]
    async fn upload_source(image: RgbaPixels) -> SourceHandle;

    /// Uploads an image and prepares it as a search template, extracting the
    /// alpha channel as a transparency mask.
    #[host_call]
    async fn upload_template(image: RgbaPixels) -> TemplateHandle;

    /// Drops a prepared source. Unknown handles are ignored.
    #[host_call]
    async fn release_source(handle: SourceHandle);

    /// Drops a prepared template. Unknown handles are ignored.
    #[host_call]
    async fn release_template(handle: TemplateHandle);

    /// Searches for `template` inside a previously uploaded source.
    #[host_call]
    async fn find(
        request_id: RequestId,
        source: SourceHandle,
        template: TemplateHandle,
        options: FindImageTemplateOptions,
        search_one: bool,
    ) -> FindOutcome;

    /// Captures the area described by `capture` and searches it for `template`.
    ///
    /// Matches are returned in capture-local coordinates; the host offsets them
    /// by `capture.rect.top_left`.
    #[host_call]
    async fn find_on_screen(
        request_id: RequestId,
        capture: CaptureSpec,
        template: TemplateHandle,
        options: FindImageTemplateOptions,
        search_one: bool,
    ) -> FindOutcome;

    /// Cancels an in-flight find request. Unknown request ids are ignored.
    #[host_call]
    async fn cancel(request_id: RequestId);

    /// Reports that an in-flight find request has entered a new step.
    ///
    /// Step changes are answered, so the extension can tell when the host has
    /// them all and hold its result back until then.
    #[extension_call]
    async fn progress(request_id: RequestId, progress: FindImageProgress);

    /// Reports how far into its current step an in-flight find request is.
    ///
    /// Lossy on purpose: samples are coalesced and sent without a reply, so the
    /// host sees roughly the shape of a step rather than every value. Anything
    /// that must not be missed — the step changes, and the final 1 —
    /// belongs on [`Self::progress`] instead.
    #[extension_call(no_reply)]
    fn progress_sample(request_id: RequestId, progress: FindImageProgress);
}

#[cfg(test)]
mod tests {
    use image::{Rgba, RgbaImage};
    use serde::Serialize;

    use super::RgbaPixels;

    /// Mirrors the wire shape so a test can produce a message the real
    /// `Serialize` impl could never emit.
    #[derive(Serialize)]
    #[serde(rename = "RgbaPixels")]
    struct MalformedRgbaPixels {
        width: u32,
        height: u32,
        pixels: Vec<u8>,
    }

    fn sample_image() -> RgbaImage {
        RgbaImage::from_fn(2, 3, |x, y| {
            let x = u8::try_from(x).expect("test image is small");
            let y = u8::try_from(y).expect("test image is small");
            Rgba([x, y, x + y, 255])
        })
    }

    #[test]
    fn round_trips_through_serde() {
        let image = sample_image();
        let encoded = serde_json::to_string(&RgbaPixels::from(image.clone())).unwrap();
        let decoded: RgbaPixels = serde_json::from_str(&encoded).unwrap();

        assert_eq!(RgbaImage::from(decoded), image);
    }

    #[test]
    fn rejects_buffer_that_does_not_match_its_dimensions() {
        // 2x3 needs 24 bytes; claim it with 4.
        let encoded = serde_json::to_string(&MalformedRgbaPixels {
            width: 2,
            height: 3,
            pixels: vec![0; 4],
        })
        .unwrap();

        let error = serde_json::from_str::<RgbaPixels>(&encoded).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("does not hold exactly 2x3 pixels"),
            "unexpected error: {error}"
        );
    }
}
