use std::{
    env::{self, consts::EXE_SUFFIX},
    fs::exists,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use color_eyre::{Result, eyre::OptionExt};
use extension::{Host, protocol::Protocol, protocols::selection::SelectionProtocol};
use tokio::{join, sync::oneshot};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, warn};

use crate::runtime::async_resource::AsyncResource;

const EXTENSION_PREFIX: &str = "extension-";

pub type ExtensionHandle<T> = AsyncResource<Option<Arc<Host<T>>>>;

#[derive(Clone, Debug)]
pub struct Extensions {
    selection: ExtensionHandle<SelectionProtocol>,
}

impl Extensions {
    pub async fn new(
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        if Self::disable_extension_discovery_for_tests() {
            let _ = task_tracker;
            let _ = cancellation_token;
            return Ok(Self {
                selection: AsyncResource::with_value(None, cancellation_token.clone()),
            });
        }

        let current_exe = env::current_exe()?; // TODO: will that work from within an appimage?
        let directory = current_exe
            .parent()
            .ok_or_eyre("expected current executable to have a parent directory")?;
        let selection = AsyncResource::new(cancellation_token.clone());
        let lookup = join!(Self::lookup_extension::<SelectionProtocol>(
            "selection",
            directory,
            task_tracker,
            cancellation_token.clone(),
            selection.clone(),
        ));
        lookup.0?;

        Ok(Self { selection })
    }

    pub async fn selection(&self) -> Result<Option<Arc<Host<SelectionProtocol>>>> {
        Ok(self.selection.wait_get().await?.as_ref().clone())
    }

    const fn disable_extension_discovery_for_tests() -> bool {
        cfg!(test)
    }

    async fn lookup_extension<P: Protocol>(
        name: &str,
        directory: &Path,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
        handle: ExtensionHandle<P>,
    ) -> Result<()> {
        let candidates = extension_executable_candidates(name, directory);
        let executable_filepath = candidates.iter().find(|path| exists(path).unwrap_or(false));
        let Some(executable_filepath) = executable_filepath else {
            warn!(
                "no extension executable found for {}. Looked in: {}",
                name,
                candidates
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            handle.set(None);
            return Ok(());
        };

        let host = Arc::new(
            Host::<P>::new(
                executable_filepath,
                task_tracker.clone(),
                cancellation_token.clone(),
                Duration::from_mins(1),
            )
            .await?,
        );

        let (ready_sender, ready_receiver) = oneshot::channel();

        let local_host = host.clone();
        task_tracker.spawn(async move {
            if let Err(error) = local_host.run(ready_sender).await {
                error!("selection extension host stopped: {error}");
            }
        });
        task_tracker.spawn(async move {
            if let Err(error) = ready_receiver.await {
                error!("selection extension readiness wait failed: {error}");
                return;
            }

            handle.set(Some(host));
        });

        Ok(())
    }
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
