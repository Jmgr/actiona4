use std::{
    collections::HashMap,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use color_eyre::{Result, eyre::eyre};

use crate::appimage::{appimage_arch, read_version};

/// A distribution without WebKitGTK, so the editor can only work with its bundled runtime.
const SMOKE_TEST_IMAGE: &str = "debian:trixie";
/// The base libraries an AppImage expects the desktop to provide (linuxdeploy's excludelist) and
/// what is needed to run headless. Deliberately no WebKitGTK or libadwaita: those have to come
/// from the AppImage.
const SMOKE_TEST_PACKAGES: [&str; 10] = [
    "dbus-x11",
    "libdrm2",
    "libegl1",
    "libgbm1",
    "libgl1",
    "libgpg-error0",
    "libgtk-4-1",
    "libx11-xcb1",
    "xauth",
    "xvfb",
];
const CONTAINER_APPIMAGE_PATH: &str = "/appimage/editor.AppImage";
const CONTAINER_WORK_DIR: &str = "/work";
/// How long the editor runs before it is stopped, and when its processes are inspected.
const EDITOR_RUN_TIME: Duration = Duration::from_secs(30);
const EDITOR_STARTUP_TIME: Duration = Duration::from_secs(20);

/// Starts the editor AppImage in a clean container and checks that its WebKit helper processes
/// run from the bundled runtime. Needs Docker and a previously built AppImage.
pub async fn smoke_test_editor_appimage(workspace_root: &Path) -> Result<()> {
    let version = read_version(workspace_root).await?;
    let appimage_path = workspace_root.join("target").join(format!(
        "actiona-editor-{version}-{}.AppImage",
        appimage_arch()?
    ));

    if !appimage_path.is_file() {
        return Err(eyre!(
            "{} not found; build it first with `cargo make appimage-nosign`",
            appimage_path.display()
        ));
    }

    let container = Container::start(&appimage_path)?;

    println!("Installing the base desktop libraries in {SMOKE_TEST_IMAGE}...");
    container.run(&["apt-get", "update", "-qq"])?;
    container.run(
        &["apt-get", "install", "-y", "-qq", "--no-install-recommends"]
            .into_iter()
            .chain(SMOKE_TEST_PACKAGES)
            .collect::<Vec<_>>(),
    )?;

    let host_webkit = container.output(&["find", "/usr/lib", "-name", "libwebkitgtk-*"])?;
    if !host_webkit.trim().is_empty() {
        return Err(eyre!(
            "The container provides WebKitGTK, so the test would not exercise the bundled copy:\n\
             {host_webkit}"
        ));
    }

    println!("Starting the editor...");
    let run_time = EDITOR_RUN_TIME.as_secs().to_string();
    let editor = container
        .exec(&[
            "timeout",
            &run_time,
            "xvfb-run",
            "-a",
            "dbus-launch",
            "--exit-with-session",
            CONTAINER_APPIMAGE_PATH,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    thread::sleep(EDITOR_STARTUP_TIME);
    let inspection = inspect_editor_processes(&container);

    // Waits for `timeout` to stop the editor.
    let output = editor.wait_with_output()?;
    let editor_log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let result = if ["error while loading shared libraries", "panicked"]
        .iter()
        .any(|error| editor_log.contains(error))
    {
        Err(eyre!("The editor or a WebKit helper failed to start."))
    } else {
        inspection
    };

    result.map_err(|error| {
        eyre!("Editor AppImage smoke test failed: {error}\n--- editor output:\n{editor_log}")
    })?;
    println!("Editor AppImage smoke test passed.");

    Ok(())
}

fn inspect_editor_processes(container: &Container) -> Result<()> {
    let processes = container.processes()?;
    let pid_of = |executable: &str| {
        processes
            .get(executable)
            .ok_or_else(|| eyre!("{executable} is not running."))
    };

    let editor_pid = pid_of("actiona-editor")?;
    let editor_cwd = container.output(&["readlink", &format!("/proc/{editor_pid}/cwd")])?;
    if editor_cwd.trim() != CONTAINER_WORK_DIR {
        return Err(eyre!(
            "actiona-editor runs in {}, not in the caller's working directory {CONTAINER_WORK_DIR}.",
            editor_cwd.trim()
        ));
    }

    pid_of("WebKitNetworkProcess")?;
    let web_process_pid = pid_of("WebKitWebProcess")?;
    let web_process_maps = container.output(&["cat", &format!("/proc/{web_process_pid}/maps")])?;
    if !web_process_maps.contains("/libwebkitgtkinjectedbundle.so") {
        return Err(eyre!(
            "WebKitWebProcess did not load the bundled injected bundle."
        ));
    }

    Ok(())
}

/// A detached container, removed when dropped.
struct Container {
    id: String,
}

impl Container {
    fn start(appimage_path: &Path) -> Result<Self> {
        let output = Command::new("docker")
            .args(["run", "--detach", "--rm", "--volume"])
            .arg(format!(
                "{}:{CONTAINER_APPIMAGE_PATH}:ro",
                appimage_path.display()
            ))
            // FUSE is not available in the container.
            .args(["--env", "APPIMAGE_EXTRACT_AND_RUN=1"])
            .args(["--env", "DEBIAN_FRONTEND=noninteractive"])
            .args(["--workdir", CONTAINER_WORK_DIR])
            .args([SMOKE_TEST_IMAGE, "sleep", "infinity"])
            .output()
            .map_err(|error| eyre!("Failed to run docker: {error}"))?;

        if !output.status.success() {
            return Err(eyre!(
                "Failed to start the {SMOKE_TEST_IMAGE} container: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(Self {
            id: String::from_utf8(output.stdout)?.trim().to_owned(),
        })
    }

    fn exec(&self, args: &[&str]) -> Command {
        let mut command = Command::new("docker");
        command.args(["exec", &self.id]).args(args);
        command
    }

    /// Runs `args` in the container, only showing its output if it fails.
    fn run(&self, args: &[&str]) -> Result<()> {
        self.output(args).map(|_| ())
    }

    fn output(&self, args: &[&str]) -> Result<String> {
        let output = self.exec(args).output()?;

        if output.status.success() {
            Ok(String::from_utf8(output.stdout)?)
        } else {
            Err(eyre!(
                "`{}` failed in the container with status {}:\n{}{}",
                args.join(" "),
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Maps the file name of each running executable to its PID.
    fn processes(&self) -> Result<HashMap<String, String>> {
        // Processes can exit while find walks /proc, which makes it fail; what it did print is
        // still accurate.
        let output = self
            .exec(&[
                "find",
                "/proc",
                "-mindepth",
                "2",
                "-maxdepth",
                "2",
                "-name",
                "exe",
                "-printf",
                "%h %l\n",
            ])
            .output()?;

        Ok(String::from_utf8(output.stdout)?
            .lines()
            .filter_map(|line| {
                let (proc_dir, executable) = line.split_once(' ')?;
                let pid = proc_dir.strip_prefix("/proc/")?;
                let name = Path::new(executable).file_name()?.to_str()?;
                Some((name.to_owned(), pid.to_owned()))
            })
            .collect())
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        let _ = Command::new("docker")
            .args(["rm", "--force", &self.id])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}
