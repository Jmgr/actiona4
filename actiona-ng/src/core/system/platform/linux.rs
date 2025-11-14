use std::{os::fd::AsFd, time::Duration};

use color_eyre::{
    Result,
    eyre::{self, eyre},
};
use procfs::process::Process;
use rustix::{
    io::Errno,
    process::{
        PidfdFlags, WaitId, WaitIdOptions, kill_process, pidfd_open, pidfd_send_signal, waitid,
    },
};
use tokio::{
    io::{Interest, unix::AsyncFd},
    select,
    time::sleep,
};
use tokio_util::sync::CancellationToken;

use crate::{CommonError::Cancelled, core::system::processes::Signal, types::pid::Pid};

impl From<Signal> for rustix::process::Signal {
    fn from(signal: Signal) -> Self {
        match signal {
            Signal::Hup => Self::HUP,
            Signal::Int => Self::INT,
            Signal::Quit => Self::QUIT,
            Signal::Term => Self::TERM,
            Signal::Kill => Self::KILL,
            Signal::Stop => Self::STOP,
            Signal::Tstp => Self::TSTP,
            Signal::Cont => Self::CONT,
            Signal::Ttin => Self::TTIN,
            Signal::Ttou => Self::TTOU,
            Signal::Winch => Self::WINCH,
            Signal::Usr1 => Self::USR1,
            Signal::Usr2 => Self::USR2,
        }
    }
}

enum ProcessSignalErrors {
    Unsupported,
    Other(eyre::ErrReport),
}

#[derive(Debug, Default)]
pub struct ProcessSignal {}

impl ProcessSignal {
    async fn send_signal_and_wait_legacy(
        pid: Pid,
        signal: Signal,
        cancellation_token: CancellationToken,
    ) -> Result<Option<i32>> {
        let process = Process::new(pid.try_into()?)?;
        let stat = process.stat()?;
        let start_time = stat.starttime;

        let pid = pid.try_into()?;
        kill_process(pid, signal.into())?;

        loop {
            match waitid(
                WaitId::Pid(pid),
                WaitIdOptions::NOHANG | WaitIdOptions::EXITED,
            ) {
                Ok(Some(status)) => {
                    // Process finished.
                    return Ok(status.exit_status());
                }
                Ok(None) => {
                    // Process not yet finished, wait and try again.
                    select! {
                        _ = sleep(Duration::from_millis(10)) => continue,
                        _ = cancellation_token.cancelled() => return Err(Cancelled.into()),
                    }
                }
                Err(err) if err == Errno::INTR => {
                    // Interrupted, try again.
                    continue;
                }
                Err(err) if err == Errno::CHILD => {
                    // This process is not our child, go to the fallback option.
                    break;
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        }

        loop {
            let Ok(new_process) = Process::new(pid.as_raw_pid()) else {
                // The process doesn't exist anymore.
                return Ok(None);
            };

            let new_stat = new_process.stat()?;
            if new_stat.starttime != start_time {
                // A process with that PID doesn't have the same start time, so the PID must have been reused.
                return Ok(None);
            }

            select! {
                _ = sleep(Duration::from_millis(10)) => continue,
                _ = cancellation_token.cancelled() => return Err(Cancelled.into()),
            }
        }
    }

    async fn send_signal_and_wait_pidfd(
        pid: Pid,
        signal: Signal,
        cancellation_token: CancellationToken,
    ) -> std::result::Result<Option<i32>, ProcessSignalErrors> {
        let pid = pid
            .try_into()
            .map_err(|error: eyre::Error| ProcessSignalErrors::Other(error))?;

        let pidfd = pidfd_open(pid, PidfdFlags::empty()).map_err(|errno| match errno {
            Errno::NOSYS => ProcessSignalErrors::Unsupported,
            errno => ProcessSignalErrors::Other(errno.into()),
        })?;

        pidfd_send_signal(&pidfd, signal.into()).map_err(|errno| match errno {
            Errno::NOSYS => ProcessSignalErrors::Unsupported,
            errno => ProcessSignalErrors::Other(errno.into()),
        })?;

        let async_fd = AsyncFd::try_with_interest(
            pidfd
                .try_clone()
                .map_err(|error| ProcessSignalErrors::Other(error.into()))?,
            Interest::READABLE,
        )
        .map_err(|error| ProcessSignalErrors::Other(error.into()))?;

        let _guard = select!{
            _ = cancellation_token.cancelled() => { return Err(ProcessSignalErrors::Other(Cancelled.into())); },
            guard = async_fd.readable() => guard,
        }.map_err(|error| ProcessSignalErrors::Other(error.into()))?;

        let status = waitid(WaitId::PidFd(pidfd.as_fd()), WaitIdOptions::EXITED)
            .map_err(|error| ProcessSignalErrors::Other(error.into()))?
            .ok_or_else(|| ProcessSignalErrors::Other(eyre!("waitid returned None")))?;

        Ok(status.exit_status())
    }

    pub fn send_signal(pid: Pid, signal: Signal) -> Result<()> {
        kill_process(pid.try_into()?, signal.into())?;

        Ok(())
    }

    pub async fn send_signal_and_wait(
        pid: Pid,
        signal: Signal,
        cancellation_token: CancellationToken,
    ) -> Result<Option<i32>> {
        match Self::send_signal_and_wait_pidfd(pid, signal, cancellation_token.clone()).await {
            Ok(result) => Ok(result), // TODO
            Err(ProcessSignalErrors::Unsupported) => {
                Self::send_signal_and_wait_legacy(pid, signal, cancellation_token).await
            } // TODO
            Err(ProcessSignalErrors::Other(err)) => Err(err),
        }
    }
}
