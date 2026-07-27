#[cfg(unix)]
use std::sync::Arc;

use color_eyre::{Result, eyre::eyre};
use extension::protocols::opencv::CaptureSpec;
#[cfg(unix)]
use parking_lot::Mutex;
use screenshot::{Capture, blacken_non_display_areas};
use tokio::sync::OnceCell;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

/// Takes screen captures on the host's behalf.
///
/// Screen matching feeds raw pixels straight into OpenCV, so capturing here
/// rather than in the host keeps full-desktop screenshots (tens of megabytes)
/// off the IPC socket entirely. The host still owns display enumeration and
/// window lookups and sends down plain geometry in [`CaptureSpec`].
#[derive(Debug)]
pub struct Capturer {
    task_tracker: TaskTracker,
    cancellation_token: CancellationToken,
    /// Created on first use: an extension that never runs a screen search
    /// should not open an X11 connection.
    screen: OnceCell<screenshot::Screen>,
    #[cfg(unix)]
    shm: Mutex<Option<Arc<screenshot::ShmSegment>>>,
}

impl Capturer {
    #[must_use]
    pub fn new(task_tracker: TaskTracker, cancellation_token: CancellationToken) -> Self {
        Self {
            task_tracker,
            cancellation_token,
            screen: OnceCell::new(),
            #[cfg(unix)]
            shm: Mutex::new(None),
        }
    }

    async fn screen(&self) -> Result<&screenshot::Screen> {
        self.screen
            .get_or_try_init(|| {
                screenshot::Screen::new(self.task_tracker.clone(), self.cancellation_token.clone())
            })
            .await
    }

    /// Captures the area described by `spec`, blackening anything outside the
    /// listed display rectangles.
    pub async fn capture(
        &self,
        spec: &CaptureSpec,
        cancellation_token: &CancellationToken,
    ) -> Result<Capture> {
        let screen = tokio::select! {
            screen = self.screen() => screen?,
            () = cancellation_token.cancelled() => return Err(eyre!("screen capture cancelled")),
        };

        let mut capture = tokio::select! {
            capture = self.capture_raw(screen, spec) => capture?,
            () = cancellation_token.cancelled() => return Err(eyre!("screen capture cancelled")),
        };

        if cancellation_token.is_cancelled() {
            return Err(eyre!("screen capture cancelled"));
        }

        if !spec.blacken_outside.is_empty() {
            // Safe on BGRA as well as RGBA: blackened pixels are zeroed.
            blacken_non_display_areas(&mut capture.bgra, spec.rect, &spec.blacken_outside);
        }

        Ok(capture)
    }

    #[cfg(unix)]
    async fn capture_raw(
        &self,
        screen: &screenshot::Screen,
        spec: &CaptureSpec,
    ) -> Result<Capture> {
        use screenshot::ShmSegment;
        use tracing::warn;

        if !spec.use_shm {
            return screen.capture_rect(spec.rect).await;
        }

        let needed = ShmSegment::capacity_for_rect(spec.rect);
        let existing = self
            .shm
            .lock()
            .as_ref()
            .filter(|segment| segment.capacity() >= needed)
            .map(Arc::clone);

        let segment = match existing {
            Some(segment) => segment,
            None => match ShmSegment::new(screen, needed).await {
                Ok(segment) => {
                    let segment = Arc::new(segment);
                    *self.shm.lock() = Some(Arc::clone(&segment));
                    segment
                }
                Err(error) => {
                    warn!("failed to allocate SHM segment, falling back to XGetImage: {error}");
                    return screen.capture_rect(spec.rect).await;
                }
            },
        };

        segment.capture_rect(screen, spec.rect).await
    }

    #[cfg(windows)]
    async fn capture_raw(
        &self,
        screen: &screenshot::Screen,
        spec: &CaptureSpec,
    ) -> Result<Capture> {
        screen.capture_rect(spec.rect).await
    }
}
