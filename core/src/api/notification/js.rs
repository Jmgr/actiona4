use std::sync::Arc;

use macros::{FromJsObject, FromSerde, IntoSerde};
use parking_lot::Mutex;
use rquickjs::{
    Ctx, Exception, JsLifetime, Promise, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
use tokio_util::task::TaskTracker;

use super::{
    NotificationAction, NotificationActionPlacement, NotificationActivationType,
    NotificationButtonStyle, NotificationCustomHint, NotificationCustomIntHint, NotificationHeader,
    NotificationInput, NotificationInputType, NotificationOptions, NotificationScenario,
    NotificationSelection, NotificationSound, NotificationUrgency,
};
use crate::{
    IntoJsResult,
    api::{
        image::js::JsImage,
        js::{
            abort_controller::JsAbortSignal,
            classes::{HostClass, SingletonClass, register_enum, register_host_class},
            duration::JsDuration,
            task::task_with_token,
        },
        point::js::JsPoint,
    },
    runtime::WithUserData,
    types::display::display_with_type,
};

/// @expand
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "NotificationUrgency")]
pub enum JsNotificationUrgency {
    /// `NotificationUrgency.Low`
    Low,
    /// `NotificationUrgency.Normal`
    Normal,
    /// `NotificationUrgency.Critical`
    Critical,
}

impl From<JsNotificationUrgency> for NotificationUrgency {
    fn from(value: JsNotificationUrgency) -> Self {
        match value {
            JsNotificationUrgency::Low => Self::Low,
            JsNotificationUrgency::Normal => Self::Normal,
            JsNotificationUrgency::Critical => Self::Critical,
        }
    }
}

/// Toast notification scenario.
///
/// @platforms =windows
/// @expand
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "NotificationScenario")]
pub enum JsNotificationScenario {
    /// `NotificationScenario.Reminder`
    Reminder,
    /// `NotificationScenario.Alarm`
    Alarm,
    /// `NotificationScenario.IncomingCall`
    IncomingCall,
    /// `NotificationScenario.Urgent`
    Urgent,
}

impl From<JsNotificationScenario> for NotificationScenario {
    fn from(value: JsNotificationScenario) -> Self {
        match value {
            JsNotificationScenario::Reminder => Self::Reminder,
            JsNotificationScenario::Alarm => Self::Alarm,
            JsNotificationScenario::IncomingCall => Self::IncomingCall,
            JsNotificationScenario::Urgent => Self::Urgent,
        }
    }
}

/// Notification sound.
///
/// @platforms =windows
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "NotificationSound")]
pub enum JsNotificationSound {
    /// `NotificationSound.Default`
    Default,
    /// `NotificationSound.IM`
    IM,
    /// `NotificationSound.Mail`
    Mail,
    /// `NotificationSound.Reminder`
    Reminder,
    /// `NotificationSound.SMS`
    SMS,
    /// `NotificationSound.None`
    None,
    /// `NotificationSound.LoopingAlarm`
    LoopingAlarm,
    /// `NotificationSound.LoopingAlarm2`
    LoopingAlarm2,
    /// `NotificationSound.LoopingAlarm3`
    LoopingAlarm3,
    /// `NotificationSound.LoopingAlarm4`
    LoopingAlarm4,
    /// `NotificationSound.LoopingAlarm5`
    LoopingAlarm5,
    /// `NotificationSound.LoopingAlarm6`
    LoopingAlarm6,
    /// `NotificationSound.LoopingAlarm7`
    LoopingAlarm7,
    /// `NotificationSound.LoopingAlarm8`
    LoopingAlarm8,
    /// `NotificationSound.LoopingAlarm9`
    LoopingAlarm9,
    /// `NotificationSound.LoopingAlarm10`
    LoopingAlarm10,
    /// `NotificationSound.LoopingCall`
    LoopingCall,
    /// `NotificationSound.LoopingCall2`
    LoopingCall2,
    /// `NotificationSound.LoopingCall3`
    LoopingCall3,
    /// `NotificationSound.LoopingCall4`
    LoopingCall4,
    /// `NotificationSound.LoopingCall5`
    LoopingCall5,
    /// `NotificationSound.LoopingCall6`
    LoopingCall6,
    /// `NotificationSound.LoopingCall7`
    LoopingCall7,
    /// `NotificationSound.LoopingCall8`
    LoopingCall8,
    /// `NotificationSound.LoopingCall9`
    LoopingCall9,
    /// `NotificationSound.LoopingCall10`
    LoopingCall10,
}

impl From<JsNotificationSound> for NotificationSound {
    fn from(value: JsNotificationSound) -> Self {
        match value {
            JsNotificationSound::Default => Self::Default,
            JsNotificationSound::IM => Self::IM,
            JsNotificationSound::Mail => Self::Mail,
            JsNotificationSound::Reminder => Self::Reminder,
            JsNotificationSound::SMS => Self::SMS,
            JsNotificationSound::None => Self::None,
            JsNotificationSound::LoopingAlarm => Self::LoopingAlarm,
            JsNotificationSound::LoopingAlarm2 => Self::LoopingAlarm2,
            JsNotificationSound::LoopingAlarm3 => Self::LoopingAlarm3,
            JsNotificationSound::LoopingAlarm4 => Self::LoopingAlarm4,
            JsNotificationSound::LoopingAlarm5 => Self::LoopingAlarm5,
            JsNotificationSound::LoopingAlarm6 => Self::LoopingAlarm6,
            JsNotificationSound::LoopingAlarm7 => Self::LoopingAlarm7,
            JsNotificationSound::LoopingAlarm8 => Self::LoopingAlarm8,
            JsNotificationSound::LoopingAlarm9 => Self::LoopingAlarm9,
            JsNotificationSound::LoopingAlarm10 => Self::LoopingAlarm10,
            JsNotificationSound::LoopingCall => Self::LoopingCall,
            JsNotificationSound::LoopingCall2 => Self::LoopingCall2,
            JsNotificationSound::LoopingCall3 => Self::LoopingCall3,
            JsNotificationSound::LoopingCall4 => Self::LoopingCall4,
            JsNotificationSound::LoopingCall5 => Self::LoopingCall5,
            JsNotificationSound::LoopingCall6 => Self::LoopingCall6,
            JsNotificationSound::LoopingCall7 => Self::LoopingCall7,
            JsNotificationSound::LoopingCall8 => Self::LoopingCall8,
            JsNotificationSound::LoopingCall9 => Self::LoopingCall9,
            JsNotificationSound::LoopingCall10 => Self::LoopingCall10,
        }
    }
}

/// Activation type for toast actions and headers.
///
/// @platforms =windows
/// @expand
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "NotificationActivationType")]
pub enum JsNotificationActivationType {
    /// `NotificationActivationType.Foreground`
    Foreground,
    /// `NotificationActivationType.Background`
    Background,
    /// `NotificationActivationType.Protocol`
    Protocol,
}

impl From<JsNotificationActivationType> for NotificationActivationType {
    fn from(value: JsNotificationActivationType) -> Self {
        match value {
            JsNotificationActivationType::Foreground => Self::Foreground,
            JsNotificationActivationType::Background => Self::Background,
            JsNotificationActivationType::Protocol => Self::Protocol,
        }
    }
}

/// Placement of a toast action button.
///
/// @platforms =windows
/// @expand
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "NotificationActionPlacement")]
pub enum JsNotificationActionPlacement {
    /// `NotificationActionPlacement.ContextMenu`
    ContextMenu,
}

impl From<JsNotificationActionPlacement> for NotificationActionPlacement {
    fn from(value: JsNotificationActionPlacement) -> Self {
        match value {
            JsNotificationActionPlacement::ContextMenu => Self::ContextMenu,
        }
    }
}

/// Style of a toast action button.
///
/// @platforms =windows
/// @expand
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "NotificationButtonStyle")]
pub enum JsNotificationButtonStyle {
    /// `NotificationButtonStyle.Success`
    Success,
    /// `NotificationButtonStyle.Critical`
    Critical,
}

impl From<JsNotificationButtonStyle> for NotificationButtonStyle {
    fn from(value: JsNotificationButtonStyle) -> Self {
        match value {
            JsNotificationButtonStyle::Success => Self::Success,
            JsNotificationButtonStyle::Critical => Self::Critical,
        }
    }
}

/// Input type for toast input fields.
///
/// @platforms =windows
/// @expand
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    IntoSerde,
    PartialEq,
    Serialize,
)]
#[serde(rename = "NotificationInputType")]
pub enum JsNotificationInputType {
    /// `NotificationInputType.Text`
    Text,
    /// `NotificationInputType.Selection`
    Selection,
}

impl From<JsNotificationInputType> for NotificationInputType {
    fn from(value: JsNotificationInputType) -> Self {
        match value {
            JsNotificationInputType::Text => Self::Text,
            JsNotificationInputType::Selection => Self::Selection,
        }
    }
}

/// A custom string hint for Linux notifications.
///
/// @platforms =linux
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsNotificationCustomHint {
    /// Hint name.
    pub name: String,
    /// Hint value.
    pub value: String,
}

impl From<JsNotificationCustomHint> for NotificationCustomHint {
    fn from(value: JsNotificationCustomHint) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

/// A custom integer hint for Linux notifications.
///
/// @platforms =linux
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsNotificationCustomIntHint {
    /// Hint name.
    pub name: String,
    /// Integer hint value.
    pub value: i32,
}

impl From<JsNotificationCustomIntHint> for NotificationCustomIntHint {
    fn from(value: JsNotificationCustomIntHint) -> Self {
        Self {
            name: value.name,
            value: value.value,
        }
    }
}

/// A notification action button.
///
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsNotificationAction {
    /// Action identifier (used as arguments on Windows).
    pub identifier: String,
    /// Action label visible to the user.
    pub label: String,
    /// Action type string (Windows-specific, e.g. for protocol activation).
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub action_type: Option<String>,
    /// Activation type for this action.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub activation_type: Option<JsNotificationActivationType>,
    /// Placement of this action button.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub placement: Option<JsNotificationActionPlacement>,
    /// Visual style of the button.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub button_style: Option<JsNotificationButtonStyle>,
    /// ID of the input element this action is associated with.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub input_id: Option<String>,
}

impl From<JsNotificationAction> for NotificationAction {
    fn from(value: JsNotificationAction) -> Self {
        Self {
            identifier: value.identifier,
            label: value.label,
            action_type: value.action_type,
            activation_type: value.activation_type.map(Into::into),
            placement: value.placement.map(Into::into),
            button_style: value.button_style.map(Into::into),
            input_id: value.input_id,
        }
    }
}

/// A notification header for grouping toasts in Action Center.
///
/// @platforms =windows
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsNotificationHeader {
    /// Unique identifier for this header group.
    pub id: String,
    /// Title displayed for the header group.
    pub title: String,
    /// Arguments passed when the header is clicked.
    pub arguments: String,
}

impl From<JsNotificationHeader> for NotificationHeader {
    fn from(value: JsNotificationHeader) -> Self {
        Self {
            id: value.id,
            title: value.title,
            arguments: value.arguments,
        }
    }
}

/// A toast input field.
///
/// @platforms =windows
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsNotificationInput {
    /// Unique identifier for this input.
    pub id: String,
    /// Type of input field.
    pub input_type: Option<JsNotificationInputType>,
    /// Placeholder text shown when the input is empty.
    ///
    /// @default `undefined`
    pub placeholder: Option<String>,
    /// Title displayed above the input.
    ///
    /// @default `undefined`
    pub title: Option<String>,
    /// Default value for the input.
    ///
    /// @default `undefined`
    pub default_input: Option<String>,
}

impl From<JsNotificationInput> for NotificationInput {
    fn from(value: JsNotificationInput) -> Self {
        Self {
            id: value.id,
            input_type: value
                .input_type
                .map(Into::into)
                .unwrap_or(NotificationInputType::Text),
            placeholder: value.placeholder,
            title: value.title,
            default_input: value.default_input,
        }
    }
}

/// A selection option for a dropdown input.
///
/// @platforms =windows
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsNotificationSelection {
    /// Unique identifier for this selection option.
    pub id: String,
    /// Display text for this selection option.
    pub content: String,
}

impl From<JsNotificationSelection> for NotificationSelection {
    fn from(value: JsNotificationSelection) -> Self {
        Self {
            id: value.id,
            content: value.content,
        }
    }
}

/// Options for a notification.
///
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsNotificationOptions {
    /// Title of the notification (summary line).
    ///
    /// @default `undefined`
    pub title: Option<String>,

    /// Application name, filled by default with executable name.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub app_name: Option<String>,

    /// Body text of the notification.
    /// Multiple lines possible, may support simple markup.
    /// On Linux, check `notification.capabilities()` for a list.
    ///
    /// @default `undefined`
    pub body: Option<String>,

    /// Icon name/path assigned to the notification icon field.
    /// Usually available in /usr/share/icons.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub icon_name: Option<String>,

    /// Whether to set the icon automatically from executable name.
    ///
    /// @platforms =linux
    /// @default `false`
    pub auto_icon: bool,

    /// Icon image to display with the notification.
    ///
    /// @default `undefined`
    pub icon: Option<JsImage>,

    /// Timeout before the notification is automatically dismissed.
    /// Note that most servers don't respect this setting.
    ///
    /// @default `undefined`
    pub timeout: Option<JsDuration>,

    /// If `true`, action identifiers may be interpreted as icon names.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub action_icons: Option<bool>,

    /// Notification category such as `email`, `im`, or `device`.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub category: Option<String>,

    /// Desktop entry id (usually app `.desktop` name without extension).
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub desktop_entry: Option<String>,

    /// If `true`, keep notification resident until explicitly dismissed.
    /// Also automatically sets the timeout to never expire unless an explicit
    /// timeout is provided.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub resident: Option<bool>,

    /// Absolute path to a sound file to play for this notification.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub sound_file: Option<String>,

    /// Themeable freedesktop sound name, e.g. `message-new-instant`.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub sound_name: Option<String>,

    /// If `true`, suppress notification sounds.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub suppress_sound: Option<bool>,

    /// If `true`, request non-persistent behavior from the server.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub transient: Option<bool>,

    /// Target screen position for the notification.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub point: Option<JsPoint>,

    /// Urgency level.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub urgency: Option<JsNotificationUrgency>,

    /// Custom string key/value pairs forwarded as-is.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub custom_hints: Option<Vec<JsNotificationCustomHint>>,

    /// Custom integer key/value pairs forwarded as-is.
    ///
    /// @platforms =linux
    /// @default `undefined`
    pub custom_int_hints: Option<Vec<JsNotificationCustomIntHint>>,

    /// Notification actions.
    ///
    /// @default `undefined`
    pub actions: Option<Vec<JsNotificationAction>>,

    /// Attribution text displayed at the bottom of the notification.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub attribution_text: Option<String>,

    /// Hero image displayed prominently at the top of the notification.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub hero_image: Option<JsImage>,

    /// Whether to crop the icon into a circle.
    ///
    /// @platforms =windows
    /// @default `false`
    pub icon_crop_circle: bool,

    /// Toast scenario that adjusts notification behavior.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub scenario: Option<JsNotificationScenario>,

    /// Sound to play with the notification.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub sound: Option<JsNotificationSound>,

    /// Whether to loop the notification sound.
    ///
    /// @platforms =windows
    /// @default `false`
    pub sound_looping: bool,

    /// Whether to suppress all notification sound.
    ///
    /// @platforms =windows
    /// @default `false`
    pub silent: bool,

    /// Header for grouping notifications in Action Center.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub header: Option<JsNotificationHeader>,

    /// Input fields displayed in the notification.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub inputs: Option<Vec<JsNotificationInput>>,

    /// Selection options for dropdown inputs.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub selections: Option<Vec<JsNotificationSelection>>,

    /// Tag for identifying and replacing notifications.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub tag: Option<String>,

    /// Group identifier for organizing notifications.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub group: Option<String>,

    /// Remote ID for cross-device notification correlation.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub remote_id: Option<String>,

    /// Launch string passed to the app when the notification is clicked.
    ///
    /// @platforms =windows
    /// @default `undefined`
    pub launch: Option<String>,

    /// Whether to enable button styling on actions.
    ///
    /// @platforms =windows
    /// @default `false`
    pub use_button_style: bool,
}

impl From<JsNotificationOptions> for NotificationOptions {
    fn from(value: JsNotificationOptions) -> Self {
        Self {
            title: value.title,
            app_name: value.app_name,
            body: value.body,
            icon_name: value.icon_name,
            auto_icon: value.auto_icon,
            icon: value.icon.map(|i| i.into_inner()),
            timeout: value.timeout.map(Into::into),
            action_icons: value.action_icons,
            category: value.category,
            desktop_entry: value.desktop_entry,
            resident: value.resident,
            sound_file: value.sound_file,
            sound_name: value.sound_name,
            suppress_sound: value.suppress_sound,
            transient: value.transient,
            point: value.point.map(Into::into),
            urgency: value.urgency.map(Into::into),
            custom_hints: value
                .custom_hints
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
            custom_int_hints: value
                .custom_int_hints
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
            actions: value
                .actions
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
            attribution_text: value.attribution_text,
            hero_image: value.hero_image.map(|i| i.into_inner()),
            icon_crop_circle: value.icon_crop_circle,
            scenario: value.scenario.map(Into::into),
            sound: value.sound.map(Into::into),
            sound_looping: value.sound_looping,
            silent: value.silent,
            header: value.header.map(Into::into),
            inputs: value
                .inputs
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
            selections: value
                .selections
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
            tag: value.tag,
            group: value.group,
            remote_id: value.remote_id,
            launch: value.launch,
            use_button_style: value.use_button_style,
        }
    }
}

/// The global notification singleton for sending desktop notifications.
/// @singleton
#[derive(JsLifetime)]
#[rquickjs::class(rename = "Notification")]
pub struct JsNotification {
    inner: super::Notification,
}

impl<'js> Trace<'js> for JsNotification {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> SingletonClass<'js> for JsNotification {
    fn register_dependencies(ctx: &Ctx<'js>) -> rquickjs::Result<()> {
        register_enum::<JsNotificationUrgency>(ctx)?;
        register_enum::<JsNotificationScenario>(ctx)?;
        register_enum::<JsNotificationSound>(ctx)?;
        register_enum::<JsNotificationActivationType>(ctx)?;
        register_enum::<JsNotificationActionPlacement>(ctx)?;
        register_enum::<JsNotificationButtonStyle>(ctx)?;
        register_enum::<JsNotificationInputType>(ctx)?;
        register_host_class::<JsNotificationHandle>(ctx)?;
        Ok(())
    }
}

impl JsNotification {
    /// @skip
    #[must_use]
    pub const fn new(task_tracker: TaskTracker) -> Self {
        Self {
            inner: super::Notification::new(task_tracker),
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsNotification {
    /// Shows a desktop notification.
    pub async fn show(
        &self,
        ctx: Ctx<'_>,
        options: Opt<JsNotificationOptions>,
    ) -> Result<JsNotificationHandle> {
        let options = options.0.unwrap_or_default();
        let notification_options = options.into();
        self.inner
            .show(notification_options)
            .await
            .map(JsNotificationHandle::new)
            .into_js_result(&ctx)
    }

    /// Server capabilities.
    ///
    /// @platforms =linux
    pub fn capabilities(&self, ctx: Ctx<'_>) -> Result<Vec<String>> {
        super::Notification::capabilities().into_js_result(&ctx)
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Notification", &self.inner)
    }
}

/// Options for waiting on a notification action or close event.
///
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsWaitForActionOptions {
    /// Abort signal to cancel waiting.
    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,
}

/// A handle for a shown desktop notification.
#[derive(JsLifetime)]
#[rquickjs::class(rename = "NotificationHandle")]
pub struct JsNotificationHandle {
    inner: Mutex<Option<Arc<super::NotificationHandle>>>,
}

impl JsNotificationHandle {
    #[must_use]
    fn new(inner: super::NotificationHandle) -> Self {
        Self {
            inner: Mutex::new(Some(Arc::new(inner))),
        }
    }

    fn take_handle(&self, ctx: &Ctx<'_>) -> rquickjs::Result<super::NotificationHandle> {
        let mut inner = self.inner.lock();
        let handle = inner.take().ok_or_else(|| {
            Exception::throw_message(
                ctx,
                "the notification handle has already been consumed by waitForAction or waitUntilClosed",
            )
        })?;

        match Arc::try_unwrap(handle) {
            Ok(handle) => Ok(handle),
            Err(handle) => {
                *inner = Some(handle);
                Err(Exception::throw_message(
                    ctx,
                    "cannot consume the notification handle while an update is in progress",
                ))
            }
        }
    }
}

impl<'js> HostClass<'js> for JsNotificationHandle {}

impl<'js> Trace<'js> for JsNotificationHandle {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsNotificationHandle {
    /// Programmatically closes the notification.
    ///
    /// ```ts
    /// const handle = await notification.show({ title: "Hello", resident: true });
    /// await sleep("5s");
    /// await handle.close();
    /// ```
    pub async fn close(&self, ctx: Ctx<'_>) -> Result<()> {
        let handle = self.take_handle(&ctx)?;
        handle.close().await.into_js_result(&ctx)
    }

    /// Updates the notification with new options.
    ///
    /// ```ts
    /// const handle = await notification.show({ title: "Initial" });
    /// await handle.update({ title: "Updated", body: "New body" });
    /// ```
    ///
    /// @platforms =linux
    pub async fn update<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsNotificationOptions>,
    ) -> Result<()> {
        let handle = self.inner.lock().as_ref().cloned().ok_or_else(|| {
            Exception::throw_message(
                &ctx,
                "cannot update: the notification handle has already been consumed",
            )
        })?;
        let options: super::NotificationOptions = options.0.unwrap_or_default().into();

        handle.update(options).await.into_js_result(&ctx)
    }

    /// Waits for an action to be invoked on this notification, or for the notification to close.
    /// Returns the action identifier, or `null` if the notification was closed without an action.
    ///
    /// ```ts
    /// const handle = await notification.show({ title: "Update available", actions: [{ identifier: "update", label: "Update now" }] });
    /// const action = await handle.waitForAction();
    /// if (action === "update") { await runUpdate(); }
    /// ```
    ///
    /// @returns Task<string | null>
    pub fn wait_for_action<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsWaitForActionOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal;
        let task_tracker = ctx.user_data().task_tracker();
        let handle = self.take_handle(&ctx)?;

        task_with_token(ctx, signal, async move |ctx, token| {
            handle
                .wait_for_action(token, task_tracker)
                .await
                .into_js_result(&ctx)
        })
    }

    /// Waits until this notification is closed or the optional abort signal is triggered.
    ///
    /// ```ts
    /// const handle = await notification.show({ title: "Waiting..." });
    /// await handle.waitUntilClosed();
    /// ```
    ///
    /// @returns Task<void>
    pub fn wait_until_closed<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsWaitForActionOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal;
        let task_tracker = ctx.user_data().task_tracker();
        let handle = self.take_handle(&ctx)?;

        task_with_token(ctx, signal, async move |ctx, token| {
            handle
                .wait_for_action(token, task_tracker)
                .await
                .map(|_| ())
                .into_js_result(&ctx)
        })
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        let inner = self.inner.lock();
        inner.as_deref().map_or_else(
            || display_with_type("NotificationHandle", "(consumed)"),
            |handle| display_with_type("NotificationHandle", handle),
        )
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::runtime::Runtime;

    #[test]
    #[traced_test]
    #[ignore]
    fn test_show() {
        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine
                .eval_async::<()>(
                    r#"
                    await notification.show({ title: "Test Notification", body: "This is a test" });
                    "#,
                )
                .await
                .unwrap();
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    #[cfg(unix)]
    fn test_update() {
        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine
                .eval_async::<()>(
                    r#"
                    let handle = await notification.show({ title: "Test Notification", body: "This is a test" });
                    await sleep("2s");
                    await handle.update({ title: "Test Notification", body: "This is another test"});
                    "#,
                )
                .await
                .unwrap();
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_closed() {
        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine
                .eval_async::<()>(
                    r#"
                    let handle = await notification.show({ title: "Test Notification", body: "This is a test" });
                    await handle.waitUntilClosed();
                    "#,
                )
                .await
                .unwrap();
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_close() {
        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine
                .eval_async::<()>(
                    r#"
                    let handle = await notification.show({ title: "Closing soon", body: "This will be closed programmatically", resident: true });
                    await sleep("2s");
                    await handle.close();
                    "#,
                )
                .await
                .unwrap();
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_action() {
        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine
                .eval_async::<()>(
                    r#"
                    const handle = await notification.show({ title: "Update available", actions: [{ identifier: "update", label: "Update now" }, { identifier: "no_nothing", label: "Do nothing" }] });
                    const action = await handle.waitForAction();
                    println(action);
                    "#,
                )
                .await
                .unwrap();
        });
    }
}
