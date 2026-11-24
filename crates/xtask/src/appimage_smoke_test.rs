use std::{
    path::Path,
    process::{Command, Stdio},
};

use color_eyre::{Result, eyre::eyre};

use crate::appimage::{appimage_arch, read_version};

const SMOKE_TEST_IMAGE: &str = "debian:trixie";
const SMOKE_TEST_PACKAGES: [&str; 7] = [
    "dbus-x11",
    "libegl1",
    "libgl1",
    "libvulkan1",
    "mesa-vulkan-drivers",
    "xauth",
    "xvfb",
];
const CONTAINER_APPIMAGE_PATH: &str = "/appimage/editor.AppImage";
const CONTAINER_WORK_DIR: &str = "/work";

/// Starts the packaged editor in a clean container. The initialization-only
/// editor exits successfully after GPUI Kit starts.
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

    println!("Starting the editor...");
    container.run(&[
        "timeout",
        "30",
        "xvfb-run",
        "-a",
        "dbus-launch",
        "--exit-with-session",
        CONTAINER_APPIMAGE_PATH,
    ])?;
    println!("Editor AppImage smoke test passed.");

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
