use std::sync::Arc;

use color_eyre::{Result, eyre::Context};
use rfd::{MessageDialog, MessageDialogResult};
use sentry_rust_minidump::Handle;

use crate::built_info;

const SENTRY_DSN: &str =
    "https://4d7d4abdc99f240244aaff1701358119@crash.actiona.app/5428144307680296";

pub struct CrashReportingGuard {
    _sentry: sentry::ClientInitGuard,
    _minidump: Handle,
}

pub fn setup_crash_reporting(app_name: &str) -> Result<CrashReportingGuard> {
    let client = sentry::init((
        SENTRY_DSN,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            auto_session_tracking: true,
            before_send: Some(Arc::new(move |event| {
                let dialog = MessageDialog::new()
                    .set_title("Send crash report?")
                    .set_description("Actiona just crashed, do you want to send the crash report?")
                    .set_level(rfd::MessageLevel::Warning)
                    .set_buttons(rfd::MessageButtons::YesNo);

                if dialog.show() == MessageDialogResult::Yes {
                    Some(event)
                } else {
                    None
                }
            })),
            ..Default::default()
        },
    ));

    sentry::configure_scope(|scope| {
        scope.set_tag("app_name", app_name);

        if let Some(git_hash) = built_info::GIT_COMMIT_HASH {
            scope.set_tag("git_hash", git_hash);
        }
    });

    let minidump = sentry_rust_minidump::init(&client).context("starting crash reporter")?;

    Ok(CrashReportingGuard {
        _sentry: client,
        _minidump: minidump,
    })
}
