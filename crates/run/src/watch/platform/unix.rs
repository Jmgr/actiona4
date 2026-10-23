use std::{
    fs::{File, OpenOptions},
    io::{self, ErrorKind, IsTerminal, Read, stdin},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    process::Stdio,
};

use color_eyre::{Result, eyre::Context};
use tokio::{
    io::{AsyncWriteExt, unix::AsyncFd},
    process::{Child, ChildStdin, Command},
    task::JoinHandle,
};

use super::{RunningScriptBase, ScriptPlatform};

pub type RunningScript = RunningScriptBase<UnixScript>;

pub struct UnixScript {
    input_task: Option<JoinHandle<()>>,
}

impl Drop for UnixScript {
    fn drop(&mut self) {
        if let Some(task) = &self.input_task {
            task.abort();
        }
    }
}

impl ScriptPlatform for UnixScript {
    fn new(child: &mut Child) -> Result<Self> {
        Ok(Self {
            input_task: forward_terminal_input(child)?,
        })
    }

    fn configure_command(command: &mut Command) {
        // The background process group cannot read directly from the foreground terminal.
        if stdin().is_terminal() {
            command.stdin(Stdio::piped());
        }
        // Only the supervisor receives terminal Ctrl+C; the child stops through its socket.
        command.process_group(0);
    }

    #[allow(unsafe_code)]
    fn kill_child(child: &mut Child) -> Result<()> {
        #[cfg(linux)]
        if let Some(pid) = child.id() {
            let pid = i32::try_from(pid)?;
            // SAFETY: the child owns this process group; a negative PID targets the group,
            // including Linux dialog helpers that cannot clean up after a forced stop.
            let result = unsafe { libc::kill(-pid, libc::SIGKILL) };
            if result != 0 {
                child.start_kill().context("terminating watched script")?;
            }
        }
        #[cfg(not(linux))]
        child.start_kill().context("terminating watched script")?;

        Ok(())
    }
}

pub fn paths_match(left: &Path, right: &Path) -> bool {
    left == right
}

async fn forward_input(input: AsyncFd<File>, mut output: ChildStdin) -> io::Result<()> {
    let mut buffer = [0; 4096];
    loop {
        let mut ready = input.readable().await?;
        let count = match ready.try_io(|input| input.get_ref().read(&mut buffer)) {
            Ok(result) => result?,
            Err(_) => continue,
        };
        if count == 0 {
            return Ok(());
        }
        output.write_all(&buffer[..count]).await?;
    }
}

fn forward_terminal_input(child: &mut Child) -> Result<Option<JoinHandle<()>>> {
    let Some(output) = child.stdin.take() else {
        return Ok(None);
    };
    let input = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open("/dev/tty")
        .context("opening terminal input")?;
    #[allow(unsafe_code)]
    #[expect(
        clippy::undocumented_unsafe_blocks,
        reason = "AsyncFd owns the live File, which is never replaced or closed while registered"
    )]
    let input = unsafe { AsyncFd::register(input) }.map_err(io::Error::from)?;
    Ok(Some(tokio::spawn(async move {
        if let Err(error) = forward_input(input, output).await
            && error.kind() != ErrorKind::BrokenPipe
        {
            eprintln!("[watch] Could not forward terminal input: {error}");
        }
    })))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    #[cfg(linux)]
    use std::{env, ffi::OsString, process::Stdio};

    use tokio::time::timeout;

    #[cfg(linux)]
    use crate::watch::child_command;
    use crate::watch::{ScriptWatcher, check_change, debounce, read_script};

    #[tokio::test]
    async fn watches_symlink_targets_and_refreshes_retargeted_links() {
        use std::{fs, os::unix::fs::symlink};

        use tokio_util::sync::CancellationToken;

        let directory = tempfile::tempdir().expect("script directory");
        let first_directory = directory.path().join("first");
        let second_directory = directory.path().join("second");
        fs::create_dir(&first_directory).expect("first target directory");
        fs::create_dir(&second_directory).expect("second target directory");
        let first = first_directory.join("actual.ts");
        let second = second_directory.join("actual.ts");
        let intermediate = directory.path().join("intermediate.ts");
        let filepath = directory.path().join("linked.ts");
        fs::write(&first, "first").expect("write first target");
        fs::write(&second, "second").expect("write second target");
        symlink("first/actual.ts", &intermediate).expect("relative target link");
        symlink("intermediate.ts", &filepath).expect("script link");
        let (mut watcher, mut changes) = ScriptWatcher::new(&filepath).expect("watch link");

        fs::write(&first, "second").expect("edit symlink target");
        check_change(
            timeout(Duration::from_secs(2), changes.recv())
                .await
                .expect("target edit detected"),
        )
        .expect("target event");

        fs::remove_file(&intermediate).expect("remove intermediate link");
        symlink("second/actual.ts", &intermediate).expect("retarget intermediate link");
        assert!(
            debounce(&mut changes, &CancellationToken::new())
                .await
                .expect("drain retarget events")
        );
        watcher.refresh(&filepath).expect("refresh link targets");
        assert_eq!(
            read_script(&filepath).await.expect("read link"),
            Some(b"second".to_vec())
        );

        fs::remove_file(&second).expect("remove target");
        check_change(
            timeout(Duration::from_secs(2), changes.recv())
                .await
                .expect("target deletion detected"),
        )
        .expect("deletion event");
        watcher
            .refresh(&filepath)
            .expect("keep watching missing target");
        debounce(&mut changes, &CancellationToken::new())
            .await
            .expect("drain deletion events");
        fs::write(&second, "restored").expect("restore target");
        check_change(
            timeout(Duration::from_secs(2), changes.recv())
                .await
                .expect("target restoration detected"),
        )
        .expect("restoration event");
    }

    #[cfg(linux)]
    #[tokio::test]
    async fn forwards_terminal_input_without_suspending_the_child() {
        use tokio::{io::AsyncReadExt, process::Command};

        let mut command = Command::new(env::current_exe().expect("test executable"));
        command
            .args([
                "--ignored",
                "--exact",
                "watch::platform::unix::tests::terminal_supervisor",
            ])
            .env("ACTIONA_WATCH_TERMINAL_TEST", "1")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = command.spawn().expect("start terminal supervisor");
        let stdout = child.stdout.take().expect("worker stdout");
        let stderr = child.stderr.take().expect("worker stderr");
        let output = tokio::spawn(async move {
            let mut output = Vec::new();
            stdout
                .chain(stderr)
                .read_to_end(&mut output)
                .await
                .expect("worker output");
            String::from_utf8_lossy(&output).into_owned()
        });
        let status = timeout(Duration::from_secs(5), child.wait())
            .await
            .expect("terminal worker deadline")
            .expect("terminal worker status");
        let output = output.await.expect("output task");
        assert!(
            status.success(),
            "terminal supervisor failed ({status}): {output}"
        );
    }

    #[cfg(linux)]
    #[tokio::test]
    #[ignore = "isolated controlling-terminal fixture"]
    #[allow(unsafe_code)]
    async fn terminal_supervisor() {
        use std::{
            fs::File,
            io::Write,
            os::fd::{AsRawFd, FromRawFd},
            ptr::{null, null_mut},
        };

        if env::var_os("ACTIONA_WATCH_TERMINAL_TEST").is_none() {
            return;
        }
        let mut master = -1;
        let mut slave = -1;
        // SAFETY: this subprocess creates its own session and terminal; openpty initializes both FDs.
        unsafe {
            // Closing the fixture's terminal must not terminate its test harness.
            assert_ne!(libc::signal(libc::SIGHUP, libc::SIG_IGN), libc::SIG_ERR);
            assert!(libc::setsid() >= 0);
            assert_eq!(
                libc::openpty(&raw mut master, &raw mut slave, null_mut(), null(), null()),
                0
            );
            assert_eq!(libc::ioctl(slave, libc::TIOCSCTTY, 0), 0);
            assert_eq!(libc::dup2(slave, libc::STDIN_FILENO), libc::STDIN_FILENO);
        }
        // SAFETY: each freshly opened descriptor is owned exactly once by its File.
        let (mut master, slave) = unsafe { (File::from_raw_fd(master), File::from_raw_fd(slave)) };
        let mut command = child_command(
            [
                "--ignored",
                "--exact",
                "watch::platform::unix::tests::terminal_reader",
                "--nocapture",
            ]
            .map(OsString::from),
            "unused-control",
        )
        .expect("build background child");
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = command.spawn().expect("start background reader");
        let forwarding = super::forward_terminal_input(&mut child)
            .expect("forward terminal input")
            .expect("terminal stdin was piped");
        // SAFETY: these queries use live descriptors and the PID of our own child.
        unsafe {
            assert_ne!(
                libc::tcgetpgrp(slave.as_raw_fd()),
                libc::getpgid(i32::try_from(child.id().expect("child PID")).expect("PID range"))
            );
            assert_eq!(
                libc::fcntl(slave.as_raw_fd(), libc::F_GETFL) & libc::O_NONBLOCK,
                0
            );
        }
        master
            .write_all(b"terminal input\n")
            .expect("terminal line");
        master.write_all(&[4]).expect("terminal EOF");
        let output = timeout(Duration::from_secs(2), child.wait_with_output())
            .await
            .expect("background reader completed")
            .expect("reader output");
        forwarding.await.expect("forwarding task completed");
        assert!(
            output.status.success(),
            "reader failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("terminal input"));
    }

    #[cfg(linux)]
    #[test]
    #[ignore = "background terminal-reader fixture"]
    fn terminal_reader() {
        use std::io::{Read, stdin};

        if env::var_os("ACTIONA_WATCH_TERMINAL_TEST").is_some() {
            let mut input = String::new();
            stdin()
                .read_to_string(&mut input)
                .expect("read forwarded stdin");
            assert_eq!(input, "terminal input\n");
            println!("{input}");
        }
    }
}
