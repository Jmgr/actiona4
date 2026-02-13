use macros::{FromJsObject, FromSerde, IntoSerde};
use parking_lot::Mutex;
use rquickjs::{
    Ctx, Exception, JsLifetime, Promise, Result,
    class::{Trace, Tracer},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

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
};

use super::{
    NotificationAction, NotificationCustomHint, NotificationCustomIntHint, NotificationOptions,
    NotificationUrgency,
};

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
    Low,
    Normal,
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

/// A custom string hint for Linux notifications.
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
    /// Action identifier.
    pub identifier: String,
    /// Action label visible to the user.
    pub label: String,
}

impl From<JsNotificationAction> for NotificationAction {
    fn from(value: JsNotificationAction) -> Self {
        Self {
            identifier: value.identifier,
            label: value.label,
        }
    }
}

/// Options for a notification.
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
    /// Check `notification.capabilities()` for a list.
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
    /// @platforms =linux
    /// @default `undefined`
    pub actions: Option<Vec<JsNotificationAction>>,
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
        register_host_class::<JsNotificationHandle>(ctx)?;
        Ok(())
    }
}

impl JsNotification {
    /// @skip
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: super::Notification::default(),
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
    pub async fn capabilities(&self, ctx: Ctx<'_>) -> Result<Vec<String>> {
        super::Notification::capabilities().into_js_result(&ctx)
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
    inner: Mutex<Option<super::NotificationHandle>>,
}

impl JsNotificationHandle {
    #[must_use]
    fn new(inner: super::NotificationHandle) -> Self {
        Self {
            inner: Mutex::new(Some(inner)),
        }
    }

    fn take_handle(&self, ctx: &Ctx<'_>) -> rquickjs::Result<super::NotificationHandle> {
        self.inner.lock().take().ok_or_else(|| {
            Exception::throw_message(
                ctx,
                "the notification handle has already been consumed by waitForAction or waitUntilClosed",
            )
        })
    }
}

impl<'js> HostClass<'js> for JsNotificationHandle {}

impl<'js> Trace<'js> for JsNotificationHandle {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsNotificationHandle {
    /// Updates the notification with new options.
    ///
    /// ```ts
    /// const handle = await notification.show({ title: "Initial" });
    /// await handle.update({ title: "Updated", body: "New body" });
    /// ```
    ///
    pub async fn update<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsNotificationOptions>,
    ) -> Result<()> {
        let inner = self.inner.lock();
        let handle = inner.as_ref().ok_or_else(|| {
            Exception::throw_message(
                &ctx,
                "cannot update: waitUntilClosed has already been called for this notification handle",
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
    /// if (action === "update") { /* ... */ }
    /// ```
    ///
    /// @returns Task<string | null>
    pub fn wait_for_action<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsWaitForActionOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
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
        let signal = options.signal.clone();
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
