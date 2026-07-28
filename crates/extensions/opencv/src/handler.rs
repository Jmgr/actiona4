use std::{
    collections::{HashMap, hash_map::Entry},
    sync::Arc,
    time::Duration,
};

use color_eyre::{Result, eyre::eyre};
use extension::protocols::opencv::{
    CaptureSpec, FindImageProgress, FindImageStep, FindImageTemplateOptions, FindOutcome, Match,
    OpenCVProtocolExtension, RequestId, RgbaPixels, SourceHandle, TemplateHandle,
};
use parking_lot::Mutex;
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::{JoinHandle, block_in_place},
    time::sleep,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{
    capture::Capturer,
    find_image::{ProgressSink, Source, Template, cancelled},
};

/// How often a search's intra-step samples are picked up and sent on.
const SAMPLE_INTERVAL: Duration = Duration::from_millis(50);

/// A message on its way back to the host over the shared progress stream.
#[derive(Debug)]
pub enum ProgressReport {
    /// A step change, tagged with the request it belongs to. Delivered
    /// reliably, in order, and waited for before the request answers.
    Step(RequestId, FindImageProgress),

    /// A reading from inside a step. Sent without a reply and dropped freely.
    Sample(RequestId, FindImageProgress),

    /// A marker that answers once every step change queued ahead of it has
    /// reached the host. See [`OpenCVExtension::flush_progress`].
    Flush(oneshot::Sender<()>),
}

/// A cancellation token for one request, including an entry made by a cancel
/// message which raced ahead of its find message.
#[derive(Debug)]
struct Request {
    cancellation_token: CancellationToken,
    active: bool,
}

/// Removes a request from the cancellation registry once its handler returns.
///
/// The token is deliberately kept in the map after `cancel` is received so a
/// second cancellation remains harmless and the running work can keep using
/// the same child token.
struct RequestGuard {
    requests: Arc<Mutex<HashMap<RequestId, Request>>>,
    request_id: RequestId,
}

impl Drop for RequestGuard {
    fn drop(&mut self) {
        self.requests.lock().remove(&self.request_id);
    }
}

/// Prepared images owned by this process, addressed by handle.
///
/// The host caches handles on its own `Image` objects and re-uploads when it
/// gets [`FindOutcome::UnknownHandle`] back, which is what makes an extension
/// restart transparent: a fresh process simply starts with an empty registry.
#[derive(Debug)]
struct Registry {
    sources: HashMap<SourceHandle, Arc<Source>>,
    templates: HashMap<TemplateHandle, Arc<Template>>,
}

impl Registry {
    fn new() -> Self {
        Self {
            sources: HashMap::new(),
            templates: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct OpenCVExtension {
    registry: Mutex<Registry>,
    /// Root token for the extension process. Each find request gets a child,
    /// so shutting down the process cancels all work without requests
    /// cancelling one another.
    cancellation_token: CancellationToken,
    requests: Arc<Mutex<HashMap<RequestId, Request>>>,
    capturer: Capturer,
    progress: mpsc::UnboundedSender<ProgressReport>,
    task_tracker: TaskTracker,
}

impl OpenCVExtension {
    #[must_use]
    pub fn new(
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
        progress: mpsc::UnboundedSender<ProgressReport>,
    ) -> Self {
        Self {
            registry: Mutex::new(Registry::new()),
            cancellation_token: cancellation_token.clone(),
            requests: Arc::default(),
            capturer: Capturer::new(task_tracker.clone(), cancellation_token),
            progress,
            task_tracker,
        }
    }

    fn source(&self, handle: SourceHandle) -> Option<Arc<Source>> {
        self.registry.lock().sources.get(&handle).map(Arc::clone)
    }

    fn template(&self, handle: TemplateHandle) -> Option<Arc<Template>> {
        self.registry.lock().templates.get(&handle).map(Arc::clone)
    }

    /// Registers a request before it starts any work and gives it a child of
    /// the extension-lifetime token.
    fn begin_request(&self, request_id: RequestId) -> (CancellationToken, RequestGuard) {
        let token = match self.requests.lock().entry(request_id) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().active = true;
                entry.get().cancellation_token.clone()
            }
            Entry::Vacant(entry) => {
                let token = self.cancellation_token.child_token();
                entry.insert(Request {
                    cancellation_token: token.clone(),
                    active: true,
                });
                token
            }
        };

        (
            token,
            RequestGuard {
                requests: Arc::clone(&self.requests),
                request_id,
            },
        )
    }

    /// Removes a cancellation that arrived without a corresponding request.
    ///
    /// A cancelled placeholder normally becomes active immediately when the
    /// earlier find message's handler is scheduled. The timeout prevents a
    /// failed or pre-cancelled host call from growing the registry forever.
    fn schedule_cancelled_placeholder_cleanup(&self, request_id: RequestId) {
        const PLACEHOLDER_TTL: Duration = Duration::from_secs(10);

        let requests = self.requests.clone();
        let cancellation_token = self.cancellation_token.clone();
        self.task_tracker.spawn(async move {
            tokio::select! {
                () = cancellation_token.cancelled() => {},
                () = sleep(PLACEHOLDER_TTL) => {
                    let mut requests = requests.lock();
                    if requests.get(&request_id).is_some_and(|request| !request.active) {
                        requests.remove(&request_id);
                    }
                }
            }
        });
    }

    /// Builds the sink a search reports through, and the tasks that carry what
    /// it writes to the host.
    ///
    /// Step changes are forwarded one for one. Samples are read from the watch
    /// channel no more often than [`SAMPLE_INTERVAL`], so a search that reports
    /// per tile costs the same number of messages as one that reports rarely.
    ///
    /// The returned handle finishes once the search has dropped the sink and
    /// every step change it wrote has been queued for the host.
    fn progress_sink(
        &self,
        request_id: RequestId,
    ) -> (ProgressSink, JoinHandle<()>, JoinHandle<()>) {
        let (steps, mut step_updates) = mpsc::unbounded_channel();
        let (samples, mut sample_updates) = watch::channel(FindImageProgress::default());

        let sample_progress = self.progress.clone();
        let sampling = self.task_tracker.spawn(async move {
            // Ends with the search: the sink holds the only sender, and losing
            // it closes the channel.
            while sample_updates.changed().await.is_ok() {
                let sample = *sample_updates.borrow_and_update();
                if sample_progress
                    .send(ProgressReport::Sample(request_id, sample))
                    .is_err()
                {
                    break;
                }

                sleep(SAMPLE_INTERVAL).await;
            }
        });

        let step_progress = self.progress.clone();
        let forwarding = self.task_tracker.spawn(async move {
            while let Some(update) = step_updates.recv().await {
                if step_progress
                    .send(ProgressReport::Step(request_id, update))
                    .is_err()
                {
                    break;
                }
            }
        });

        (ProgressSink::new(steps, samples), forwarding, sampling)
    }

    /// Waits until every step change queued so far has reached the host.
    ///
    /// The stream is drained by a task of its own, so without this a reply
    /// would overtake the changes a search sent just before finishing. The host
    /// stops listening for a request as soon as it has an answer for it, so
    /// those last steps — including `Finished` at 1 — would be dropped.
    ///
    /// Samples need no such care: they are allowed to go missing, which is the
    /// whole reason the two travel separately.
    async fn flush_progress(&self) {
        let (flushed, delivered) = oneshot::channel();

        if self.progress.send(ProgressReport::Flush(flushed)).is_ok() {
            // An error means the drain has stopped, leaving nothing to wait for.
            let _ = delivered.await;
        }
    }

    /// Runs one search on the blocking pool, so the IPC reactor stays free to
    /// service `cancel` while OpenCV is busy.
    async fn search(
        &self,
        request_id: RequestId,
        source: Arc<Source>,
        template: Arc<Template>,
        options: FindImageTemplateOptions,
        search_one: bool,
        cancellation_token: CancellationToken,
    ) -> Result<Vec<Match>> {
        let (progress, forwarding, sampling) = self.progress_sink(request_id);
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                if search_one {
                    source
                        .find_template(&template, options, &cancellation_token, &progress)
                        .map(|found| found.into_iter().collect())
                } else {
                    source.find_template_all(&template, options, &cancellation_token, &progress)
                }
            })
            .await;

        // Nobody wants a reading from a search that has already finished, and
        // the sampler may still be holding one it slept through. Stopping it
        // here keeps that stale sample off the wire entirely.
        sampling.abort();

        // The search has dropped its sender by now, so this only waits for what
        // it already wrote, and then for the host to have seen all of it.
        let _ = forwarding.await;
        self.flush_progress().await;

        result.map_err(|error| eyre!("find task failed: {error}"))?
    }
}

impl OpenCVProtocolExtension for OpenCVExtension {
    async fn upload_source(&self, image: RgbaPixels) -> Result<SourceHandle> {
        let source = block_in_place(|| Source::from_rgba(image.as_raw(), image.size()))?;

        let mut registry = self.registry.lock();
        let handle = SourceHandle::generate();
        registry.sources.insert(handle, source);

        Ok(handle)
    }

    async fn upload_template(&self, image: RgbaPixels) -> Result<TemplateHandle> {
        let template = block_in_place(|| Template::from_rgba(image.as_raw(), image.size()))?;

        let mut registry = self.registry.lock();
        let handle = TemplateHandle::generate();
        registry.templates.insert(handle, template);

        Ok(handle)
    }

    async fn release_source(&self, handle: SourceHandle) -> Result<()> {
        self.registry.lock().sources.remove(&handle);
        Ok(())
    }

    async fn release_template(&self, handle: TemplateHandle) -> Result<()> {
        self.registry.lock().templates.remove(&handle);
        Ok(())
    }

    async fn find(
        &self,
        request_id: RequestId,
        source: SourceHandle,
        template: TemplateHandle,
        options: FindImageTemplateOptions,
        search_one: bool,
    ) -> Result<FindOutcome> {
        let (cancellation_token, _request) = self.begin_request(request_id);
        let (Some(source), Some(template)) = (self.source(source), self.template(template)) else {
            return Ok(FindOutcome::UnknownHandle);
        };

        let matches = self
            .search(
                request_id,
                source,
                template,
                options,
                search_one,
                cancellation_token,
            )
            .await?;

        Ok(FindOutcome::Matches(matches))
    }

    async fn find_on_screen(
        &self,
        request_id: RequestId,
        capture: CaptureSpec,
        template: TemplateHandle,
        options: FindImageTemplateOptions,
        search_one: bool,
    ) -> Result<FindOutcome> {
        let (cancellation_token, _request) = self.begin_request(request_id);
        let Some(template) = self.template(template) else {
            return Ok(FindOutcome::UnknownHandle);
        };

        let _ = self.progress.send(ProgressReport::Step(
            request_id,
            FindImageProgress::started(FindImageStep::Capturing, 0.0),
        ));

        let captured = self.capturer.capture(&capture, &cancellation_token).await?;
        if cancellation_token.is_cancelled() {
            return Err(cancelled());
        }
        let source = block_in_place(|| Source::from_bgra(&captured.bgra, captured.size))?;
        if cancellation_token.is_cancelled() {
            return Err(cancelled());
        }

        let matches = self
            .search(
                request_id,
                source,
                template,
                options,
                search_one,
                cancellation_token,
            )
            .await?;

        Ok(FindOutcome::Matches(matches))
    }

    async fn cancel(&self, request_id: RequestId) -> Result<()> {
        let (token, placeholder) = match self.requests.lock().entry(request_id) {
            Entry::Occupied(entry) => (entry.get().cancellation_token.clone(), false),
            Entry::Vacant(entry) => {
                let token = self.cancellation_token.child_token();
                entry.insert(Request {
                    cancellation_token: token.clone(),
                    active: false,
                });
                (token, true)
            }
        };
        token.cancel();
        if placeholder {
            self.schedule_cancelled_placeholder_cleanup(request_id);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use extension::protocols::opencv::{
        OpenCVProtocolExtension, RequestIdProvider, SourceHandle, TemplateHandle,
    };
    use tokio::sync::mpsc;
    use tokio_util::{sync::CancellationToken, task::TaskTracker};

    use super::OpenCVExtension;

    #[test]
    fn replacement_handles_are_distinct_from_stale_handles() {
        assert_ne!(SourceHandle::generate(), SourceHandle::generate());
        assert_ne!(TemplateHandle::generate(), TemplateHandle::generate());
    }

    #[test]
    fn root_cancellation_reaches_request_children() {
        let root = CancellationToken::new();
        let (progress, _receiver) = mpsc::unbounded_channel();
        let extension = OpenCVExtension::new(TaskTracker::new(), root.clone(), progress);
        let (request, _guard) = extension.begin_request(RequestIdProvider::default().next_id());

        root.cancel();

        assert!(request.is_cancelled());
    }

    #[tokio::test]
    async fn cancellation_before_request_starts_cancels_its_child() {
        let root = CancellationToken::new();
        let (progress, _receiver) = mpsc::unbounded_channel();
        let extension = OpenCVExtension::new(TaskTracker::new(), root.clone(), progress);
        let request_id = RequestIdProvider::default().next_id();

        extension
            .cancel(request_id)
            .await
            .expect("cancelling an unstarted request should succeed");
        let (request, _guard) = extension.begin_request(request_id);

        assert!(request.is_cancelled());
        root.cancel();
    }
}
