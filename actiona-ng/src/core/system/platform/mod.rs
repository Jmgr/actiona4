use eyre::Result;
use tokio_util::sync::CancellationToken;

use crate::core::system::processes::Signal;

#[cfg(unix)]
pub mod linux;

#[cfg(windows)]
pub mod win;

pub(crate) trait ProcessSignal {
    fn send_signal(pid: u32, signal: Signal) -> Result<()>;
    async fn send_signal_and_wait(
        pid: u32,
        signal: Signal,
        cancellation_token: CancellationToken,
    ) -> Result<Option<i32>>;
}
