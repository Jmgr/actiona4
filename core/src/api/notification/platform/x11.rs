use notify_rust::Hint;
use parking_lot::Mutex;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::{
    api::notification::{NotificationOptions, Result},
    cancel_on,
};

#[derive(Default)]
pub struct Notification;

impl Notification {
    pub async fn show(
        &self,
        text: &str,
        mut options: NotificationOptions,
    ) -> Result<NotificationHandle> {
        let notification = Self::build_notification(text, &mut options).await?;
        let inner = notification.show_async().await?;

        Ok(NotificationHandle { inner })
    }

    async fn build_notification(
        text: &str,
        options: &mut NotificationOptions,
    ) -> Result<notify_rust::Notification> {
        let mut notification = notify_rust::Notification::new();
        notification.summary(text);

        if let Some(body) = &options.body {
            notification.body(body);
        }

        notification.hint(Hint::Transient(true));

        if let Some(timeout) = options.timeout {
            notification.timeout(timeout);
        }

        if let Some(icon) = options.icon.take() {
            let icon = notify_rust::Image::try_from(icon.into_rgba8())?;
            notification.image_data(icon);
        }

        Ok(notification)
    }
}

pub struct NotificationHandle {
    inner: notify_rust::NotificationHandle,
}

impl NotificationHandle {
    pub async fn wait_until_closed(
        self,
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Result<()> {
        let (sender, receiver) = oneshot::channel::<()>();
        let sender = Mutex::new(Some(sender));

        task_tracker.spawn_blocking(move || {
            self.inner.on_close(|_reason| {
                if let Some(sender) = sender.lock().take() {
                    let _ = sender.send(());
                }
            });
        });

        cancel_on(&cancellation_token, receiver).await??;

        Ok(())
    }
}
