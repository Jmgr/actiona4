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
            Some(Err("unexpected message".to_owned()))
        })
        .await
    }

    /// `message_handler` answering `None` leaves the message unanswered, which
    /// is what a no-reply call expects.
    pub async fn with_handler<F, Fut>(timeout: Duration, message_handler: F) -> Result<Self>
    where
        F: Fn(P::ExtensionRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<Result<P::HostResponse, String>>> + Send,
    {
        let message_handler = Arc::new(message_handler);
        let (key, server) = IpcRpcServer::initialize_server(move |message| {
            let message_handler = Arc::clone(&message_handler);
            async move {
                if let WireMessage::ExtensionRequest(request) = message {
                    message_handler(request)
                        .await
                        .map(WireMessage::HostResponse)
                } else {
                    error!("host: unexpected message received: {message:?}");
                    None
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

    #[must_use]
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

        let WireMessage::ExtensionResponse(response) = response else {
            bail!("host: unexpected reply received: {response:?}");
        };

        Ok(response)
    }

    /// Sends a message the extension will not answer.
    ///
    /// `send_timeout` hands the message to the transport before it returns and
    /// the future it gives back only waits for a reply, so dropping that future
    /// is the send. The timeout is not a delivery deadline: it only says how
    /// long the reply slot registered for the message lingers before being
    /// reaped, so it stays short regardless of the protocol's own timeout.
    pub fn notify(&self, message: P::HostRequest) -> Result<()> {
        const REPLY_SLOT_TTL: Duration = Duration::from_secs(5);

        if !self.server.client_connected() {
            bail!("host: extension is not connected");
        }

        drop(
            self.server
                .send_timeout(WireMessage::HostRequest(message), REPLY_SLOT_TTL),
        );

        Ok(())
    }

    pub async fn wait_for_client_to_connect(&self) -> Result<()> {
        Ok(self.server.wait_for_client_to_connect().await?)
    }

    pub async fn wait_for_client_to_disconnect(&self) -> Result<()> {
        Ok(self.server.wait_for_client_to_disconnect().await?)
    }
}
