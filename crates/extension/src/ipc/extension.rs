use std::{sync::Arc, time::Duration};

use color_eyre::{Result, eyre::bail};
use ipc_rpc::{ConnectionKey, IpcRpcClient};
use tracing::error;

use crate::protocol::{Protocol, WireMessage};

#[derive(Debug)]
pub struct Extension<P: Protocol> {
    client: IpcRpcClient<WireMessage<P>>,
    timeout: Duration,
}

impl<P: Protocol> Extension<P> {
    pub async fn new(key: ConnectionKey, timeout: Duration) -> Result<Self> {
        Self::with_handler(key, timeout, async |_message| {
            Err("unexpected message".to_string())
        })
        .await
    }

    pub async fn with_handler<F, Fut>(
        key: ConnectionKey,
        timeout: Duration,
        message_handler: F,
    ) -> Result<Self>
    where
        F: Fn(P::HostRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<P::ExtensionResponse, String>> + Send,
    {
        let message_handler = Arc::new(message_handler);
        let client = IpcRpcClient::initialize_client(key, move |message| {
            let message_handler = Arc::clone(&message_handler);
            async move {
                match message {
                    WireMessage::HostRequest(request) => {
                        let response = message_handler(request).await;
                        Some(WireMessage::ExtensionResponse(response))
                    }
                    _ => {
                        error!("extension: unexpected message received: {message:?}");
                        None
                    }
                }
            }
        })
        .await?;

        Ok(Self { client, timeout })
    }

    pub async fn send(
        &self,
        message: P::ExtensionRequest,
    ) -> color_eyre::Result<Result<P::HostResponse, String>> {
        let response = self
            .client
            .send_timeout(WireMessage::ExtensionRequest(message), self.timeout)
            .await?;

        Ok(match response {
            WireMessage::HostResponse(response) => response,
            _ => bail!("extension: unexpected reply received: {response:?}"),
        })
    }

    pub async fn wait_for_host_to_disconnect(&self) -> Result<()> {
        Ok(self.client.wait_for_server_to_disconnect().await?)
    }
}
