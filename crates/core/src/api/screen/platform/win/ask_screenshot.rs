use color_eyre::{Result, eyre::eyre};
use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions,
    tokio::{Listener, Stream, prelude::*},
};
use tokio::{io::AsyncReadExt, select};
use tokio_util::sync::CancellationToken;
use url::Url;
use uuid::Uuid;
use windows::ApplicationModel::DataTransfer::SharedStorageAccessManager;

use crate::api::image::Image;

const REDIRECT_SCHEME: &str = "actiona-run";
const REDIRECT_HOST: &str = "screenclip-response";

/// Socket name prefix; the correlation ID is appended to make it unique per
/// call.  Each `ask_screenshot` invocation owns its socket, so multiple
/// concurrent instances are routed correctly without any shared state.
const SOCKET_PREFIX: &str = "actiona-screenclip-";

/// Asks the user to interactively select a screen area via the Windows
/// Snipping Tool (`ms-screenclip:` URI scheme).
///
/// Launches the Snipping Tool capture overlay, waits for the user to complete
/// a snip, then redeems the shared-storage token to obtain the image.
///
/// Returns `Ok(None)` when the user cancels (HTTP-equivalent 499) or when
/// `cancellation` fires.
pub async fn ask_screenshot(cancellation: CancellationToken) -> Result<Option<Image>> {
    let correlation_id = Uuid::new_v4().to_string();
    let socket_name = format!("{SOCKET_PREFIX}{correlation_id}");

    let name = socket_name
        .as_str()
        .to_ns_name::<GenericNamespaced>()
        .map_err(|e| eyre!("invalid socket name: {e}"))?;

    // Create the listener *before* launching Snipping Tool so the relay
    // process can connect the moment it starts — no TOCTOU window.
    let listener = ListenerOptions::new()
        .name(name)
        .create_tokio()
        .map_err(|e| eyre!("failed to create callback socket: {e}"))?;

    // ms-screenclip URI: rectangle mode, all snipping modes enabled.
    let launch_uri = format!(
        "ms-screenclip://capture/image\
         ?rectangle\
         &enabledModes=SnippingAllModes\
         &user-agent=actiona-run\
         &x-request-correlation-id={correlation_id}\
         &redirect-uri={REDIRECT_SCHEME}://{REDIRECT_HOST}"
    );

    open::that(&launch_uri).map_err(|e| eyre!("failed to launch Snipping Tool: {e}"))?;

    select! {
        result = receive_url(listener, cancellation.clone()) => result,
        _ = cancellation.cancelled() => Ok(None),
    }
}

/// Accepts one connection from the relay process and reads the callback URL.
async fn receive_url(listener: Listener, cancellation: CancellationToken) -> Result<Option<Image>> {
    let mut connection: Stream = listener
        .accept()
        .await
        .map_err(|e| eyre!("socket accept failed: {e}"))?;

    let mut buf = String::new();
    connection
        .read_to_string(&mut buf)
        .await
        .map_err(|e| eyre!("socket read failed: {e}"))?;

    let url = Url::parse(&buf).map_err(|e| eyre!("invalid callback URL: {e}"))?;
    handle_callback(url, cancellation).await
}

async fn handle_callback(url: Url, cancellation: CancellationToken) -> Result<Option<Image>> {
    let mut code = None;
    let mut token = None;
    let mut reason = String::new();

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = value.parse::<u16>().ok(),
            "file-access-token" => token = Some(value.into_owned()),
            "reason" => reason = value.into_owned(),
            _ => {}
        }
    }

    match code {
        Some(200) => {
            let token = token.ok_or_else(|| eyre!("Snipping Tool response missing token"))?;
            let path = redeem_token(&token, cancellation.clone()).await?;
            let image = select! {
                result = Image::load(&path) => result?,
                _ = cancellation.cancelled() => return Ok(None),
            };
            Ok(Some(image))
        }
        // 499 = user cancelled the snip
        Some(499) => Ok(None),
        Some(code) => Err(eyre!("Snipping Tool error {code}: {reason}")),
        None => Err(eyre!("Snipping Tool response missing status code")),
    }
}

async fn redeem_token(token: &str, cancellation: CancellationToken) -> Result<String> {
    select! {
        result = async {
            let file = SharedStorageAccessManager::RedeemTokenForFileAsync(&token.into())
                .map_err(|e| eyre!("failed to redeem storage token: {e}"))?
                .await
                .map_err(|e| eyre!("storage token redemption failed: {e}"))?;
            file.Path()
                .map(|p| p.to_string())
                .map_err(|e| eyre!("failed to get file path: {e}"))
        } => result,
        _ = cancellation.cancelled() => Err(eyre!("cancelled")),
    }
}
