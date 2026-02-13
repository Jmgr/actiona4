use std::time::Duration;

use color_eyre::Result;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

use crate::api::{image::Image, point::Point};

pub mod js;
mod platform;

#[derive(Clone, Debug, Default)]
pub struct NotificationOptions {
    pub title: Option<String>,
    pub app_name: Option<String>,
    pub body: Option<String>,
    pub icon_name: Option<String>,
    pub auto_icon: bool,
    pub icon: Option<Image>,
    pub timeout: Option<Duration>,
    pub action_icons: Option<bool>,
    pub category: Option<String>,
    pub desktop_entry: Option<String>,
    pub resident: Option<bool>,
    pub sound_file: Option<String>,
    pub sound_name: Option<String>,
    pub suppress_sound: Option<bool>,
    pub transient: Option<bool>,
    pub point: Option<Point>,
    pub urgency: Option<NotificationUrgency>,
    pub custom_hints: Vec<NotificationCustomHint>,
    pub custom_int_hints: Vec<NotificationCustomIntHint>,
    pub actions: Vec<NotificationAction>,
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationUrgency {
    Low,
    Normal,
    Critical,
}

#[derive(Clone, Debug)]
pub struct NotificationCustomHint {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct NotificationCustomIntHint {
    pub name: String,
    pub value: i32,
}

#[derive(Clone, Debug)]
pub struct NotificationAction {
    pub identifier: String,
    pub label: String,
}

#[derive(Default)]
pub struct Notification {
    inner: platform::Notification,
}

impl Notification {
    pub async fn show(&self, options: NotificationOptions) -> Result<NotificationHandle> {
        let inner = self.inner.show(options).await?;
        Ok(NotificationHandle { inner })
    }

    pub fn capabilities() -> Result<Vec<String>> {
        platform::Notification::capabilities()
    }
}

pub struct NotificationHandle {
    inner: platform::NotificationHandle,
}

impl NotificationHandle {
    pub async fn update(&self, options: NotificationOptions) -> Result<()> {
        self.inner.update(options).await
    }

    pub async fn wait_for_action(
        self,
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Result<Option<String>> {
        self.inner
            .wait_for_action(cancellation_token, task_tracker)
            .await
    }
}
