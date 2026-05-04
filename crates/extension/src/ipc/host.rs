use std::{sync::Arc, time::Duration};

use color_eyre::{Result, eyre::bail};
use ipc_rpc::{ConnectionKey, IpcRpcServer};
use tracing::error;

use crate::protocol::{Protocol, WireMessage};

#[derive(Debug)]
pub struct Host<P: Protocol> {
    key: ConnectionKey,
    server: IpcRpcServer<WireMessage<P>>,
    timeout: Duration,
}

impl<P: Protocol> Host<P> {
    pub async fn new(timeout: Duration) -> Result<Self> {
        Self::with_handler(timeout, async |_message| {
            Err("unexpected message".to_string())
        })
        .await
    }

    pub async fn with_handler<F, Fut>(timeout: Duration, message_handler: F) -> Result<Self>
    where
        F: Fn(P::ExtensionRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<P::HostResponse, String>> + Send,
    {
        let message_handler = Arc::new(message_handler);
        let (key, server) = IpcRpcServer::initialize_server(move |message| {
            let message_handler = Arc::clone(&message_handler);
            async move {
                match message {
                    WireMessage::ExtensionRequest(request) => {
                        let response = message_handler(request).await;
                        Some(WireMessage::HostResponse(response))
                    }
                    _ => {
                        error!("host: unexpected message received: {message:?}");
                        None
                    }
                }
            }
        })
        .await?;

        Ok(Self {
            key,
            server,
            timeout,
        })
    }

    pub fn key(&self) -> ConnectionKey {
        self.key.clone()
    }

    pub async fn send(
        &self,
        message: P::HostRequest,
    ) -> color_eyre::Result<Result<P::ExtensionResponse, String>> {
        let response = self
            .server
            .send_timeout(WireMessage::HostRequest(message), self.timeout)
            .await?;

        Ok(match response {
            WireMessage::ExtensionResponse(response) => response,
            _ => bail!("host: unexpected reply received: {response:?}"),
        })
    }

    pub async fn wait_for_client_to_connect(&self) -> Result<()> {
        Ok(self.server.wait_for_client_to_connect().await?)
    }

    pub async fn wait_for_client_to_disconnect(&self) -> Result<()> {
        Ok(self.server.wait_for_client_to_disconnect().await?)
    }
}
