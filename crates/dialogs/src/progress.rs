//! Handle to an open progress dialog.

use tokio::{sync::watch, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::ProgressOptions;

/// What a progress dialog shows.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgressState {
    /// Between 0 and 1, or `None` to show activity instead.
    pub value: Option<f64>,
    pub text: String,
}

impl ProgressState {
    /// What a new progress dialog shows: backends pass this to the dialog as it starts.
    pub fn initial(options: &ProgressOptions) -> Self {
        Self {
            value: options.value.map(|value| value.clamp(0.0, 1.0)),
            text: options.text.clone(),
        }
    }
}

/// An open progress dialog. Dropping it closes the dialog.
///
/// Updates are sent to a task that drives the dialog, and only the latest state is shown, so
/// they can be made in a tight loop.
#[derive(Debug)]
pub struct Progress {
    /// Dropping the sender tells the task to close the dialog.
    state: Option<watch::Sender<ProgressState>>,
    cancelled: CancellationToken,
    task: Option<JoinHandle<()>>,
}

impl Progress {
    /// `task` drives the dialog: it shows each new state from `state`, cancels `cancelled` if
    /// the user cancels the dialog, and closes the dialog once `state` is closed.
    #[cfg_attr(windows, allow(dead_code))]
    pub(crate) const fn new(
        state: watch::Sender<ProgressState>,
        cancelled: CancellationToken,
        task: JoinHandle<()>,
    ) -> Self {
        Self {
            state: Some(state),
            cancelled,
            task: Some(task),
        }
    }

    fn update(&self, update: impl FnOnce(&mut ProgressState)) {
        if let Some(state) = &self.state {
            state.send_if_modified(|state| {
                let previous = state.clone();
                update(state);
                *state != previous
            });
        }
    }

    /// Sets the progress, between 0 and 1, or shows activity instead with `None`, for work of
    /// unknown length.
    pub fn set_value(&self, value: Option<f64>) {
        self.update(|state| state.value = value.map(|value| value.clamp(0.0, 1.0)));
    }

    pub fn set_text(&self, text: &str) {
        self.update(|state| text.clone_into(&mut state.text));
    }

    /// Whether the user cancelled or closed the dialog.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.is_cancelled()
    }

    /// Resolves when the user cancels or closes the dialog.
    pub async fn cancelled(&self) {
        self.cancelled.cancelled().await;
    }

    /// Closes the dialog and waits until it is gone.
    pub async fn close(mut self) {
        self.state = None;
        if let Some(task) = self.task.take() {
            _ = task.await;
        }
    }
}
