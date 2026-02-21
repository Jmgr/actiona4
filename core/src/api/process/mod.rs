use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use color_eyre::{Result, eyre::eyre};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, BufReader},
    process::{Child, ChildStdin, Command},
    sync::mpsc,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[cfg(unix)]
use crate::api::system::processes::Signal;
use crate::{
    cancel_on,
    types::{display::DisplayFields, pid::Pid},
};

pub mod js;

/// Options for starting a process.
#[derive(Clone, Debug, Default)]
pub struct StartProcessOptions {
    pub args: Vec<String>,
    pub working_directory: Option<String>,
    pub env: Option<HashMap<String, String>>,
}

/// The result of a process that has finished.
#[derive(Clone, Debug)]
pub struct ProcessExitResult {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl ProcessExitResult {
    #[must_use]
    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    #[must_use]
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    #[must_use]
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    #[must_use]
    pub fn into_parts(self) -> (Option<i32>, String, String) {
        (self.exit_code, self.stdout, self.stderr)
    }
}

impl Display for ProcessExitResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DisplayFields::default()
            .display_if_some("exitCode", &self.exit_code)
            .finish(f)
    }
}

/// Spawns a reader task for a stdio stream that sends lines to the mpsc channel.
fn spawn_line_reader(
    task_tracker: &TaskTracker,
    reader: BufReader<impl tokio::io::AsyncRead + Unpin + Send + 'static>,
    sender: mpsc::Sender<String>,
    cancellation_token: CancellationToken,
) {
    task_tracker.spawn(async move {
        let mut reader = reader;
        let mut line = String::new();
        loop {
            line.clear();
            let Ok(result) = cancel_on(&cancellation_token, reader.read_line(&mut line)).await
            else {
                break;
            };

            match result {
                Ok(0) => break, // EOF
                Ok(_) => {
                    let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
                    if sender.send(trimmed.to_string()).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
}

async fn read_stream_to_string(mut stream: impl AsyncRead + Unpin) -> Result<String> {
    let mut reader = BufReader::new(&mut stream);
    let mut line = String::new();
    let mut lines = Vec::new();

    loop {
        line.clear();
        let count = reader.read_line(&mut line).await?;
        if count == 0 {
            break;
        }

        lines.push(
            line.trim_end_matches('\n')
                .trim_end_matches('\r')
                .to_string(),
        );
    }

    Ok(lines.join("\n"))
}

/// Process runner that creates and manages child processes.
#[derive(Clone, Debug)]
pub struct ProcessRunner {
    task_tracker: TaskTracker,
}

impl Display for ProcessRunner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DisplayFields::default().finish(f)
    }
}

impl ProcessRunner {
    #[must_use]
    pub const fn new(task_tracker: TaskTracker) -> Self {
        Self { task_tracker }
    }

    fn build_command(command: &str, options: &StartProcessOptions) -> Command {
        let mut cmd = Command::new(command);
        cmd.args(&options.args);

        if let Some(ref cwd) = options.working_directory {
            cmd.current_dir(cwd);
        }

        if let Some(ref env) = options.env {
            cmd.envs(env);
        }

        cmd
    }

    /// Start a process and return its parts for the JS layer to manage.
    pub fn start(
        &self,
        command: &str,
        options: StartProcessOptions,
        cancellation_token: CancellationToken,
    ) -> Result<StartedProcess> {
        let mut cmd = Self::build_command(command, &options);
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn()?;

        let raw_pid = child
            .id()
            .ok_or_else(|| eyre!("process exited immediately"))?;
        let pid = Pid::try_from(raw_pid)?;

        let stdin = child.stdin.take();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let stdout_receiver = stdout.map(|stdout| {
            let (sender, receiver) = mpsc::channel(256);
            spawn_line_reader(
                &self.task_tracker,
                BufReader::new(stdout),
                sender,
                cancellation_token.clone(),
            );
            receiver
        });

        let stderr_receiver = stderr.map(|stderr| {
            let (sender, receiver) = mpsc::channel(256);
            spawn_line_reader(
                &self.task_tracker,
                BufReader::new(stderr),
                sender,
                cancellation_token.clone(),
            );
            receiver
        });

        Ok(StartedProcess {
            pid,
            stdin,
            stdout_receiver,
            stderr_receiver,
            child,
        })
    }

    /// Start a process and collect stdout/stderr until it exits.
    pub async fn start_and_wait(
        &self,
        command: &str,
        options: StartProcessOptions,
        cancellation_token: CancellationToken,
    ) -> Result<ProcessExitResult> {
        let mut cmd = Self::build_command(command, &options);
        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let stdout_reader = tokio::spawn(async move {
            match stdout {
                Some(stdout) => read_stream_to_string(stdout).await,
                None => Ok(String::new()),
            }
        });

        let stderr_reader = tokio::spawn(async move {
            match stderr {
                Some(stderr) => read_stream_to_string(stderr).await,
                None => Ok(String::new()),
            }
        });

        let status = tokio::select! {
            _ = cancellation_token.cancelled() => {
                child.kill().await?;
                child.wait().await?
            }
            status = child.wait() => status?,
        };

        let stdout = stdout_reader
            .await
            .map_err(|error| eyre!("failed to join stdout reader task: {error}"))??;
        let stderr = stderr_reader
            .await
            .map_err(|error| eyre!("failed to join stderr reader task: {error}"))??;

        Ok(ProcessExitResult {
            exit_code: status.code(),
            stdout,
            stderr,
        })
    }

    /// Start a detached process and return only its PID.
    pub fn start_detached(&self, command: &str, options: StartProcessOptions) -> Result<Pid> {
        let mut cmd = Self::build_command(command, &options);
        cmd.stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(false);

        let child = cmd.spawn()?;
        let raw_pid = child
            .id()
            .ok_or_else(|| eyre!("process exited immediately"))?;

        Pid::try_from(raw_pid)
    }
}

/// Send a signal to a process by PID (Linux only).
#[cfg(unix)]
pub fn send_signal(pid: Pid, signal: Signal) -> Result<()> {
    use crate::api::system::platform::linux::ProcessSignal;
    ProcessSignal::send_signal(pid, signal)
}

/// Send a signal to a process by PID and wait for it to exit (Linux only).
#[cfg(unix)]
pub async fn send_signal_and_wait(
    pid: Pid,
    signal: Signal,
    cancellation_token: CancellationToken,
) -> Result<Option<i32>> {
    use crate::api::system::platform::linux::ProcessSignal;
    ProcessSignal::send_signal_and_wait(pid, signal, cancellation_token).await
}

/// Kill a process by PID (SIGKILL on Unix, TerminateProcess on Windows).
pub fn kill_by_pid(pid: Pid) -> Result<()> {
    #[cfg(unix)]
    {
        send_signal(pid, Signal::Kill)
    }

    #[cfg(windows)]
    {
        let process_id: u32 = pid.into();
        crate::platform::win::process_info::terminate_process_by_pid(process_id)
    }
}

/// Terminate a process by PID (SIGTERM on Unix, WM_CLOSE to all windows on Windows).
pub fn terminate_by_pid(pid: Pid) -> Result<()> {
    #[cfg(unix)]
    {
        send_signal(pid, Signal::Term)
    }

    #[cfg(windows)]
    {
        use crate::platform::win::process_info::{send_close_message_to_window, windows_for_pid};

        let process_id: u32 = pid.into();
        let windows = windows_for_pid(process_id)?;
        let mut close_failed = false;
        for hwnd in windows.iter().copied() {
            if send_close_message_to_window(hwnd).is_err() {
                close_failed = true;
            }
        }

        // Console/background processes may have no windows. In that case,
        // or if window-close signaling fails, fall back to force termination.
        if windows.is_empty() || close_failed {
            return crate::platform::win::process_info::terminate_process_by_pid(process_id);
        }

        Ok(())
    }
}

/// The result of starting a process — provides all the pieces the JS layer needs.
pub struct StartedProcess {
    pub pid: Pid,
    pub stdin: Option<ChildStdin>,
    pub stdout_receiver: Option<mpsc::Receiver<String>>,
    pub stderr_receiver: Option<mpsc::Receiver<String>>,
    pub child: Child,
}
