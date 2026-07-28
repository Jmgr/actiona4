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
            Some(Err("unexpected message".to_owned()))
        })
        .await
    }

    /// `message_handler` answering `None` leaves the message unanswered, which
    /// is what a no-reply call expects.
    pub async fn with_handler<F, Fut>(
        key: ConnectionKey,
        timeout: Duration,
        message_handler: F,
    ) -> Result<Self>
    where
        F: Fn(P::HostRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Option<Result<P::ExtensionResponse, String>>> + Send,
    {
        let message_handler = Arc::new(message_handler);
        let client = IpcRpcClient::initialize_client(key, move |message| {
            let message_handler = Arc::clone(&message_handler);
            async move {
                if let WireMessage::HostRequest(request) = message {
                    message_handler(request)
                        .await
                        .map(WireMessage::ExtensionResponse)
                } else {
                    error!("extension: unexpected message received: {message:?}");
                    None
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

        let WireMessage::HostResponse(response) = response else {
            bail!("extension: unexpected reply received: {response:?}");
        };

        Ok(response)
    }

    /// Sends a message the host will not answer.
    ///
    /// `send_timeout` hands the message to the transport before it returns and
    /// the future it gives back only waits for a reply, so dropping that future
    /// is the send. The timeout is not a delivery deadline: it only says how
    /// long the reply slot registered for the message lingers before being
    /// reaped, so it stays short regardless of the protocol's own timeout.
    pub fn notify(&self, message: P::ExtensionRequest) -> Result<()> {
        const REPLY_SLOT_TTL: Duration = Duration::from_secs(5);

        drop(
            self.client
                .send_timeout(WireMessage::ExtensionRequest(message), REPLY_SLOT_TTL),
        );

        Ok(())
    }

    pub async fn wait_for_host_to_disconnect(&self) -> Result<()> {
        Ok(self.client.wait_for_server_to_disconnect().await?)
    }
}
