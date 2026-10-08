use std::{
    env,
    ffi::OsString,
    io::ErrorKind,
    path::{Path, absolute},
    process::{ExitStatus, Stdio},
    time::Duration,
};

use color_eyre::{
    Result,
    eyre::{Context, OptionExt},
};
use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, ToNsName,
    tokio::{Listener, Stream, prelude::*},
};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::{
    fs,
    io::AsyncReadExt,
    process::{Child, Command},
    select, signal,
    sync::mpsc,
    time::{sleep, timeout},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use uuid::Uuid;
#[cfg(windows)]
use windows::Win32::System::Threading::CREATE_NEW_PROCESS_GROUP;

use crate::{CANCELLED_EXIT_CODE, ScriptCancelled};

const DEBOUNCE: Duration = Duration::from_millis(150);
const STOP_TIMEOUT: Duration = Duration::from_secs(3);

pub async fn connect_control(name: &str) -> Result<Stream> {
    Stream::connect(name.to_ns_name::<GenericNamespaced>()?)
        .await
        .context("connecting to watch supervisor")
}

pub fn listen_for_stop(
    mut control: Stream,
    cancellation_token: CancellationToken,
    supervisor_stop: CancellationToken,
    task_tracker: &TaskTracker,
) {
    task_tracker.spawn(async move {
        let mut byte = [0];
        select! {
            biased;
            () = cancellation_token.cancelled() => {}
            // Closing the connection also stops the child if the supervisor dies.
            _ = control.read(&mut byte) => {
                supervisor_stop.cancel();
                cancellation_token.cancel();
            }
        }
    });
}

struct RunningScript {
    child: Child,
    listener: Listener,
    control: Option<Stream>,
}

impl RunningScript {
    fn start() -> Result<Self> {
        let name = format!("actiona-watch-{}", Uuid::new_v4());
        let listener = ListenerOptions::new()
            .name(name.as_str().to_ns_name::<GenericNamespaced>()?)
            .create_tokio()
            .context("creating watch control socket")?;
        let mut command = child_command(env::args_os().skip(1), &name)?;
        let child = command.spawn().context("starting watched script")?;
        Ok(Self {
            child,
            listener,
            control: None,
        })
    }

    async fn stop(&mut self, grace: Duration) -> Result<ExitStatus> {
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

    #[allow(unsafe_code)]
    async fn force_stop(&mut self) -> Result<ExitStatus> {
        #[cfg(linux)]
        if let Some(pid) = self.child.id() {
            let pid = i32::try_from(pid)?;
            // SAFETY: the child owns this process group; a negative PID targets the group,
            // including Linux dialog helpers that cannot clean up after a forced stop.
            let result = unsafe { libc::kill(-pid, libc::SIGKILL) };
            if result != 0 {
                self.child
                    .start_kill()
                    .context("terminating watched script")?;
            }
        }
        #[cfg(not(linux))]
        self.child
            .start_kill()
            .context("terminating watched script")?;

        self.child
            .wait()
            .await
            .context("waiting for watched script to stop")
    }
}

fn child_command(
    arguments: impl IntoIterator<Item = OsString>,
    control_name: &str,
) -> Result<Command> {
    let mut command = Command::new(env::current_exe().context("finding current executable")?);
    let mut positional_only = false;
    for argument in arguments {
        if argument == "--" {
            positional_only = true;
        }
        if !positional_only && argument == "--watch" {
            command.args(["--watch-control", control_name]);
        } else {
            command.arg(argument);
        }
    }
    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    command.kill_on_drop(true);

    // Only the supervisor receives terminal Ctrl+C. It stops the child through the socket.
    #[cfg(unix)]
    command.process_group(0);
    #[cfg(windows)]
    command.creation_flags(CREATE_NEW_PROCESS_GROUP.0);
    Ok(command)
}

fn affects_script(event: &Event, filepath: &Path) -> bool {
    event.need_rescan()
        || (!matches!(event.kind, EventKind::Access(_))
            && event.paths.iter().any(|path| paths_match(path, filepath)))
}

#[cfg(not(windows))]
fn paths_match(left: &Path, right: &Path) -> bool {
    left == right
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn paths_match(left: &Path, right: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt as _;

    use windows::Win32::Globalization::{CSTR_EQUAL, CompareStringOrdinal};

    // Windows opens paths without regard to casing, but notifications use the on-disk spelling.
    let left: Vec<_> = left.as_os_str().encode_wide().collect();
    let right: Vec<_> = right.as_os_str().encode_wide().collect();
    // SAFETY: both slices contain valid UTF-16 buffers with explicit lengths.
    unsafe { CompareStringOrdinal(&left, &right, true) == CSTR_EQUAL }
}

type Changes = mpsc::Receiver<notify::Result<()>>;

pub async fn run(filepath: &Path) -> Result<()> {
    // Resolve the directory, keeping the filename so replacing the file does not detach the watch.
    let filepath = absolute(filepath)?;
    let directory = filepath
        .parent()
        .ok_or_eyre("script has no parent directory")?;
    let filename = filepath.file_name().ok_or_eyre("script has no filename")?;
    let filepath = fs::canonicalize(directory).await?.join(filename);
    let (sender, mut changes) = mpsc::channel(64);
    let watched_path = filepath.clone();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
        let change = match event {
            Ok(event) if affects_script(&event, &watched_path) => Ok(()),
            Ok(_) => return,
            Err(error) => Err(error),
        };
        // Events are coalesced; reading the file after the debounce always gets the latest revision.
        let _ = sender.try_send(change);
    })?;
    watcher.watch(
        filepath
            .parent()
            .ok_or_eyre("script has no parent directory")?,
        RecursiveMode::NonRecursive,
    )?;
    let source = fs::read(&filepath)
        .await
        .context("reading watched script")?;

    let shutdown = CancellationToken::new();
    let signal_token = shutdown.clone();
    let signal_task = tokio::spawn(async move {
        if let Err(error) = signal::ctrl_c().await {
            eprintln!("[watch] Could not listen for Ctrl+C: {error}");
        }
        signal_token.cancel();
    });

    let mut script = None;
    let result = async {
        script = Some(RunningScript::start()?);
        eprintln!("[watch] Watching {}", filepath.display());
        run_loop(&filepath, source, &mut changes, &shutdown, &mut script).await
    }
    .await;

    let stopped = if let Some(mut script) = script {
        script.stop(STOP_TIMEOUT).await.map(|_| ())
    } else {
        Ok(())
    };
    signal_task.abort();
    let _ = signal_task.await;
    stopped?;
    result
}

enum EventSource {
    Changed,
    Connected(Stream),
    Exited(ExitStatus),
    Stopped,
}

async fn next_event(
    script: &mut Option<RunningScript>,
    changes: &mut Changes,
    shutdown: &CancellationToken,
) -> Result<EventSource> {
    if let Some(script) = script {
        select! {
            biased;
            () = shutdown.cancelled() => Ok(EventSource::Stopped),
            status = script.child.wait() => Ok(EventSource::Exited(status?)),
            connection = script.listener.accept(), if script.control.is_none() => {
                Ok(EventSource::Connected(connection?))
            }
            change = changes.recv() => {
                check_change(change)?;
                Ok(EventSource::Changed)
            }
        }
    } else {
        select! {
            () = shutdown.cancelled() => Ok(EventSource::Stopped),
            change = changes.recv() => {
                check_change(change)?;
                Ok(EventSource::Changed)
            }
        }
    }
}

fn check_change(change: Option<notify::Result<()>>) -> Result<()> {
    change
        .ok_or_eyre("script watcher stopped")?
        .context("watching script")
}

async fn debounce(changes: &mut Changes, shutdown: &CancellationToken) -> Result<bool> {
    loop {
        select! {
            () = shutdown.cancelled() => return Ok(false),
            change = changes.recv() => check_change(change)?,
            () = sleep(DEBOUNCE) => return Ok(true),
        }
    }
}

async fn read_script(filepath: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(filepath).await {
        Ok(source) => Ok(Some(source)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("reading watched script"),
    }
}

async fn run_loop(
    filepath: &Path,
    source: Vec<u8>,
    changes: &mut Changes,
    shutdown: &CancellationToken,
    script: &mut Option<RunningScript>,
) -> Result<()> {
    let mut source = Some(source);
    loop {
        match next_event(script, changes, shutdown).await? {
            EventSource::Stopped => return Err(ScriptCancelled.into()),
            EventSource::Connected(control) => {
                if let Some(script) = script {
                    script.control = Some(control);
                }
            }
            EventSource::Exited(status) => {
                *script = None;
                if status.code() == Some(i32::from(CANCELLED_EXIT_CODE)) {
                    return Err(ScriptCancelled.into());
                }
                eprintln!("[watch] Script exited with {status}; waiting for changes.");
            }
            EventSource::Changed => {
                if !debounce(changes, shutdown).await? {
                    return Err(ScriptCancelled.into());
                }
                if read_script(filepath).await? == source {
                    continue;
                }
                if let Some(mut previous) = script.take() {
                    let status = previous.stop(STOP_TIMEOUT).await?;
                    if status.code() == Some(i32::from(CANCELLED_EXIT_CODE)) {
                        return Err(ScriptCancelled.into());
                    }
                }
                // Saves may continue while the previous process is shutting down.
                if !debounce(changes, shutdown).await? {
                    return Err(ScriptCancelled.into());
                }
                source = read_script(filepath).await?;
                if source.is_some() {
                    eprintln!("[watch] Reloading {}", filepath.display());
                    *script = Some(RunningScript::start()?);
                } else {
                    eprintln!("[watch] Script was removed; waiting for it to reappear.");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, ffi::OsString, iter::once, path::Path, process::Stdio, thread, time::Duration};

    use clap::Parser;
    use notify::{Event, EventKind, event::AccessKind};
    use tokio::time::timeout;

    use super::{RunningScript, affects_script, child_command, connect_control, listen_for_stop};
    use crate::{
        args::{Args, Commands},
        maybe_insert_default_run,
    };

    #[test]
    fn parses_watch_with_implicit_and_explicit_run() {
        for arguments in [
            vec!["actiona-run", "--watch", "script.ts"],
            vec!["actiona-run", "script.ts", "--watch"],
            vec!["actiona-run", "run", "--watch", "script.ts"],
        ] {
            let arguments = arguments.into_iter().map(OsString::from).collect();
            let args =
                Args::try_parse_from(maybe_insert_default_run(arguments)).expect("parse watch");
            assert!(matches!(args.command, Commands::Run { watch: true, .. }));
        }
        assert!(Args::try_parse_from(["actiona-run", "eval", "--watch", "1"]).is_err());
    }

    #[test]
    fn forwards_run_options_without_recursively_watching() {
        for arguments in [
            vec![
                "--update-check=false",
                "run",
                "--watch",
                "--seed=42",
                "--no-tray",
                "--stop-hotkey=Ctrl+F12",
                "script with spaces.ts",
            ],
            vec![
                "--update-check",
                "false",
                "--watch",
                "--seed",
                "42",
                "--no-tray",
                "--stop-hotkey",
                "Ctrl+F12",
                "script with spaces.ts",
            ],
        ] {
            let command =
                child_command(arguments.into_iter().map(OsString::from), "private-socket")
                    .expect("build child");
            let forwarded = once(OsString::from("actiona-run"))
                .chain(command.as_std().get_args().map(OsString::from))
                .collect();
            let forwarded = Args::try_parse_from(maybe_insert_default_run(forwarded))
                .expect("parse forwarded options");
            assert_eq!(forwarded.update_check, Some(false));
            match forwarded.command {
                Commands::Run {
                    filepath,
                    watch,
                    watch_control,
                    run_args,
                    script_args,
                } => {
                    assert_eq!(filepath, Path::new("script with spaces.ts"));
                    assert!(!watch);
                    assert_eq!(watch_control.as_deref(), Some("private-socket"));
                    assert_eq!(run_args.seed, Some(42));
                    assert!(script_args.no_tray);
                    assert_eq!(script_args.stop_hotkey.as_deref(), Some("Ctrl+F12"));
                }
                other => panic!("expected run, got {other:?}"),
            }
        }
    }

    #[test]
    fn preserves_watch_as_a_filename_after_the_option_separator() {
        let command = child_command(
            ["run", "--watch", "--no-stop-hotkey", "--", "--watch"].map(OsString::from),
            "private-socket",
        )
        .expect("build child");
        let forwarded = once(OsString::from("actiona-run"))
            .chain(command.as_std().get_args().map(OsString::from));
        let forwarded = Args::try_parse_from(forwarded).expect("parse forwarded options");
        match forwarded.command {
            Commands::Run {
                filepath,
                watch,
                watch_control,
                script_args,
                ..
            } => {
                assert_eq!(filepath, Path::new("--watch"));
                assert!(!watch);
                assert_eq!(watch_control.as_deref(), Some("private-socket"));
                assert!(script_args.no_stop_hotkey);
            }
            other => panic!("expected run, got {other:?}"),
        }
    }

    #[test]
    fn ignores_access_and_unrelated_files() {
        let filepath = Path::new("script.ts");
        assert!(affects_script(
            &Event::new(EventKind::Any).add_path(filepath.into()),
            filepath
        ));
        assert!(!affects_script(
            &Event::new(EventKind::Any).add_path("index.d.ts".into()),
            filepath
        ));
        assert!(!affects_script(
            &Event::new(EventKind::Access(AccessKind::Read)).add_path(filepath.into()),
            filepath
        ));
    }

    #[tokio::test]
    async fn supervisor_disconnect_cancels_child() {
        use interprocess::local_socket::{
            GenericNamespaced, ListenerOptions, ToNsName, tokio::prelude::*,
        };
        use tokio_util::{sync::CancellationToken, task::TaskTracker};
        use uuid::Uuid;

        let name = format!("actiona-watch-test-{}", Uuid::new_v4());
        let listener = ListenerOptions::new()
            .name(
                name.as_str()
                    .to_ns_name::<GenericNamespaced>()
                    .expect("socket name"),
            )
            .create_tokio()
            .expect("create listener");
        let child = connect_control(&name).await.expect("connect child");
        let parent = listener.accept().await.expect("accept child");
        let cancellation = CancellationToken::new();
        let supervisor_stop = CancellationToken::new();
        let tasks = TaskTracker::new();
        listen_for_stop(child, cancellation.clone(), supervisor_stop.clone(), &tasks);
        assert!(!cancellation.is_cancelled());
        drop(parent);
        timeout(Duration::from_secs(2), cancellation.cancelled())
            .await
            .expect("child cancelled after disconnect");
        tasks.close();
        tasks.wait().await;
        assert!(supervisor_stop.is_cancelled());
    }

    #[tokio::test]
    async fn user_stop_takes_priority_over_supervisor_disconnect() {
        use interprocess::local_socket::{
            GenericNamespaced, ListenerOptions, ToNsName, tokio::prelude::*,
        };
        use tokio_util::{sync::CancellationToken, task::TaskTracker};
        use uuid::Uuid;

        let name = format!("actiona-watch-user-stop-test-{}", Uuid::new_v4());
        let listener = ListenerOptions::new()
            .name(
                name.as_str()
                    .to_ns_name::<GenericNamespaced>()
                    .expect("socket name"),
            )
            .create_tokio()
            .expect("create listener");
        let child = connect_control(&name).await.expect("connect child");
        let parent = listener.accept().await.expect("accept child");
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let supervisor_stop = CancellationToken::new();
        let tasks = TaskTracker::new();
        drop(parent);
        listen_for_stop(child, cancellation, supervisor_stop.clone(), &tasks);
        tasks.close();
        tasks.wait().await;
        assert!(!supervisor_stop.is_cancelled());
    }

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
            .args(["--ignored", "--exact", "watch::tests::unresponsive_worker"])
            .env("ACTIONA_WATCH_UNRESPONSIVE_TEST", "1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        #[cfg(unix)]
        command.process_group(0);
        let mut script = RunningScript {
            child: command.spawn().expect("start worker"),
            listener,
            control: None,
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
