use std::sync::Arc;

use color_eyre::eyre::eyre;
use notify_rust::{Hint, Urgency, get_capabilities};
use parking_lot::Mutex;
use tokio::sync::oneshot;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::{
    api::notification::{NotificationOptions, NotificationUrgency, Result},
    cancel_on,
};

#[derive(Default)]
pub struct Notification;

impl Notification {
    pub async fn show(&self, options: NotificationOptions) -> Result<NotificationHandle> {
        let notification = Self::build_notification(options)?;
        let inner = notification.show_async().await?;

        Ok(NotificationHandle {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    fn build_notification(options: NotificationOptions) -> Result<notify_rust::Notification> {
        let mut notification = notify_rust::Notification::new();
        notification.summary(options.title.as_deref().unwrap_or_default());

        if let Some(app_name) = &options.app_name {
            notification.appname(app_name);
        }

        if let Some(body) = &options.body {
            notification.body(body);
        }

        if let Some(icon_name) = &options.icon_name {
            notification.icon(icon_name);
        }

        if options.auto_icon {
            notification.auto_icon();
        }

        if let Some(action_icons) = options.action_icons {
            notification.hint(Hint::ActionIcons(action_icons));
        }

        if let Some(category) = &options.category {
            notification.hint(Hint::Category(category.clone()));
        }

        if let Some(desktop_entry) = &options.desktop_entry {
            notification.hint(Hint::DesktopEntry(desktop_entry.clone()));
        }

        if let Some(resident) = options.resident {
            notification.hint(Hint::Resident(resident));
            if resident {
                notification.timeout(notify_rust::Timeout::Never);
            }
        }

        if let Some(sound_file) = &options.sound_file {
            notification.hint(Hint::SoundFile(sound_file.clone()));
        }

        if let Some(sound_name) = &options.sound_name {
            notification.hint(Hint::SoundName(sound_name.clone()));
        }

        if let Some(suppress_sound) = options.suppress_sound {
            notification.hint(Hint::SuppressSound(suppress_sound));
        }

        notification.hint(Hint::Transient(options.transient.unwrap_or(true)));

        if let Some(point) = options.point {
            notification.hint(Hint::X(point.x.into()));
            notification.hint(Hint::Y(point.y.into()));
        }

        if let Some(urgency) = options.urgency {
            notification.urgency(match urgency {
                NotificationUrgency::Low => Urgency::Low,
                NotificationUrgency::Normal => Urgency::Normal,
                NotificationUrgency::Critical => Urgency::Critical,
            });
        }

        for hint in &options.custom_hints {
            notification.hint(Hint::Custom(hint.name.clone(), hint.value.clone()));
        }

        for hint in &options.custom_int_hints {
            notification.hint(Hint::CustomInt(hint.name.clone(), hint.value));
        }

        for action in &options.actions {
            notification.action(&action.identifier, &action.label);
        }

        if let Some(timeout) = options.timeout {
            notification.timeout(timeout);
        }

        if let Some(icon) = options.icon {
            let icon = notify_rust::Image::try_from(icon.into_rgba8())?;
            notification.image_data(icon);
        }

        Ok(notification)
    }

    pub fn capabilities() -> Result<Vec<String>> {
        get_capabilities().map_err(|err| eyre!("{err}"))
    }
}

pub struct NotificationHandle {
    inner: Arc<Mutex<notify_rust::NotificationHandle>>,
}

impl NotificationHandle {
    /// Programmatically close the notification.
    pub async fn close(self) -> Result<()> {
        let inner = Arc::into_inner(self.inner)
            .expect("update should not be running during close")
            .into_inner();

        tokio::task::spawn_blocking(move || {
            inner.close();
        })
        .await?;

        Ok(())
    }

    /// notify_rust's `update()` calls `block_on` internally, so it must run
    /// off the async runtime thread via `spawn_blocking`.
    pub async fn update(&self, options: NotificationOptions) -> Result<()> {
        let notification = Notification::build_notification(options)?;
        let inner = self.inner.clone();

        tokio::task::spawn_blocking(move || {
            let mut handle = inner.lock();
            **handle = notification;
            handle.update();
        })
        .await?;

        Ok(())
    }

    pub async fn wait_for_action(
        self,
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Result<Option<String>> {
        let (sender, receiver) = oneshot::channel::<Option<String>>();
        let sender = Mutex::new(Some(sender));
        let inner = Arc::into_inner(self.inner)
            .expect("update should not be running during wait_for_action")
            .into_inner();

        task_tracker.spawn_blocking(move || {
            inner.wait_for_action(|action| {
                let sender = sender.lock().take();
                if let Some(sender) = sender {
                    let value = match action {
                        "__closed" => None,
                        action => Some(action.to_owned()),
                    };
                    let _ = sender.send(value);
                }
            });
        });

        let action = cancel_on(&cancellation_token, receiver).await??;

        Ok(action)
    }
}

#[cfg(test)]
mod tests {
    use notify_rust::{Hint, Timeout};

    use super::Notification;
    use crate::api::{
        notification::{
            NotificationAction, NotificationCustomHint, NotificationCustomIntHint,
            NotificationOptions, NotificationUrgency,
        },
        point::point,
    };

    #[test]
    fn build_notification_sets_title_and_common_hints() {
        let options = NotificationOptions {
            title: Some("Configured title".to_owned()),
            body: Some("Configured body".to_owned()),
            action_icons: Some(true),
            category: Some("email".to_owned()),
            desktop_entry: Some("actiona4".to_owned()),
            resident: Some(true),
            sound_file: Some("/tmp/sound.ogg".to_owned()),
            sound_name: Some("message-new-instant".to_owned()),
            suppress_sound: Some(false),
            transient: Some(false),
            point: Some(point(12, 34)),
            urgency: Some(NotificationUrgency::Critical),
            custom_hints: vec![NotificationCustomHint {
                name: "x-actiona-key".to_owned(),
                value: "x-actiona-value".to_owned(),
            }],
            custom_int_hints: vec![NotificationCustomIntHint {
                name: "x-actiona-int".to_owned(),
                value: 42,
            }],
            actions: vec![NotificationAction {
                identifier: "open".to_owned(),
                label: "Open".to_owned(),
                action_type: None,
                activation_type: None,
                placement: None,
                button_style: None,
                input_id: None,
            }],
            ..NotificationOptions::default()
        };

        let notification = Notification::build_notification(options).unwrap();

        assert_eq!(notification.summary, "Configured title");
        assert_eq!(notification.body, "Configured body");
        assert_eq!(notification.actions, vec!["open", "Open"]);
        assert!(notification.hints.contains(&Hint::ActionIcons(true)));
        assert!(
            notification
                .hints
                .contains(&Hint::Category("email".to_owned()))
        );
        assert!(
            notification
                .hints
                .contains(&Hint::DesktopEntry("actiona4".to_owned()))
        );
        assert!(notification.hints.contains(&Hint::Resident(true)));
        assert!(
            notification
                .hints
                .contains(&Hint::SoundFile("/tmp/sound.ogg".to_owned()))
        );
        assert!(
            notification
                .hints
                .contains(&Hint::SoundName("message-new-instant".to_owned()))
        );
        assert!(notification.hints.contains(&Hint::SuppressSound(false)));
        assert!(notification.hints.contains(&Hint::Transient(false)));
        assert!(notification.hints.contains(&Hint::X(12)));
        assert!(notification.hints.contains(&Hint::Y(34)));
        assert_eq!(notification.timeout, Timeout::Never);
    }

    #[test]
    fn build_notification_uses_empty_title_and_default_transient_hint() {
        let options = NotificationOptions::default();

        let notification = Notification::build_notification(options).unwrap();

        assert_eq!(notification.summary, "");
        assert!(notification.hints.contains(&Hint::Transient(true)));
    }
}
