use std::{
    env::{self, consts::EXE_SUFFIX},
    fs::exists,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use color_eyre::{Result, eyre::OptionExt};
use extension::{
    Host,
    protocol::Protocol,
    protocols::{opencv::OpenCVProtocol, selection::SelectionProtocol},
};
use tokio::{join, sync::oneshot};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, warn};

use crate::{
    api::image::find_image::{OpenCVClient, ProgressHandler, ProgressRegistry},
    runtime::async_resource::AsyncResource,
};

const EXTENSION_PREFIX: &str = "extension-";

/// The selection overlay only ever waits on a person, so a short timeout is
/// really an "is it still alive" check.
const SELECTION_TIMEOUT: Duration = Duration::from_mins(1);

/// Image matching can legitimately run for minutes on a large desktop with no
/// downscaling, so it gets a much longer leash.
const OPENCV_TIMEOUT: Duration = Duration::from_mins(10);

pub type ExtensionHandle<T> = AsyncResource<Option<Arc<T>>>;

#[derive(Clone, Debug)]
pub struct Extensions {
    selection: ExtensionHandle<Host<SelectionProtocol>>,
    opencv: ExtensionHandle<OpenCVClient>,
}

impl Extensions {
    pub async fn new(
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
        discover_extensions: bool,
    ) -> Result<Self> {
        if !discover_extensions {
            return Ok(Self {
                selection: AsyncResource::with_value(None, cancellation_token.clone()),
                opencv: AsyncResource::with_value(None, cancellation_token),
            });
        }

        let current_exe = env::current_exe()?; // TODO: will that work from within an appimage?
        let directory = current_exe
            .parent()
            .ok_or_eyre("expected current executable to have a parent directory")?;
        let selection = AsyncResource::new(cancellation_token.clone());
        let opencv = AsyncResource::new(cancellation_token.clone());

        let (selection_result, opencv_result) = join!(
            Self::setup_selection(
                directory,
                &task_tracker,
                &cancellation_token,
                selection.clone(),
            ),
            Self::setup_opencv(
                directory,
                &task_tracker,
                &cancellation_token,
                opencv.clone(),
            ),
        );
        selection_result?;
        opencv_result?;

        Ok(Self { selection, opencv })
    }

    pub async fn maybe_selection(&self) -> Result<Option<Arc<Host<SelectionProtocol>>>> {
        Ok(self.selection.wait_get().await?.as_ref().clone())
    }

    pub async fn selection(&self) -> Result<Arc<Host<SelectionProtocol>>> {
        self.maybe_selection()
            .await?
            .ok_or_eyre("selection extension is not available")
    }

    pub async fn maybe_opencv(&self) -> Result<Option<Arc<OpenCVClient>>> {
        Ok(self.opencv.wait_get().await?.as_ref().clone())
    }

    pub async fn opencv(&self) -> Result<Arc<OpenCVClient>> {
        self.maybe_opencv()
            .await?
            .ok_or_eyre("opencv extension is not available")
    }

    /// Locates an extension, builds its host via `build`, and starts supervising it.
    ///
    /// `build` returns the host to supervise plus the value callers should get
    /// from the handle. For most extensions those are the same thing; the
    /// OpenCV one wraps its host in a client.
    async fn setup<P, T, F, Fut>(
        name: &'static str,
        directory: &Path,
        task_tracker: &TaskTracker,
        handle: ExtensionHandle<T>,
        build: F,
    ) -> Result<()>
    where
        P: Protocol,
        T: Send + Sync + 'static,
        F: FnOnce(PathBuf) -> Fut,
        Fut: Future<Output = Result<(Arc<Host<P>>, Arc<T>)>>,
    {
        let Some(path) = find_extension_executable(name, directory) else {
            handle.set(None);
            return Ok(());
        };

        let (host, value) = build(path).await?;
        spawn_host(name, host, value, task_tracker, handle);

        Ok(())
    }

    async fn setup_selection(
        directory: &Path,
        task_tracker: &TaskTracker,
        cancellation_token: &CancellationToken,
        handle: ExtensionHandle<Host<SelectionProtocol>>,
    ) -> Result<()> {
        Self::setup("selection", directory, task_tracker, handle, async |path| {
            let host = Arc::new(
                Host::<SelectionProtocol>::new(
                    &path,
                    task_tracker.clone(),
                    cancellation_token.clone(),
                    SELECTION_TIMEOUT,
                )
                .await?,
            );

            Ok((Arc::clone(&host), host))
        })
        .await
    }

    async fn setup_opencv(
        directory: &Path,
        task_tracker: &TaskTracker,
        cancellation_token: &CancellationToken,
        handle: ExtensionHandle<OpenCVClient>,
    ) -> Result<()> {
        Self::setup("opencv", directory, task_tracker, handle, async |path| {
            // The registry is shared with the handler so progress reported by
            // the extension reaches the task that asked for it.
            let progress = Arc::new(ProgressRegistry::default());
            let host = Arc::new(
                Host::<OpenCVProtocol>::with_handler_impl(
                    &path,
                    task_tracker.clone(),
                    cancellation_token.clone(),
                    OPENCV_TIMEOUT,
                    ProgressHandler::new(Arc::clone(&progress)),
                )
                .await?,
            );
            let client = OpenCVClient::new(
                Arc::clone(&host),
                progress,
                task_tracker,
                cancellation_token,
            );

            Ok((host, client))
        })
        .await
    }
}

/// Supervises an extension process and publishes `value` once it has connected,
/// so callers never see a handle before the extension can answer.
fn spawn_host<P: Protocol, T: Send + Sync + 'static>(
    name: &'static str,
    host: Arc<Host<P>>,
    value: Arc<T>,
    task_tracker: &TaskTracker,
    handle: ExtensionHandle<T>,
) {
    let (ready_sender, ready_receiver) = oneshot::channel();

    task_tracker.spawn(async move {
        if let Err(error) = host.run(ready_sender).await {
            error!("{name} extension host stopped: {error}");
        }
    });
    task_tracker.spawn(async move {
        if let Err(error) = ready_receiver.await {
            error!("{name} extension readiness wait failed: {error}");
            return;
        }

        handle.set(Some(value));
    });
}

/// Resolves an extension's executable, or logs where it looked and gives up.
///
/// A missing extension is not fatal: the features it backs report a clear
/// error when used, and everything else keeps working.
fn find_extension_executable(name: &str, directory: &Path) -> Option<PathBuf> {
    let candidates = extension_executable_candidates(name, directory);
    let found = candidates
        .iter()
        .find(|path| exists(path).unwrap_or(false))
        .cloned();

    if found.is_none() {
        warn!(
            "no extension executable found for {}. Looked in: {}",
            name,
            candidates
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    found
}

pub(crate) fn extension_executable_name(name: &str) -> String {
    format!("{EXTENSION_PREFIX}{name}{EXE_SUFFIX}")
}

pub(crate) fn extension_executable_candidates(name: &str, directory: &Path) -> Vec<PathBuf> {
    let filename = extension_executable_name(name);
    let mut candidates = vec![directory.join(&filename)];

    if let Some(parent) = directory.parent() {
        candidates.push(parent.join(filename));
    }

    candidates
}
