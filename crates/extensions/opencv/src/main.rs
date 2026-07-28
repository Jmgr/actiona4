#![cfg_attr(windows, windows_subsystem = "console")]

use std::{env, sync::Arc, time::Duration};

use actiona_common::sentry::setup_crash_reporting;
use color_eyre::{Result, eyre::OptionExt};
use extension::{Extension, protocols::opencv::OpenCVProtocol};
use extension_opencv::{
    find_image,
    handler::{OpenCVExtension, ProgressReport},
};
use tokio::{runtime::Builder, sync::mpsc};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::error;
use tracing_subscriber::fmt as tracing_fmt;

/// Generous compared to the selection extension's one minute: a full-desktop
/// `findAll` with no downscaling can legitimately run for several minutes.
const REQUEST_TIMEOUT: Duration = Duration::from_mins(10);

#[allow(clippy::needless_raw_strings)]
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() -> Result<()> {
    let _guard = setup_crash_reporting(built_info::PKG_NAME)?;
    tracing_fmt::init();

    find_image::setup_threading()?;

    let runtime = Builder::new_multi_thread().enable_all().build()?;
    let task_tracker = TaskTracker::new();
    let cancellation_token = CancellationToken::new();

    let result = runtime.block_on(run(&task_tracker, &cancellation_token));

    // Always tear down, even if the extension failed: leaving the process alive
    // would keep the host's stdout pipe open and stall whoever spawned us.
    cancellation_token.cancel();
    task_tracker.close();
    runtime.block_on(task_tracker.wait());

    result
}

async fn run(task_tracker: &TaskTracker, cancellation_token: &CancellationToken) -> Result<()> {
    // Pre-JIT OpenCV's Lab conversion so the first search isn't paying for it.
    task_tracker.spawn_blocking(|| {
        if let Err(error) = find_image::warm_up() {
            error!("failed to warm up OpenCV: {error}");
        }
    });

    let (progress_sender, mut progress_receiver) = mpsc::unbounded_channel();

    let extension = Arc::new(Extension::<OpenCVProtocol>::with_handler_impl(
        env::args().nth(1).ok_or_eyre("expected a key")?.into(),
        task_tracker.clone(),
        cancellation_token.clone(),
        REQUEST_TIMEOUT,
        OpenCVExtension::new(
            task_tracker.clone(),
            cancellation_token.clone(),
            progress_sender,
        ),
    ));

    // The handler is built before the `Extension` that owns the connection, so
    // progress reaches the host through this drain task rather than a direct
    // back-reference.
    //
    // The loop has to watch the cancellation token rather than just draining
    // until the channel closes: the sender lives inside the handler, which this
    // task's own `Arc<Extension>` keeps alive, so `recv()` would never return
    // `None` and shutdown would hang waiting for this task to finish.
    let progress_extension = Arc::clone(&extension);
    let progress_token = cancellation_token.clone();
    task_tracker.spawn(async move {
        loop {
            let report = tokio::select! {
                () = progress_token.cancelled() => break,
                report = progress_receiver.recv() => report,
            };
            let Some(report) = report else {
                break;
            };

            match report {
                ProgressReport::Step(request_id, progress) => {
                    if let Err(error) = progress_extension.progress(request_id, progress).await {
                        // A failed report is not worth failing the search over: the
                        // host either went away or is no longer interested.
                        error!("failed to report progress for request {request_id}: {error}");
                    }
                }
                // Nothing to await: the sample is queued and forgotten, so a
                // busy host slows nothing down here.
                ProgressReport::Sample(request_id, progress) => {
                    if let Err(error) = progress_extension.progress_sample(request_id, progress) {
                        error!("failed to sample progress for request {request_id}: {error}");
                    }
                }
                // Every update sent before this one has now been answered for,
                // which is all the waiting search needs to know.
                ProgressReport::Flush(flushed) => {
                    let _ = flushed.send(());
                }
            }
        }
    });

    extension.run().await
}
