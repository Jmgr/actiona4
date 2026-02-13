use tokio::{select, sync::oneshot};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use winrt_toast_reborn::{Toast, ToastDuration, ToastManager};

use crate::api::notification::{NotificationOptions, Result, save_image_to_temp};

const AUMID: &str = "app.actiona.actiona4-run";

pub struct NotificationHandle {
    receiver: oneshot::Receiver<()>,
}

fn build_toast(options: &NotificationOptions) -> Result<Toast> {
    let mut toast = Toast::new();
    toast.text1(options.title.as_deref().unwrap_or_default());

    if let Some(body) = &options.body {
        toast.text2(body.as_str());
    }

    if let Some(icon) = &options.icon {
        let path = save_image_to_temp(icon)?;
        let image = winrt_toast_reborn::Image::new_local(path)
            .with_placement(winrt_toast_reborn::ImagePlacement::AppLogoOverride);
        toast.image(1, image);
    }

    if let Some(timeout) = options.timeout {
        if timeout.as_secs() > 7 {
            toast.duration(ToastDuration::Long);
        }
        toast.expires_in(*timeout);
    }

    Ok(toast)
}

pub async fn show(options: NotificationOptions) -> Result<()> {
    show_with_handle(options).await?;
    Ok(())
}

pub async fn show_and_wait(
    options: NotificationOptions,
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
) -> Result<()> {
    let handle = show_with_handle(options).await?;
    wait_until_closed(handle, cancellation_token, task_tracker).await
}

pub async fn show_with_handle(options: NotificationOptions) -> Result<NotificationHandle> {
    let toast = build_toast(&options)?;

    let (tx, rx) = oneshot::channel::<()>();
    let tx = std::sync::Mutex::new(Some(tx));

    let manager = ToastManager::new(AUMID).on_dismissed(move |_| {
        if let Some(tx) = tx.lock().unwrap().take() {
            let _ = tx.send(());
        }
    });

    manager.show(&toast)?;

    Ok(NotificationHandle { receiver: rx })
}

pub async fn wait_until_closed(
    mut handle: NotificationHandle,
    cancellation_token: CancellationToken,
    _task_tracker: TaskTracker,
) -> Result<()> {
    select! {
        _ = &mut handle.receiver => {},
        _ = cancellation_token.cancelled() => {},
    }

    Ok(())
}
