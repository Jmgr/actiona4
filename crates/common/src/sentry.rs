use std::{panic, sync::Arc};

use color_eyre::{Result, eyre::Context};
use rfd::{MessageDialog, MessageDialogResult};
#[cfg(windows)]
use sentry::integrations::panic::PanicIntegration;
use sentry::{
    integrations::{
        backtrace::{AttachStacktraceIntegration, ProcessStacktraceIntegration},
        contexts::ContextIntegration,
        debug_images::DebugImagesIntegration,
    },
    protocol::Value,
};
use sentry_rust_minidump::Handle;

use crate::built_info;

const SENTRY_DSN: &str =
    "https://4d7d4abdc99f240244aaff1701358119@crash.actiona.app/5428144307680296";

pub struct CrashReportingGuard {
    _sentry: sentry::ClientInitGuard,
    _minidump: Arc<Handle>,
}

pub fn setup_crash_reporting(app_name: &str) -> Result<CrashReportingGuard> {
    let options = sentry::ClientOptions {
        release: sentry::release_name!(),
        auto_session_tracking: true,
        default_integrations: false,
        before_send: Some(Arc::new(move |mut event| {
            if event.message.is_none() {
                if let Some(Value::String(message)) = event.extra.get("panic.message") {
                    event.message = Some(format!("panic: {message}"));
                }
            }

            if event.culprit.is_none() {
                if let Some(Value::String(location)) = event.extra.get("panic.location") {
                    event.culprit = Some(location.clone());
                }
            }

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
    }
    .add_integration(AttachStacktraceIntegration::new())
    .add_integration(DebugImagesIntegration::new())
    .add_integration(ContextIntegration::new())
    .add_integration(ProcessStacktraceIntegration::new());

    #[cfg(windows)]
    let options = options.add_integration(PanicIntegration::new());

    let client = sentry::init((SENTRY_DSN, options));

    sentry::configure_scope(|scope| {
        scope.set_tag("app_name", app_name);

        if let Some(git_hash) = built_info::GIT_COMMIT_HASH {
            scope.set_tag("git_hash", git_hash);
        }
    });

    let minidump =
        Arc::new(sentry_rust_minidump::init(&client).context("starting crash reporter")?);
    install_abort_panic_metadata_hook(minidump.clone());

    Ok(CrashReportingGuard {
        _sentry: client,
        _minidump: minidump,
    })
}

fn install_abort_panic_metadata_hook(minidump: Arc<Handle>) {
    if !cfg!(panic = "abort") {
        return;
    }

    let previous_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|message| (*message).to_owned())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "panic payload is not a string".to_owned());

        minidump.set_extra("panic.message".to_owned(), Some(Value::String(message)));

        if let Some(location) = info.location() {
            minidump.set_extra(
                "panic.location".to_owned(),
                Some(Value::String(format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                ))),
            );
        }

        previous_hook(info);
    }));
}
