use std::sync::Once;

use tracing::warn;

use crate::api::notification::platform::AUMID;

pub fn ensure_notification_registration() {
    const DISPLAY_NAME: &str = "Actiona";

    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        let result = winrt_toast_reborn::register(AUMID, DISPLAY_NAME, None::<&std::path::Path>);

        if let Err(e) = result {
            warn!("Could not register notification app: {e}");
        }
    });
}
