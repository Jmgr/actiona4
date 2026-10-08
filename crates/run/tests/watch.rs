#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::os::unix::process::CommandExt as _;
    use std::{
        fs,
        io::{BufRead, BufReader, Read, Write},
        path::Path,
        process::{Child, Command, Stdio},
        sync::mpsc::{self, Receiver},
        thread::{self, JoinHandle},
        time::{Duration, Instant},
    };

    use actiona_common::sentry::DISABLE_CRASH_REPORTING_ENV;

    const WAIT: Duration = Duration::from_secs(20);

    struct WatchedScript {
        child: Child,
        output: Receiver<String>,
        readers: Vec<JoinHandle<()>>,
    }

    impl WatchedScript {
        fn start(filepath: &Path) -> Self {
            let mut command = Command::new(env!("CARGO_BIN_EXE_actiona-run"));
            command
                .args([
                    "--update-check",
                    "false",
                    "run",
                    "--watch",
                    "--no-tray",
                    "--no-stop-hotkey",
                ])
                .arg(filepath);
            command.env(DISABLE_CRASH_REPORTING_ENV, "1");
            command
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            #[cfg(unix)]
            command.process_group(0);
            let mut child = command.spawn().expect("start watched script");
            let (sender, output) = mpsc::channel();
            let stdout = child.stdout.take().expect("stdout");
            let stderr = child.stderr.take().expect("stderr");
            let readers = [Box::new(stdout) as Box<dyn Read + Send>, Box::new(stderr)]
                .into_iter()
                .map(|pipe| {
                    let sender = sender.clone();
                    thread::spawn(move || {
                        for line in BufReader::new(pipe).lines() {
                            let line = line.expect("read process output");
                            if sender.send(line).is_err() {
                                break;
                            }
                        }
                    })
                })
                .collect();
            Self {
                child,
                output,
                readers,
            }
        }

        fn wait_for(&self, expected: &str) {
            let deadline = Instant::now() + WAIT;
            let mut seen = Vec::new();
            loop {
                let line = self
                    .output
                    .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                    .unwrap_or_else(|error| {
                        panic!("waiting for {expected:?}: {error}; output: {seen:?}")
                    });
                if line.contains(expected) {
                    return;
                }
                seen.push(line);
            }
        }

        fn assert_no_run(&self, marker: &str) {
            let deadline = Instant::now() + Duration::from_millis(700);
            while let Ok(line) = self
                .output
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            {
                assert!(!line.contains(marker), "unexpected duplicate run: {line}");
            }
        }

        fn kill_supervisor(&mut self) {
            self.child.kill().expect("terminate supervisor");
            self.child.wait().expect("reap supervisor");
        }

        fn wait_for_output_closed(&self) {
            let deadline = Instant::now() + WAIT;
            loop {
                match self
                    .output
                    .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                {
                    Ok(_) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => return,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        panic!("script stayed alive after supervisor stopped")
                    }
                }
            }
        }
    }

    impl Drop for WatchedScript {
        fn drop(&mut self) {
            let _ = self.child.kill();
            let _ = self.child.wait();
            // Readers exit once both the supervisor and its child close their output pipes.
            if !thread::panicking() {
                for reader in self.readers.drain(..) {
                    reader.join().expect("output reader");
                }
            }
        }
    }

    #[test]
    fn reloads_after_completion_and_script_errors() {
        let directory = tempfile::tempdir().expect("script directory");
        let filepath = directory.path().join("script.ts");
        let original = "const value = 'first'; println(value);";
        fs::write(&filepath, original).expect("write script");
        let mut watched = WatchedScript::start(&filepath);
        watched.wait_for("first");
        watched.wait_for("waiting for changes");

        fs::write(&filepath, original).expect("save identical contents");
        fs::write(
            directory.path().join("unrelated.ts"),
            "println('unrelated');",
        )
        .expect("write unrelated file");
        watched.assert_no_run("first");

        fs::write(&filepath, "const value = ;").expect("write invalid script");
        watched.wait_for("Reloading");
        watched.wait_for("waiting for changes");

        fs::write(&filepath, "const value = 'recovered'; println(value);").expect("fix script");
        watched.wait_for("recovered");
        watched.wait_for("waiting for changes");
        watched.kill_supervisor();
        watched.wait_for_output_closed();
    }

    #[test]
    fn replacement_and_deletion_stop_the_previous_run() {
        let directory = tempfile::tempdir().expect("script directory");
        let filepath = directory.path().join("script.ts");
        fs::write(&filepath, "try { println('first running'); await sleep('1h'); } finally { println('first stopped'); }")
        .expect("write first script");
        let mut watched = WatchedScript::start(&filepath);
        watched.wait_for("first running");

        let mut replacement =
            tempfile::NamedTempFile::new_in(directory.path()).expect("replacement");
        write!(replacement, "try {{ println('second running'); await sleep('1h'); }} finally {{ println('second stopped'); }}")
        .expect("write replacement");
        replacement
            .persist(&filepath)
            .expect("replace script atomically");
        watched.wait_for("first stopped");
        watched.wait_for("second running");

        fs::remove_file(&filepath).expect("remove script");
        watched.wait_for("second stopped");
        watched.wait_for("waiting for it to reappear");
        fs::write(&filepath, "try { println('restored running'); await sleep('1h'); } finally { println('restored stopped'); }")
        .expect("restore script");
        watched.wait_for("restored running");

        watched.kill_supervisor();
        watched.wait_for("restored stopped");
        watched.wait_for_output_closed();
    }

    #[test]
    fn reload_interrupts_a_busy_loop() {
        let directory = tempfile::tempdir().expect("script directory");
        let filepath = directory.path().join("script.ts");
        fs::write(&filepath, "println('busy loop'); while (true) {}").expect("write busy loop");
        let mut watched = WatchedScript::start(&filepath);
        watched.wait_for("busy loop");
        fs::write(&filepath, "println('loop replaced');").expect("replace busy loop");
        watched.wait_for("loop replaced");
        watched.wait_for("waiting for changes");
        watched.kill_supervisor();
        watched.wait_for_output_closed();
    }

    #[cfg(windows)]
    #[test]
    fn reloads_with_differently_cased_filename() {
        let directory = tempfile::tempdir().expect("script directory");
        let filepath = directory.path().join("script-é.ts");
        fs::write(&filepath, "try { println('first running'); await sleep('1h'); } finally { println('first stopped'); }")
            .expect("write first script");
        let mut watched = WatchedScript::start(&directory.path().join("SCRIPT-É.TS"));
        watched.wait_for("first running");

        fs::write(&filepath, "try { println('second running'); await sleep('1h'); } finally { println('second stopped'); }")
            .expect("edit script using its on-disk spelling");
        watched.wait_for("first stopped");
        watched.wait_for("second running");

        fs::remove_file(&filepath).expect("remove script");
        watched.wait_for("second stopped");
        watched.wait_for("waiting for it to reappear");
        fs::write(directory.path().join("Script-É.ts"), "try { println('restored running'); await sleep('1h'); } finally { println('restored stopped'); }")
            .expect("restore script with different casing");
        watched.wait_for("restored running");

        watched.kill_supervisor();
        watched.wait_for("restored stopped");
        watched.wait_for_output_closed();
    }

    #[cfg(unix)]
    #[test]
    #[allow(unsafe_code)]
    fn ctrl_c_stops_the_supervisor_and_child() {
        let directory = tempfile::tempdir().expect("script directory");
        let filepath = directory.path().join("script.ts");
        fs::write(
            &filepath,
            "try { println('running'); await sleep('1h'); } finally { println('cancelled'); }",
        )
        .expect("write script");
        let mut watched = WatchedScript::start(&filepath);
        watched.wait_for("running");
        let pid = i32::try_from(watched.child.id()).expect("supervisor PID");
        // SAFETY: the supervisor owns this process group; SIGINT reproduces terminal Ctrl+C.
        assert_eq!(unsafe { libc::kill(-pid, libc::SIGINT) }, 0);
        watched.wait_for("cancelled");
        watched.wait_for_output_closed();
        let status = watched.child.wait().expect("reap supervisor");
        assert_eq!(status.code(), Some(i32::from(run::CANCELLED_EXIT_CODE)));
    }
}
