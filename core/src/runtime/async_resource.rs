use std::sync::Arc;

use arc_swap::ArcSwapOption;
use color_eyre::Result;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

use crate::cancel_on;

#[derive(Clone, Debug)]
pub struct AsyncResource<T> {
    value: Arc<ArcSwapOption<T>>,
    notify: Arc<Notify>,
    cancellation_token: CancellationToken,
}

impl<T> AsyncResource<T> {
    /// Creates a new resource.
    #[must_use]
    pub fn new(cancellation_token: CancellationToken) -> Self {
        Self {
            value: Default::default(),
            notify: Default::default(),
            cancellation_token,
        }
    }

    /// Updates the resource.
    pub fn set(&self, value: T) {
        self.value.store(Some(Arc::new(value)));
        self.notify.notify_waiters();
    }

    /// Returns the current value if available, without waiting.
    #[must_use]
    pub fn try_get(&self) -> Option<Arc<T>> {
        self.value.load_full()
    }

    /// Waits until the resource is available.
    /// Returns an error if cancelled.
    pub async fn wait_get(&self) -> Result<Arc<T>> {
        loop {
            // Register interest first so we can't miss the wakeup.
            let notified = self.notify.notified();

            if let Some(v) = self.value.load_full() {
                return Ok(v);
            }

            cancel_on(&self.cancellation_token, notified).await?;
        }
    }

    /// Waits until the resource has changed.
    /// There is no guarantee that the value is different.
    /// Returns an error if cancelled.
    pub async fn changed(&self) -> Result<()> {
        cancel_on(&self.cancellation_token, self.notify.notified()).await
    }
}
