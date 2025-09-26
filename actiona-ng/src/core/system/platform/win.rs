#![allow(unsafe_code)]

use eyre::Result;

use crate::core::system::platform::ProcessSignal;

#[derive(Default, Debug)]
pub struct ProcessSignalImpl {}

impl ProcessSignal for ProcessSignalImpl {
    fn send_signal(pid: u32, signal: crate::core::system::processes::Signal) -> Result<()> {
        todo!()
    }

    async fn send_signal_and_wait(
        pid: u32,
        signal: crate::core::system::processes::Signal,
        cancellation_token: tokio_util::sync::CancellationToken,
    ) -> Result<Option<i32>> {
        todo!()
    }
}

// TODO: "graceful" process kill
