use editor::{
    app::App,
    i18n::I18nProvider,
    rpc::{ApiClient, Transport},
};
use leptos::{mount::mount_to_body, prelude::*};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::spawn_local;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = WebView, js_name = request, catch)]
    async fn webview_request(method: &str, data: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = WebView, js_name = showDevTools)]
    fn webview_show_devtools();
}

/// The UI's [`Transport`]: carries the generated client's JSON over the webview
/// bridge. Constructing an [`ApiClient<WebviewTransport>`] then gives fully
/// typed host calls.
struct WebviewTransport;

impl Transport for WebviewTransport {
    type Error = String;

    async fn request(
        &self,
        method: &'static str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let input = serde_wasm_bindgen::to_value(&input).map_err(|e| e.to_string())?;
        let output = webview_request(method, input)
            .await
            .map_err(|e| format!("{e:?}"))?;
        serde_wasm_bindgen::from_value(output).map_err(|e| e.to_string())
    }
}

/// A typed client over the webview bridge. Cheap to construct because the
/// transport is a ZST, so call sites make one on demand.
fn host() -> ApiClient<WebviewTransport> {
    ApiClient::new(WebviewTransport)
}

/*
fn load_rows_from_host() {
    spawn_local(async move {
        let _ = host().load_rows().await;
    });
}

fn reload_ui() {
    if let Err(error) = web_sys::window()
        .expect("browser window is available")
        .location()
        .reload()
    {
        leptos::logging::error!("{error:?}");
    }
}

fn show_devtools() {
    unsafe {
        webview_show_devtools();
    }
}
*/

fn exit_app() {
    spawn_local(async move {
        let _ = host().exit().await;
    });
}

fn main() {
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! {
            <I18nProvider>
                <App on_exit=exit_app />
            </I18nProvider>
        }
    });
}
