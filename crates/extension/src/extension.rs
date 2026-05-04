use std::{pin::Pin, sync::Arc, time::Duration};

use color_eyre::{Result, eyre::eyre};
use derive_where::derive_where;
use ipc_rpc::ConnectionKey;
use parking_lot::Mutex;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, warn};

use crate::{IpcExtension, protocol::Protocol};

type ExtensionHandlerFut<P> = Pin<
    Box<dyn Future<Output = Result<<P as Protocol>::ExtensionResponse, String>> + Send + 'static>,
>;

type ExtensionHandler<P> =
    Arc<dyn Fn(<P as Protocol>::HostRequest) -> ExtensionHandlerFut<P> + Send + Sync + 'static>;

#[derive_where(Debug)]
pub struct Extension<P: Protocol> {
    token: CancellationToken,
    #[derive_where(skip)]
    handler: ExtensionHandler<P>,
    key: ConnectionKey,
    inner: Mutex<Option<Arc<IpcExtension<P>>>>,
    task_tracker: TaskTracker,
    timeout: Duration,
}

impl<P: Protocol> Extension<P> {
    pub fn new(
        key: ConnectionKey,
        task_tracker: TaskTracker,
        token: CancellationToken,
        timeout: Duration,
    ) -> Self {
        Self::with_handler(key, task_tracker, token, timeout, async |_| {
            Err("unexpected message".to_string())
        })
    }

    pub fn with_handler<F, Fut>(
        key: ConnectionKey,
        task_tracker: TaskTracker,
        token: CancellationToken,
        timeout: Duration,
        message_handler: F,
    ) -> Self
    where
        F: Fn(P::HostRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<P::ExtensionResponse, String>> + Send + 'static,
    {
        let handler: ExtensionHandler<P> =
            Arc::new(move |request| Box::pin(message_handler(request)) as ExtensionHandlerFut<P>);
        Self {
            token,
            handler,
            key,
            inner: Mutex::new(None),
            task_tracker,
            timeout,
        }
    }

    pub async fn run(&self) -> Result<()> {
        // Install signal handlers synchronously so SIGTERM/SIGINT don't kill
        // us via the default action before our handler future runs.
        let signal_fut = install_signal_handler()?;

        let handler = Arc::clone(&self.handler);
        *self.inner.lock() = None;
        let inner = Arc::new(build_inner(self.key.clone(), &handler, self.timeout).await?);
        *self.inner.lock() = Some(Arc::clone(&inner));
        info!("extension connected");

        let signal_token = self.token.clone();
        let signal_task = self.task_tracker.spawn(async move {
            signal_fut.await;
            signal_token.cancel();
        });

        tokio::select! {
            _ = self.token.cancelled() => {
                info!("extension cancellation requested");
            }
            res = inner.wait_for_host_to_disconnect() => {
                match res {
                    Ok(()) => info!("host disconnected cleanly"),
                    Err(e) => warn!("host disconnect: {e}"),
                }
            }
        }

        signal_task.abort();
        // Drop inner explicitly so IpcRpcClient::Drop sends Hangup before we return.
        *self.inner.lock() = None;
        Ok(())
    }

    /// Sends a request to the host. Resolves promptly if the extension is
    /// cancelled rather than waiting out the full timeout.
    pub async fn send(
        &self,
        message: P::ExtensionRequest,
    ) -> color_eyre::Result<Result<P::HostResponse, String>> {
        let inner = self
            .inner
            .lock()
            .clone()
            .ok_or_else(|| eyre!("extension is not connected"))?;
        tokio::select! {
            () = self.token.cancelled() => Err(eyre!("extension cancelled")),
            result = inner.send(message) => result,
        }
    }
}

async fn build_inner<P: Protocol>(
    key: ConnectionKey,
    handler: &ExtensionHandler<P>,
    timeout: Duration,
) -> Result<IpcExtension<P>> {
    let handler = Arc::clone(handler);
    IpcExtension::with_handler(key, timeout, move |req| handler(req)).await
}

#[cfg(unix)]
fn install_signal_handler() -> Result<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> {
    use tokio::signal::unix::{SignalKind, signal};
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;
    Ok(Box::pin(async move {
        tokio::select! {
            _ = sigterm.recv() => {}
            _ = sigint.recv() => {}
        }
    }))
}

#[cfg(not(unix))]
fn install_signal_handler() -> Result<Pin<Box<dyn Future<Output = ()> + Send + 'static>>> {
    Ok(Box::pin(async {
        let _ = tokio::signal::ctrl_c().await;
    }))
}
