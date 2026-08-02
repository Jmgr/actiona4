use std::sync::Arc;

use actiona_common::sentry::setup_crash_reporting;
use color_eyre::Result;
use editor::rpc::api_serve;
#[cfg(not(debug_assertions))]
use include_dir::include_dir;
use tokio::runtime::{Handle as TokioHandle, Runtime as TokioRuntime};
use webview_app::{application::Application, request, webview::WebView};

use crate::rpc_host::{Events, HostApi};

mod rpc_host;

#[allow(clippy::needless_raw_strings)]
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn create_webview(app: &Application, tokio_handle: TokioHandle) -> WebView {
    let builder = WebView::builder(app)
        .title("Actiona 4")
        .initial_bounds(1500, 1000);

    #[cfg(debug_assertions)]
    let builder = builder
        .url("http://127.0.0.1:8080")
        .devtools(true)
        .console_logging(true);

    #[cfg(not(debug_assertions))]
    let builder = builder
        .webroot(include_dir!("$CARGO_MANIFEST_DIR/dist"))
        .default_contextmenu_disabled();

    let webview = builder.build();

    connect_host_bridge(&webview, tokio_handle);

    webview
}

/// Route every webview request through the typed [`Api`] dispatcher.
///
/// `api_serve` decides synchronously whether the command is known (returning
/// `None` → "not handled here"); the actual async work runs on the tokio
/// runtime inside `request_blocking`'s worker thread, so a slow handler never
/// blocks the GTK main loop.
fn connect_host_bridge(webview: &WebView, tokio_handle: TokioHandle) {
    // The event sink pushes host → UI notifications over the same webview; the
    // api borrows it so handlers can emit events while serving requests.
    let events = Arc::new(Events::new(webview.get_handle()));
    let api = Arc::new(HostApi::new(events));

    webview.connect_request(move |req, id, cmd, json| {
        if cmd == "exit" {
            req.quit_application();
            return true;
        }

        let input = serde_json::from_str(&json).unwrap_or(serde_json::Value::Null);
        match api_serve(api.clone(), &cmd, input) {
            Some(fut) => {
                let tokio_handle = tokio_handle.clone();
                request::request_blocking(req, id, move || {
                    serde_json::to_string(&tokio_handle.block_on(fut))
                        .expect("RPC: failed to serialize response")
                });
                true
            }
            None => false,
        }
    });
}

fn main() -> Result<()> {
    let _guard = setup_crash_reporting(built_info::PKG_NAME)?;

    let runtime = TokioRuntime::new().expect("failed to start tokio runtime");
    let handle = runtime.handle().clone();

    Application::new("app.actiona.editor")
        .on_activate(move |app| create_webview(app, handle.clone()))
        .run();

    Ok(())
}
