use std::sync::OnceLock;

use eyre::{Result, eyre};

static REGISTRATION_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

pub fn register_notification_application(aumid: &str, display_name: &str) -> Result<()> {
    winrt_toast_reborn::register(aumid, display_name, None).map_err(|error| eyre!(error))
}

pub fn unregister_notification_application(aumid: &str) -> Result<()> {
    match winrt_toast_reborn::unregister(aumid) {
        Ok(()) => Ok(()),
        Err(winrt_toast_reborn::WinToastError::Io(error))
            if error.kind() == std::io::ErrorKind::NotFound =>
        {
            Ok(())
        }
        Err(error) => Err(eyre!(error)),
    }
}

pub fn ensure_notification_registration(aumid: &str, display_name: &str) -> Result<()> {
    match REGISTRATION_RESULT.get_or_init(|| {
        register_notification_application(aumid, display_name).map_err(|error| error.to_string())
    }) {
        Ok(()) => Ok(()),
        Err(error_message) => Err(eyre!(error_message.clone())),
    }
}
