/// Performs automatic platform-specific setup that should run on every invocation.
/// Currently this only does work on Windows (notification app registration).
pub fn ensure_platform_setup() {
    #[cfg(windows)]
    ensure_notification_registration();
}

#[cfg(windows)]
fn ensure_notification_registration() {
    const AUMID: &str = "app.actiona.actiona4-run";

    let result = winrt_toast_reborn::register(AUMID, "Actiona", None::<&str>);

    if let Err(e) = result {
        tracing::warn!("Could not register notification app: {e}");
    }
}
