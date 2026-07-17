use std::{
    path::{Path, PathBuf},
    pin::Pin,
    process::Stdio,
    sync::Arc,
    time::Duration,
};

use color_eyre::{Result, eyre::eyre};
use derive_where::derive_where;
use futures_util::StreamExt;
use parking_lot::Mutex;
use tokio::{io::AsyncRead, process::Command, sync::oneshot, time::sleep};
use tokio_util::{
    codec::{FramedRead, LinesCodec},
    sync::CancellationToken,
    task::TaskTracker,
};
use tracing::{error, info, warn};

use crate::{IpcHost, RESTART_DELAY, protocol::Protocol};

type HostHandlerFut<P> =
    Pin<Box<dyn Future<Output = Result<<P as Protocol>::HostResponse, String>> + Send + 'static>>;

type HostHandler<P> =
    Arc<dyn Fn(<P as Protocol>::ExtensionRequest) -> HostHandlerFut<P> + Send + Sync + 'static>;

const MAX_PLUGIN_OUTPUT_LINE: usize = 64 * 1024;

#[cfg(windows)]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

#[derive_where(Debug)]
pub struct Host<P: Protocol> {
    executable_path: PathBuf,
    token: CancellationToken,
    #[derive_where(skip)]
    handler: HostHandler<P>,
    inner: Mutex<Arc<IpcHost<P>>>,
    task_tracker: TaskTracker,
    timeout: Duration,
}

impl<P: Protocol> Host<P> {
    pub async fn new(
        executable_path: &Path,
        task_tracker: TaskTracker,
        token: CancellationToken,
        timeout: Duration,
    ) -> Result<Self> {
        Self::with_handler(executable_path, task_tracker, token, timeout, async |_| {
            Err("unexpected message".to_string())
        })
        .await
    }

    pub async fn with_handler<F, Fut>(
        executable_path: &Path,
        task_tracker: TaskTracker,
        token: CancellationToken,
        timeout: Duration,
        message_handler: F,
    ) -> Result<Self>
    where
        F: Fn(P::ExtensionRequest) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<P::HostResponse, String>> + Send + 'static,
    {
        let handler: HostHandler<P> =
            Arc::new(move |request| Box::pin(message_handler(request)) as HostHandlerFut<P>);
        let inner = build_inner(&handler, timeout).await?;
        Ok(Self {
            executable_path: executable_path.to_path_buf(),
            token,
            handler,
            inner: Mutex::new(Arc::new(inner)),
            task_tracker,
            timeout,
        })
    }

    pub async fn run(&self, ready: oneshot::Sender<()>) -> Result<()> {
        self.run_inner(ready).await
    }

    async fn run_inner(&self, ready: oneshot::Sender<()>) -> Result<()> {
        let mut ready = Some(ready);
        loop {
            if self.token.is_cancelled() {
                return Ok(());
            }

            info!("starting process at {}", self.executable_path.display());

            let key = {
                let inner = self.inner.lock();
                inner.key()
            };

            let mut command = Command::new(&self.executable_path);
            command
                .arg(key.to_string())
                .kill_on_drop(true)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            // Make sure we spawn an independent process
            #[cfg(unix)]
            command.process_group(0);
            #[cfg(windows)]
            command.creation_flags(CREATE_NEW_PROCESS_GROUP);

            let mut child = match command.spawn() {
                Ok(child) => child,
                Err(e) => {
                    error!("failed to spawn process: {e}");
                    if restart_delay_cancelled(&self.token).await {
                        return Ok(());
                    }
                    continue;
                }
            };

            if let Some(stdout) = child.stdout.take() {
                self.task_tracker
                    .spawn(forward_lines(stdout, false, self.token.clone()));
            }
            if let Some(stderr) = child.stderr.take() {
                self.task_tracker
                    .spawn(forward_lines(stderr, true, self.token.clone()));
            }

            if let Some(pid) = child.id() {
                info!("started process with PID {pid}")
            } else {
                error!("started process but it exited immediately");
                if restart_delay_cancelled(&self.token).await {
                    return Ok(());
                }
                continue;
            }

            // Phase 1: wait for the client to connect, the child to die first,
            // or cancellation. If the child dies before connecting, the inner
            // host's listener is still pending and we can reuse it.
            let connect_result = {
                let inner = self.inner.lock().clone();
                tokio::select! {
                    () = self.token.cancelled() => {
                        let _ = child.kill().await;
                        return Ok(());
                    }
                    res = inner.wait_for_client_to_connect() => Some(res),
                    status = child.wait() => {
                        warn!("child exited before connecting: {status:?}");
                        None
                    }
                }
            };

            match connect_result {
                None => {
                    // Child died pre-connect: inner host is still usable.
                    if restart_delay_cancelled(&self.token).await {
                        return Ok(());
                    }
                    continue;
                }
                Some(Err(e)) => {
                    // Mid-handshake failure: inner host is dead, must rebuild.
                    error!("client failed to connect: {e}; rebuilding host");
                    let _ = child.kill().await;
                    self.replace_inner().await?;
                    if restart_delay_cancelled(&self.token).await {
                        return Ok(());
                    }
                    continue;
                }
                Some(Ok(())) => {
                    info!("client connected");
                    if let Some(ready) = ready.take() {
                        let _ = ready.send(());
                    }
                }
            }

            // Phase 2: connected. Wait for either the child to exit, the
            // client to disconnect, or cancellation. Either of the first two
            // means we need a fresh inner host (its one-shot is consumed).
            {
                let inner = self.inner.lock().clone();
                tokio::select! {
                    () = self.token.cancelled() => {
                        let _ = child.kill().await;
                        return Ok(());
                    }
                    status = child.wait() => {
                        warn!("child exited: {status:?}");
                    }
                    res = inner.wait_for_client_to_disconnect() => {
                        match res {
                            Ok(()) => warn!("client disconnected cleanly"),
                            Err(e) => error!("client disconnect error: {e}"),
                        }
                        let _ = child.kill().await;
                    }
                }
            }

            self.replace_inner().await?;
            if restart_delay_cancelled(&self.token).await {
                return Ok(());
            }
        }
    }

    /// Sends a request to the connected extension.
    ///
    /// If the host is cancelled (or the extension restart logic in `start`
    /// swaps the inner server while a send is pending), the call resolves
    /// promptly rather than waiting out the full timeout.
    pub async fn send(
        &self,
        message: P::HostRequest,
    ) -> color_eyre::Result<Result<P::ExtensionResponse, String>> {
        let inner = self.inner.lock().clone();
        tokio::select! {
            () = self.token.cancelled() => Err(eyre!("host cancelled")),
            result = inner.send(message) => result,
        }
    }

    async fn replace_inner(&self) -> Result<()> {
        let new_inner = build_inner(&self.handler, self.timeout).await?;
        *self.inner.lock() = Arc::new(new_inner);
        Ok(())
    }
}

async fn build_inner<P: Protocol>(
    handler: &HostHandler<P>,
    timeout: Duration,
) -> Result<IpcHost<P>> {
    let handler = Arc::clone(handler);
    IpcHost::with_handler(timeout, move |req| handler(req)).await
}

async fn restart_delay_cancelled(token: &CancellationToken) -> bool {
    tokio::select! {
        () = token.cancelled() => true,
        () = sleep(RESTART_DELAY) => false,
    }
}

/// Read lines from a child's pipe and re-emit them on the host's stdout/stderr
/// with a "plugin: " prefix. Terminates when the pipe closes or when the
/// cancellation token fires. Decode errors (e.g. an over-long line) are
/// logged but do not stop forwarding — `LinesCodec` resyncs on the next newline.
///
/// Cancellation is required because grandchildren of the extension (e.g. the
/// sentry minidump `--crash-reporter-server` helper) inherit the pipe and may
/// keep it open after the extension itself is killed, which would otherwise
/// stall `task_tracker.wait()` at shutdown.
async fn forward_lines<R>(reader: R, is_stderr: bool, token: CancellationToken)
where
    R: AsyncRead + Unpin + Send + 'static,
{
    use tokio_util::codec::LinesCodecError;

    let mut lines = FramedRead::new(
        reader,
        LinesCodec::new_with_max_length(MAX_PLUGIN_OUTPUT_LINE),
    );
    loop {
        let line = tokio::select! {
            () = token.cancelled() => return,
            line = lines.next() => line,
        };
        let Some(line) = line else { return };
        match line {
            Ok(line) => {
                if is_stderr {
                    warn!("plugin: {line}");
                } else {
                    info!("plugin: {line}");
                }
            }
            Err(LinesCodecError::MaxLineLengthExceeded) => {
                warn!("plugin: dropped over-long output line");
            }
            Err(LinesCodecError::Io(e)) => {
                // I/O error means the pipe is unusable; stop forwarding.
                warn!("error reading plugin output: {e}");
                break;
            }
        }
    }
}
