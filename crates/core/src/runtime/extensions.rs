use std::{env::consts::EXE_SUFFIX, fs::exists, path::Path, sync::Arc, time::Duration};

use color_eyre::{Result, eyre::OptionExt};
use extension::{Host, protocol::Protocol, protocols::selection::SelectionProtocol};
use tokio::join;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, warn};

const EXTENSION_PREFIX: &str = "extension-";

#[derive(Clone, Debug)]
pub struct Extensions {
    selection: Option<Arc<Host<SelectionProtocol>>>,
}

impl Extensions {
    pub async fn new(
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        if Self::disable_extension_discovery_for_tests() {
            let _ = task_tracker;
            let _ = cancellation_token;
            return Ok(Self { selection: None });
        }

        let current_exe = std::env::current_exe()?; // TODO: will that work from within an appimage?
        let directory = current_exe
            .parent()
            .ok_or_eyre("expected current executable to have a parent directory")?;
        let selection = join!(Self::lookup_extension::<SelectionProtocol>(
            "selection",
            directory,
            task_tracker,
            cancellation_token,
        ));

        Ok(Self {
            selection: selection.0?,
        })
    }

    pub fn selection(&self) -> Option<&Host<SelectionProtocol>> {
        self.selection.as_deref()
    }

    fn disable_extension_discovery_for_tests() -> bool {
        cfg!(test)
    }

    async fn lookup_extension<P: Protocol>(
        name: &str,
        directory: &Path,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Result<Option<Arc<Host<P>>>> {
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
            return Ok(None);
        };

        let host = Arc::new(
            Host::<P>::new(
                executable_filepath,
                task_tracker.clone(),
                cancellation_token.clone(),
                Duration::from_secs(60),
            )
            .await?,
        );
        let local_host = host.clone();

        task_tracker.spawn(async move {
            if let Err(error) = local_host.run().await {
                error!("selection extension host stopped: {error}");
            }
        });

        Ok(Some(host))
    }
}

pub(crate) fn extension_executable_name(name: &str) -> String {
    format!("{EXTENSION_PREFIX}{name}{EXE_SUFFIX}")
}

pub(crate) fn extension_executable_candidates(
    name: &str,
    directory: &Path,
) -> Vec<std::path::PathBuf> {
    let filename = extension_executable_name(name);
    let mut candidates = vec![directory.join(&filename)];

    if let Some(parent) = directory.parent() {
        candidates.push(parent.join(filename));
    }

    candidates
}
