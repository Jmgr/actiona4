use std::time::Duration;

use color_eyre::Result;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::api::{image::Image, point::Point};

pub mod js;
pub mod platform;

#[derive(Clone, Debug, Default)]
pub struct NotificationOptions {
    pub title: Option<String>,
    pub body: Option<String>,
    pub timeout: Option<Duration>,
    pub actions: Vec<NotificationAction>,
    pub icon: Option<Image>,

    // Linux-specific
    pub app_name: Option<String>,
    pub icon_name: Option<String>,
    pub auto_icon: bool,
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

    // Windows-specific
    pub attribution_text: Option<String>,
    pub hero_image: Option<Image>,
    pub icon_crop_circle: bool,
    pub scenario: Option<NotificationScenario>,
    pub sound: Option<NotificationSound>,
    pub sound_looping: bool,
    pub silent: bool,
    pub header: Option<NotificationHeader>,
    pub inputs: Vec<NotificationInput>,
    pub selections: Vec<NotificationSelection>,
    pub tag: Option<String>,
    pub group: Option<String>,
    pub remote_id: Option<String>,
    pub launch: Option<String>,
    pub use_button_style: bool,
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
    // Windows-specific
    pub action_type: Option<String>,
    pub activation_type: Option<NotificationActivationType>,
    pub placement: Option<NotificationActionPlacement>,
    pub button_style: Option<NotificationButtonStyle>,
    pub input_id: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationScenario {
    Reminder,
    Alarm,
    IncomingCall,
    Urgent,
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationSound {
    Default,
    IM,
    Mail,
    Reminder,
    SMS,
    None,
    LoopingAlarm,
    LoopingAlarm2,
    LoopingAlarm3,
    LoopingAlarm4,
    LoopingAlarm5,
    LoopingAlarm6,
    LoopingAlarm7,
    LoopingAlarm8,
    LoopingAlarm9,
    LoopingAlarm10,
    LoopingCall,
    LoopingCall2,
    LoopingCall3,
    LoopingCall4,
    LoopingCall5,
    LoopingCall6,
    LoopingCall7,
    LoopingCall8,
    LoopingCall9,
    LoopingCall10,
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationActivationType {
    Foreground,
    Background,
    Protocol,
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationActionPlacement {
    ContextMenu,
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationButtonStyle {
    Success,
    Critical,
}

#[derive(Clone, Debug)]
pub struct NotificationHeader {
    pub id: String,
    pub title: String,
    pub arguments: String,
}

#[derive(Clone, Debug)]
pub struct NotificationInput {
    pub id: String,
    pub input_type: NotificationInputType,
    pub placeholder: Option<String>,
    pub title: Option<String>,
    pub default_input: Option<String>,
}

#[derive(Clone, Copy, Debug)]
pub enum NotificationInputType {
    Text,
    Selection,
}

#[derive(Clone, Debug)]
pub struct NotificationSelection {
    pub id: String,
    pub content: String,
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

    pub const fn capabilities() -> Result<Vec<String>> {
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
