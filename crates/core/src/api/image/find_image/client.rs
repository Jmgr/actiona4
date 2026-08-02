use std::{collections::HashMap, sync::Arc};

use color_eyre::{Result, eyre::eyre};
use extension::{
    Host,
    protocols::opencv::{
        CaptureSpec, FindImageProgress, FindImageTemplateOptions, FindOutcome, Match,
        OpenCVProtocol, OpenCVProtocolHost, RequestId, RequestIdProvider, RgbaPixels, SourceHandle,
        TemplateHandle,
    },
};
use parking_lot::Mutex;
use tokio::sync::mpsc;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, warn};

use crate::{api::image::Image, error::CommonError};

const UNKNOWN_HANDLE_AFTER_RETRY: &str =
    "the image matching extension keeps losing its prepared images";

/// Which prepared image a queued release refers to.
#[derive(Clone, Copy, Debug)]
enum Release {
    Source(SourceHandle),
    Template(TemplateHandle),
}

/// One in-flight request's progress stream.
#[derive(Debug)]
struct ProgressStream {
    sender: mpsc::UnboundedSender<FindImageProgress>,
    /// The furthest this request has been reported to have got.
    reached: f32,
}

/// Progress senders for in-flight requests, keyed by request id.
///
/// Shared between the client (which registers a sender per request) and the
/// host-side protocol handler (which the extension calls into).
#[derive(Debug, Default)]
pub struct ProgressRegistry(Mutex<HashMap<RequestId, ProgressStream>>);

impl ProgressRegistry {
    fn insert(&self, request_id: RequestId, sender: mpsc::UnboundedSender<FindImageProgress>) {
        self.0.lock().insert(
            request_id,
            ProgressStream {
                sender,
                reached: 0.0,
            },
        );
    }

    fn remove(&self, request_id: RequestId) {
        self.0.lock().remove(&request_id);
    }

    /// Passes on a step change, which is always the newest word on a request.
    fn dispatch(&self, request_id: RequestId, progress: FindImageProgress) {
        let mut streams = self.0.lock();
        let Some(stream) = streams.get_mut(&request_id) else {
            return;
        };

        stream.reached = stream.reached.max(progress.progress);
        let _ = stream.sender.send(progress);
    }

    /// Passes on a sample, unless the request has already been reported
    /// further along than this.
    ///
    /// Samples travel without a reply and are handled in tasks of their own, so
    /// one can arrive after a step change that was sent later. Dropping those
    /// is what keeps a consumer from seeing a search go backwards — or from
    /// seeing anything at all after the final step.
    fn dispatch_sample(&self, request_id: RequestId, progress: FindImageProgress) {
        let mut streams = self.0.lock();
        let Some(stream) = streams.get_mut(&request_id) else {
            return;
        };

        if progress.progress <= stream.reached {
            return;
        }

        stream.reached = progress.progress;
        let _ = stream.sender.send(progress);
    }
}

/// Host side of the protocol: receives progress reports from the extension.
#[derive(Debug)]
pub struct ProgressHandler {
    registry: Arc<ProgressRegistry>,
}

impl ProgressHandler {
    #[must_use]
    pub const fn new(registry: Arc<ProgressRegistry>) -> Self {
        Self { registry }
    }
}

impl OpenCVProtocolHost for ProgressHandler {
    async fn progress(&self, request_id: RequestId, progress: FindImageProgress) -> Result<()> {
        self.registry.dispatch(request_id, progress);
        Ok(())
    }

    async fn progress_sample(
        &self,
        request_id: RequestId,
        progress: FindImageProgress,
    ) -> Result<()> {
        self.registry.dispatch_sample(request_id, progress);
        Ok(())
    }
}

#[cfg(test)]
mod progress_tests {
    use extension::protocols::opencv::FindImageStep;
    use tokio::sync::mpsc;

    use super::{FindImageProgress, ProgressRegistry, RequestIdProvider};

    #[test]
    fn drops_sample_at_the_next_step_boundary() {
        let registry = ProgressRegistry::default();
        let request_id = RequestIdProvider::default().next_id();
        let (sender, mut receiver) = mpsc::unbounded_channel();
        registry.insert(request_id, sender);

        registry.dispatch(
            request_id,
            FindImageProgress::started(FindImageStep::Matching, 0.2),
        );
        registry.dispatch(
            request_id,
            FindImageProgress::started(FindImageStep::Filtering, 0.7),
        );
        registry.dispatch_sample(
            request_id,
            FindImageProgress::new(FindImageStep::Matching, 0.7, 1.0),
        );

        assert_eq!(receiver.try_recv().unwrap().step, FindImageStep::Matching);
        assert_eq!(receiver.try_recv().unwrap().step, FindImageStep::Filtering);
        assert!(receiver.try_recv().is_err());
    }
}

/// Unregisters a request's progress sender when the request finishes.
struct ProgressGuard {
    registry: Arc<ProgressRegistry>,
    request_id: RequestId,
}

impl Drop for ProgressGuard {
    fn drop(&mut self) {
        self.registry.remove(self.request_id);
    }
}

/// A source image prepared inside the extension.
///
/// Dropping this queues a release so the extension can free the Mats it holds.
#[derive(Debug)]
pub struct RemoteSource {
    handle: SourceHandle,
    release: Option<mpsc::UnboundedSender<Release>>,
}

impl Drop for RemoteSource {
    fn drop(&mut self) {
        if let Some(release) = &self.release {
            let _ = release.send(Release::Source(self.handle));
        }
    }
}

/// A template image prepared inside the extension.
#[derive(Debug)]
pub struct RemoteTemplate {
    handle: TemplateHandle,
    release: Option<mpsc::UnboundedSender<Release>>,
}

impl Drop for RemoteTemplate {
    fn drop(&mut self) {
        if let Some(release) = &self.release {
            let _ = release.send(Release::Template(self.handle));
        }
    }
}

#[cfg(test)]
impl RemoteSource {
    /// A handle with no extension behind it, for cache-invalidation tests.
    pub(crate) fn detached() -> Self {
        Self {
            handle: SourceHandle::generate(),
            release: None,
        }
    }
}

#[cfg(test)]
impl RemoteTemplate {
    /// A handle with no extension behind it, for cache-invalidation tests.
    pub(crate) fn detached() -> Self {
        Self {
            handle: TemplateHandle::generate(),
            release: None,
        }
    }
}

/// Client for the out-of-process OpenCV extension.
///
/// Prepared images live in the extension and are addressed by handle, cached on
/// the `Image` they came from so repeated searches don't re-upload or redo the
/// Lab conversion.
#[derive(Debug)]
pub struct OpenCVClient {
    host: Arc<Host<OpenCVProtocol>>,
    progress: Arc<ProgressRegistry>,
    release: mpsc::UnboundedSender<Release>,
    next_request_id: RequestIdProvider,
}

impl OpenCVClient {
    #[must_use]
    pub fn new(
        host: Arc<Host<OpenCVProtocol>>,
        progress: Arc<ProgressRegistry>,
        task_tracker: &TaskTracker,
        cancellation_token: &CancellationToken,
    ) -> Arc<Self> {
        let (release, mut releases) = mpsc::unbounded_channel();

        let client = Arc::new(Self {
            host: Arc::clone(&host),
            progress,
            release,
            next_request_id: RequestIdProvider::default(),
        });

        // `Drop` cannot await, so handle releases are queued and drained here.
        // The client itself keeps a sender alive while it is stored in
        // `Extensions`, so cancellation must also stop this task at shutdown.
        let cancellation_token = cancellation_token.clone();
        task_tracker.spawn(async move {
            while let Some(release) = next_release(&mut releases, &cancellation_token).await {
                let result = match release {
                    Release::Source(handle) => host.release_source(handle).await,
                    Release::Template(handle) => host.release_template(handle).await,
                };

                if let Err(error) = result {
                    // Not worth surfacing: a dead extension has already dropped
                    // everything we were trying to release.
                    warn!("failed to release image matching handle: {error}");
                }
            }
        });

        client
    }

    fn begin(
        &self,
        progress: mpsc::UnboundedSender<FindImageProgress>,
    ) -> (RequestId, ProgressGuard) {
        let request_id = self.next_request_id.next_id();
        self.progress.insert(request_id, progress);

        (
            request_id,
            ProgressGuard {
                registry: self.progress.clone(),
                request_id,
            },
        )
    }

    /// Races a request against cancellation, telling the extension to stop if
    /// the caller gives up first.
    async fn cancellable<T>(
        &self,
        request_id: RequestId,
        token: &CancellationToken,
        request: impl Future<Output = Result<T>>,
    ) -> Result<T> {
        tokio::select! {
            () = token.cancelled() => {
                let _ = self.host.cancel(request_id).await;
                Err(CommonError::Cancelled.into())
            }
            result = request => result,
        }
    }

    async fn source_handle(&self, image: &Image) -> Result<Arc<RemoteSource>> {
        if let Some(cached) = image.source.get() {
            return Ok(cached);
        }

        info!(
            width = image.width(),
            height = image.height(),
            "uploading OpenCV source"
        );
        let handle = self
            .host
            .upload_source(RgbaPixels::from(image.as_rgba8()))
            .await?;
        info!(?handle, "uploaded OpenCV source");
        let remote = Arc::new(RemoteSource {
            handle,
            release: Some(self.release.clone()),
        });
        image.source.set(Arc::clone(&remote));

        Ok(remote)
    }

    async fn template_handle(&self, image: &Image) -> Result<Arc<RemoteTemplate>> {
        if let Some(cached) = image.template.get() {
            return Ok(cached);
        }

        info!(
            width = image.width(),
            height = image.height(),
            "uploading OpenCV template"
        );
        let handle = self
            .host
            .upload_template(RgbaPixels::from(image.as_rgba8()))
            .await?;
        info!(?handle, "uploaded OpenCV template");
        let remote = Arc::new(RemoteTemplate {
            handle,
            release: Some(self.release.clone()),
        });
        image.template.set(Arc::clone(&remote));

        Ok(remote)
    }

    /// Searches `source` for `template`.
    pub async fn find(
        &self,
        source: &Image,
        template: &Image,
        options: FindImageTemplateOptions,
        search_one: bool,
        token: &CancellationToken,
        progress: &mpsc::UnboundedSender<FindImageProgress>,
    ) -> Result<Vec<Match>> {
        if let FindOutcome::Matches(matches) = self
            .try_find(source, template, options, search_one, token, progress)
            .await?
        {
            return Ok(matches);
        }

        // The extension restarted and no longer knows our handles. Drop them so
        // the next attempt re-uploads, and try exactly once more: the retry runs
        // against a live connection, so a second miss is a real fault.
        source.source.reset();
        template.template.reset();

        match self
            .try_find(source, template, options, search_one, token, progress)
            .await?
        {
            FindOutcome::Matches(matches) => Ok(matches),
            FindOutcome::UnknownHandle => Err(eyre!(UNKNOWN_HANDLE_AFTER_RETRY)),
        }
    }

    async fn try_find(
        &self,
        source: &Image,
        template: &Image,
        options: FindImageTemplateOptions,
        search_one: bool,
        token: &CancellationToken,
        progress: &mpsc::UnboundedSender<FindImageProgress>,
    ) -> Result<FindOutcome> {
        let source_handle = self.source_handle(source).await?;
        let template_handle = self.template_handle(template).await?;
        let (request_id, _guard) = self.begin(progress.clone());

        info!(%request_id, search_one, "sending OpenCV search request");
        let outcome = self
            .cancellable(
                request_id,
                token,
                self.host.find(
                    request_id,
                    source_handle.handle,
                    template_handle.handle,
                    options,
                    search_one,
                ),
            )
            .await?;
        match &outcome {
            FindOutcome::Matches(matches) => {
                info!(%request_id, matches = matches.len(), "received OpenCV search reply");
            }
            FindOutcome::UnknownHandle => {
                info!(%request_id, "OpenCV search reported unknown handles");
            }
        }
        Ok(outcome)
    }

    /// Captures the area described by `capture` and searches it for `template`.
    ///
    /// Matches come back in capture-local coordinates.
    pub async fn find_on_screen(
        &self,
        capture: &CaptureSpec,
        template: &Image,
        options: FindImageTemplateOptions,
        search_one: bool,
        token: &CancellationToken,
        progress: &mpsc::UnboundedSender<FindImageProgress>,
    ) -> Result<Vec<Match>> {
        if let FindOutcome::Matches(matches) = self
            .try_find_on_screen(capture, template, options, search_one, token, progress)
            .await?
        {
            return Ok(matches);
        }

        // See `find`: one re-upload and retry, then give up.
        template.template.reset();

        match self
            .try_find_on_screen(capture, template, options, search_one, token, progress)
            .await?
        {
            FindOutcome::Matches(matches) => Ok(matches),
            FindOutcome::UnknownHandle => Err(eyre!(UNKNOWN_HANDLE_AFTER_RETRY)),
        }
    }

    async fn try_find_on_screen(
        &self,
        capture: &CaptureSpec,
        template: &Image,
        options: FindImageTemplateOptions,
        search_one: bool,
        token: &CancellationToken,
        progress: &mpsc::UnboundedSender<FindImageProgress>,
    ) -> Result<FindOutcome> {
        let template_handle = self.template_handle(template).await?;
        let (request_id, _guard) = self.begin(progress.clone());

        self.cancellable(
            request_id,
            token,
            self.host.find_on_screen(
                request_id,
                capture.clone(),
                template_handle.handle,
                options,
                search_one,
            ),
        )
        .await
    }
}

async fn next_release(
    releases: &mut mpsc::UnboundedReceiver<Release>,
    cancellation_token: &CancellationToken,
) -> Option<Release> {
    tokio::select! {
        () = cancellation_token.cancelled() => None,
        release = releases.recv() => release,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::{sync::mpsc, time::timeout};
    use tokio_util::sync::CancellationToken;

    use super::next_release;

    #[tokio::test]
    async fn idle_release_drain_stops_when_cancelled() {
        let (_sender, mut receiver) = mpsc::unbounded_channel();
        let cancellation_token = CancellationToken::new();
        let wait = next_release(&mut receiver, &cancellation_token);
        tokio::pin!(wait);

        assert!(timeout(Duration::ZERO, &mut wait).await.is_err());

        cancellation_token.cancel();

        assert!(
            timeout(Duration::from_secs(1), &mut wait)
                .await
                .expect("release drain should stop after cancellation")
                .is_none()
        );
    }
}
