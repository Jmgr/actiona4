use std::time::Duration;

use color_eyre::Result;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::api::image::Image;

pub mod js;
mod platform;

#[derive(Clone, Debug, Default)]
pub struct NotificationOptions {
    pub body: Option<String>,
    pub icon: Option<Image>,
    pub timeout: Option<Duration>,
}

#[derive(Default)]
pub struct Notification {
    inner: platform::Notification,
}

impl Notification {
    pub async fn show(
        &self,
        text: &str,
        options: NotificationOptions,
    ) -> Result<NotificationHandle> {
        let inner = self.inner.show(text, options).await?;
        Ok(NotificationHandle { inner })
    }
}

pub struct NotificationHandle {
    inner: platform::NotificationHandle,
}

impl NotificationHandle {
    pub async fn wait_until_closed(
        self,
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Result<()> {
        self.inner
            .wait_until_closed(cancellation_token, task_tracker)
            .await
    }
}
