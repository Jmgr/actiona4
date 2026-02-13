use macros::FromJsObject;
use parking_lot::Mutex;
use rquickjs::{
    Ctx, Exception, JsLifetime, Promise, Result,
    class::{Trace, Tracer},
    prelude::*,
};

use crate::{
    api::{
        image::js::JsImage,
        js::{
            abort_controller::JsAbortSignal,
            classes::{HostClass, SingletonClass, register_host_class},
            duration::JsDuration,
            task::task_with_token,
        },
    },
    runtime::WithUserData,
};

use super::NotificationOptions;

/// Options for showing a notification.
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsShowNotificationOptions {
    /// Body text of the notification.
    /// @default `undefined`
    pub body: Option<String>,

    /// Icon image to display with the notification.
    /// @default `undefined`
    pub icon: Option<JsImage>,

    /// Timeout before the notification is automatically dismissed.
    /// @default `undefined`
    pub timeout: Option<JsDuration>,
}

impl From<JsShowNotificationOptions> for NotificationOptions {
    fn from(value: JsShowNotificationOptions) -> Self {
        Self {
            body: value.body,
            icon: value.icon.map(|i| i.into_inner()),
            timeout: value.timeout.map(Into::into),
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
        text: String,
        options: Opt<JsShowNotificationOptions>,
    ) -> Result<JsNotificationHandle> {
        let options = options.0.unwrap_or_default();
        let notification_options = options.into();
        self.inner
            .show(&text, notification_options)
            .await
            .map(JsNotificationHandle::new)
            .map_err(|err| Exception::throw_message(&ctx, &err.to_string()))
    }
}

/// Options for waiting on a shown notification.
///
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsWaitUntilClosedOptions {
    /// Abort signal to cancel waiting for the notification to close.
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
}

impl<'js> HostClass<'js> for JsNotificationHandle {}

impl<'js> Trace<'js> for JsNotificationHandle {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsNotificationHandle {
    /// Waits until this notification is closed or the optional abort signal is triggered.
    ///
    /// ```ts
    /// const handle = await notification.showHandle("Waiting...");
    /// await handle.waitUntilClosed();
    /// ```
    ///
    /// @returns Task<void>
    pub fn wait_until_closed<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsWaitUntilClosedOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let task_tracker = ctx.user_data().task_tracker();
        let handle = self.inner.lock().take().ok_or_else(|| {
            Exception::throw_message(
                &ctx,
                "waitUntilClosed has already been called for this notification handle",
            )
        })?;

        task_with_token(ctx, signal, async move |ctx, token| {
            handle
                .wait_until_closed(token, task_tracker)
                .await
                .map_err(|err| Exception::throw_message(&ctx, &err.to_string()))
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
                    notification.show("Test Notification", { body: "This is a test", timeout: 3 });
                    "#,
                )
                .await
                .unwrap();
        });
    }
}
