use std::sync::Arc;

use parking_lot::Mutex;
use tokio::{select, sync::oneshot};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use winrt_toast_reborn::{
    Action, Audio, Header, Input, Selection, Toast, ToastDuration, ToastManager,
    content::{
        action::{ActionPlacement, ActivationType, HintButtonStyle},
        audio::{LoopingSound, Sound},
        image::{ImageHintCrop, ImagePlacement},
        input::InputType,
        text::TextPlacement,
    },
};

use crate::api::notification::{
    NotificationActionPlacement, NotificationActivationType, NotificationButtonStyle,
    NotificationInputType, NotificationOptions, NotificationScenario, NotificationSound, Result,
};

pub const AUMID: &str = "app.actiona.actiona-run";

#[derive(Default)]
pub struct Notification;

impl Notification {
    pub async fn show(&self, options: NotificationOptions) -> Result<NotificationHandle> {
        let toast = build_toast(&options)?;

        let (tx, rx) = oneshot::channel::<Option<String>>();
        let tx = Arc::new(Mutex::new(Some(tx)));

        let tx_activated = tx.clone();
        let tx_dismissed = tx;

        let manager = ToastManager::new(AUMID)
            .on_activated(None, move |activated| {
                let value = tx_activated.lock().take();
                if let Some(tx) = value {
                    let action = activated.map(|a| a.arg);
                    let _ = tx.send(action);
                }
            })
            .on_dismissed(move |_| {
                let value = tx_dismissed.lock().take();
                if let Some(tx) = value {
                    let _ = tx.send(None);
                }
            });

        manager.show(&toast)?;

        Ok(NotificationHandle {
            receiver: Arc::new(Mutex::new(Some(rx))),
        })
    }

    pub const fn capabilities() -> Result<Vec<String>> {
        Ok(Vec::new())
    }
}

pub struct NotificationHandle {
    receiver: Arc<Mutex<Option<oneshot::Receiver<Option<String>>>>>,
}

impl NotificationHandle {
    pub async fn update(&self, _options: NotificationOptions) -> Result<()> {
        // Windows Toast notifications do not support in-place updates.
        Ok(())
    }

    pub async fn wait_for_action(
        self,
        cancellation_token: CancellationToken,
        _task_tracker: TaskTracker,
    ) -> Result<Option<String>> {
        let receiver = self
            .receiver
            .lock()
            .take()
            .expect("wait_for_action should only be called once");

        select! {
            result = receiver => Ok(result.unwrap_or(None)),
            _ = cancellation_token.cancelled() => Ok(None),
        }
    }
}

fn build_toast(options: &NotificationOptions) -> Result<Toast> {
    let mut toast = Toast::new();
    toast.text1(options.title.as_deref().unwrap_or_default());

    if let Some(body) = &options.body {
        toast.text2(body.as_str());
    }

    if let Some(attribution) = &options.attribution_text {
        toast.text3(
            winrt_toast_reborn::Text::new(attribution.as_str())
                .with_placement(TextPlacement::Attribution),
        );
    }

    if let Some(icon) = &options.icon {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("actiona4_notification_icon.png");
        icon.as_rgba8().save(&path)?;

        let mut image = winrt_toast_reborn::Image::new_local(path)?
            .with_placement(ImagePlacement::AppLogoOverride);
        if options.icon_crop_circle {
            image = image.with_hint_crop(ImageHintCrop::Circle);
        }
        toast.image(1, image);
    }

    if let Some(hero) = &options.hero_image {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("actiona4_notification_hero.png");
        hero.as_rgba8().save(&path)?;

        let image =
            winrt_toast_reborn::Image::new_local(path)?.with_placement(ImagePlacement::Hero);
        toast.image(2, image);
    }

    if let Some(timeout) = options.timeout {
        if timeout.as_secs() > 7 {
            toast.duration(ToastDuration::Long);
        }
        toast.expires_in(timeout);
    }

    if let Some(scenario) = &options.scenario {
        toast.scenario(match scenario {
            NotificationScenario::Reminder => winrt_toast_reborn::Scenario::Reminder,
            NotificationScenario::Alarm => winrt_toast_reborn::Scenario::Alarm,
            NotificationScenario::IncomingCall => winrt_toast_reborn::Scenario::IncomingCall,
            NotificationScenario::Urgent => winrt_toast_reborn::Scenario::Urgent,
        });
    }

    if let Some(sound) = &options.sound {
        let src = match sound {
            NotificationSound::Default => Sound::Default,
            NotificationSound::IM => Sound::IM,
            NotificationSound::Mail => Sound::Mail,
            NotificationSound::Reminder => Sound::Reminder,
            NotificationSound::SMS => Sound::SMS,
            NotificationSound::None => Sound::None,
            NotificationSound::LoopingAlarm => Sound::Looping(LoopingSound::Alarm),
            NotificationSound::LoopingAlarm2 => Sound::Looping(LoopingSound::Alarm2),
            NotificationSound::LoopingAlarm3 => Sound::Looping(LoopingSound::Alarm3),
            NotificationSound::LoopingAlarm4 => Sound::Looping(LoopingSound::Alarm4),
            NotificationSound::LoopingAlarm5 => Sound::Looping(LoopingSound::Alarm5),
            NotificationSound::LoopingAlarm6 => Sound::Looping(LoopingSound::Alarm6),
            NotificationSound::LoopingAlarm7 => Sound::Looping(LoopingSound::Alarm7),
            NotificationSound::LoopingAlarm8 => Sound::Looping(LoopingSound::Alarm8),
            NotificationSound::LoopingAlarm9 => Sound::Looping(LoopingSound::Alarm9),
            NotificationSound::LoopingAlarm10 => Sound::Looping(LoopingSound::Alarm10),
            NotificationSound::LoopingCall => Sound::Looping(LoopingSound::Call),
            NotificationSound::LoopingCall2 => Sound::Looping(LoopingSound::Call2),
            NotificationSound::LoopingCall3 => Sound::Looping(LoopingSound::Call3),
            NotificationSound::LoopingCall4 => Sound::Looping(LoopingSound::Call4),
            NotificationSound::LoopingCall5 => Sound::Looping(LoopingSound::Call5),
            NotificationSound::LoopingCall6 => Sound::Looping(LoopingSound::Call6),
            NotificationSound::LoopingCall7 => Sound::Looping(LoopingSound::Call7),
            NotificationSound::LoopingCall8 => Sound::Looping(LoopingSound::Call8),
            NotificationSound::LoopingCall9 => Sound::Looping(LoopingSound::Call9),
            NotificationSound::LoopingCall10 => Sound::Looping(LoopingSound::Call10),
        };
        let mut audio = Audio::new(src);
        if options.sound_looping {
            audio = audio.with_looping();
        }
        if options.silent {
            audio = audio.with_silent();
        }
        toast.audio(audio);
    } else if options.silent {
        toast.audio(Audio::new(Sound::None).with_silent());
    }

    if let Some(header) = &options.header {
        toast.header(Header::new(
            header.id.as_str(),
            header.title.as_str(),
            header.arguments.as_str(),
        ));
    }

    for action in &options.actions {
        let mut a = Action::new(
            action.label.as_str(),
            action.identifier.as_str(),
            action.action_type.as_deref().unwrap_or(""),
        );
        if let Some(at) = &action.activation_type {
            a = a.with_activation_type(match at {
                NotificationActivationType::Foreground => ActivationType::Foreground,
                NotificationActivationType::Background => ActivationType::Background,
                NotificationActivationType::Protocol => ActivationType::Protocol,
            });
        }
        if let Some(p) = &action.placement {
            a = a.with_placement(match p {
                NotificationActionPlacement::ContextMenu => ActionPlacement::ContextMenu,
            });
        }
        if let Some(bs) = &action.button_style {
            a = a.with_button_style(match bs {
                NotificationButtonStyle::Success => HintButtonStyle::Success,
                NotificationButtonStyle::Critical => HintButtonStyle::Critical,
            });
        }
        if let Some(input_id) = &action.input_id {
            a = a.with_input_id(input_id.as_str());
        }
        toast.action(a);
    }

    for input in &options.inputs {
        let input_type = match input.input_type {
            NotificationInputType::Text => InputType::Text,
            NotificationInputType::Selection => InputType::Selection,
        };
        let mut i = Input::new(input.id.as_str(), input_type);
        if let Some(placeholder) = &input.placeholder {
            i = i.with_placeholder(placeholder.as_str());
        }
        if let Some(title) = &input.title {
            i = i.with_title(title.as_str());
        }
        if let Some(default_input) = &input.default_input {
            i = i.with_default_input(default_input.as_str());
        }
        toast.input(i);
    }

    for selection in &options.selections {
        toast.selection(Selection::new(
            selection.id.as_str(),
            selection.content.as_str(),
        ));
    }

    if let Some(tag) = &options.tag {
        toast.tag(tag.as_str());
    }
    if let Some(group) = &options.group {
        toast.group(group.as_str());
    }
    if let Some(remote_id) = &options.remote_id {
        toast.remote_id(remote_id.as_str());
    }
    if let Some(launch) = &options.launch {
        toast.launch(launch.as_str());
    }
    if options.use_button_style {
        toast.use_button_style();
    }

    Ok(toast)
}
