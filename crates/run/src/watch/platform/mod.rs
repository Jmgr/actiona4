use std::{env, process::ExitStatus, time::Duration};

use color_eyre::{Result, eyre::Context};
use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, ToNsName,
    tokio::{Listener, Stream, prelude::*},
};
use tokio::{
    process::{Child, Command},
    select,
    time::timeout,
};
use uuid::Uuid;

use crate::watch::child_command;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub(super) use unix::{RunningScript, UnixScript as PlatformScript, paths_match};
#[cfg(windows)]
pub(super) use windows::{RunningScript, WindowsScript as PlatformScript, paths_match};

pub trait ScriptPlatform: Sized {
    fn new(child: &mut Child) -> Result<Self>;
    fn configure_command(command: &mut Command);
    fn kill_child(child: &mut Child) -> Result<()>;
}

pub struct RunningScriptBase<P: ScriptPlatform> {
    _platform: P,
    pub(super) child: Child,
    pub(super) listener: Listener,
    pub(super) control: Option<Stream>,
}

pub(super) fn configure_command(command: &mut Command) {
    PlatformScript::configure_command(command);
}

impl<P: ScriptPlatform> RunningScriptBase<P> {
    pub fn start() -> Result<Self> {
        let name = format!("actiona-watch-{}", Uuid::new_v4());
        let listener = ListenerOptions::new()
            .name(name.as_str().to_ns_name::<GenericNamespaced>()?)
            .create_tokio()
            .context("creating watch control socket")?;
        let mut command = child_command(env::args_os().skip(1), &name)?;
        let mut child = command.spawn().context("starting watched script")?;
        let platform = P::new(&mut child)?;
        Ok(Self {
            child,
            listener,
            control: None,
            _platform: platform,
        })
    }

    pub async fn stop(&mut self, grace: Duration) -> Result<ExitStatus> {
        let graceful = async {
            if self.control.is_none() {
                select! {
                    result = self.child.wait() => return result,
                    connection = self.listener.accept() => {
                        drop(connection?);
                    }
                }
            }
            drop(self.control.take());
            self.child.wait().await
        };

        if let Ok(status) = timeout(grace, graceful).await {
            status.context("waiting for watched script to stop")
        } else {
            eprintln!("[watch] Script did not stop in time; terminating it.");
            self.force_stop().await
        }
    }

    async fn force_stop(&mut self) -> Result<ExitStatus> {
        P::kill_child(&mut self.child)?;
        self.child
            .wait()
            .await
            .context("waiting for watched script to stop")
    }
}

#[cfg(test)]
mod tests {
    use std::{env, process::Stdio, thread, time::Duration};

    use tokio::time::timeout;

    use super::{PlatformScript, RunningScript, ScriptPlatform};

    #[tokio::test]
    async fn unresponsive_child_is_killed_after_the_deadline() {
        use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
        use tokio::process::Command;
        use uuid::Uuid;

        let name = format!("actiona-watch-timeout-test-{}", Uuid::new_v4());
        let listener = ListenerOptions::new()
            .name(
                name.as_str()
                    .to_ns_name::<GenericNamespaced>()
                    .expect("socket name"),
            )
            .create_tokio()
            .expect("create listener");
        let mut command = Command::new(env::current_exe().expect("test executable"));
        command
            .args([
                "--ignored",
                "--exact",
                "watch::platform::tests::unresponsive_worker",
            ])
            .env("ACTIONA_WATCH_UNRESPONSIVE_TEST", "1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        PlatformScript::configure_command(&mut command);
        command.stdin(Stdio::null());
        let mut child = command.spawn().expect("start worker");
        let platform = PlatformScript::new(&mut child).expect("set up worker");
        let mut script = RunningScript {
            child,
            listener,
            control: None,
            _platform: platform,
        };
        timeout(
            Duration::from_secs(2),
            script.stop(Duration::from_millis(100)),
        )
        .await
        .expect("stop deadline enforced")
        .expect("stop worker");
        let status = script
            .child
            .try_wait()
            .expect("child status")
            .expect("worker reaped");
        assert!(!status.success());
    }

    #[test]
    #[ignore = "subprocess fixture for the shutdown deadline test"]
    fn unresponsive_worker() {
        if env::var_os("ACTIONA_WATCH_UNRESPONSIVE_TEST").is_some() {
            thread::sleep(Duration::from_secs(60));
        }
    }
}
