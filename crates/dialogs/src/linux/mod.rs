//! Linux backends.
//!
//! Dialogs are shown by the `zenity` or `kdialog` command-line tools. Each one runs the tool as a
//! child process with `kill_on_drop`, so dropping the dialog's future kills the process, which
//! closes its window.
//!
//! File dialogs use the xdg-desktop-portal file chooser when it is available, and fall back to
//! the tool otherwise.

use std::{
    env,
    ffi::{OsStr, OsString},
    io,
    os::unix::{ffi::OsStringExt, fs::PermissionsExt},
    path::{Path, PathBuf},
    process::Stdio,
    sync::Arc,
};

use jiff::civil::Date;
use strum::{AsRefStr, Display, EnumIs, EnumIter, EnumString};
use tokio::{
    io::AsyncWriteExt,
    process::{Child, Command},
    sync::OnceCell,
};
use tracing::{debug, warn};
use types::Color;

use crate::{
    ColorPickerOptions, DateOptions, Error, FileDialogOptions, MessageBoxOptions, MessageBoxResult,
    Progress, ProgressOptions, Result, SelectOptions, TextInputOptions, options::OpenMode,
};

mod kdialog;
mod portal;
mod zenity;

pub use kdialog::KDialog;
use portal::Portal;
pub use zenity::Zenity;

/// Forces a backend: `zenity` or `kdialog` for every dialog, or `portal` for file dialogs only.
const BACKEND_ENV_VAR: &str = "ACTIONA_DIALOGS_BACKEND";

/// Dialogs a command-line tool can show.
#[static_dispatch::setup]
pub trait Tool {
    async fn message_box(&self, options: &MessageBoxOptions) -> Result<MessageBoxResult>;
    async fn text_input(&self, options: &TextInputOptions) -> Result<Option<String>>;
    async fn color_picker(&self, options: &ColorPickerOptions) -> Result<Option<Color>>;
    async fn open(
        &self,
        options: &FileDialogOptions,
        mode: OpenMode,
    ) -> Result<Option<Vec<PathBuf>>>;
    async fn save(&self, options: &FileDialogOptions) -> Result<Option<PathBuf>>;
    /// Returns the indices of the selected items.
    async fn select(&self, options: &SelectOptions, multiple: bool) -> Result<Option<Vec<usize>>>;
    async fn date(&self, options: &DateOptions) -> Result<Option<Date>>;
    async fn progress(&self, options: &ProgressOptions) -> Result<Progress>;
}

/// Command-line tool used to show dialogs. Its string form is the program name.
#[derive(AsRefStr, Clone, Copy, Debug, Display, EnumIs, EnumIter, EnumString, Eq, PartialEq)]
#[strum(serialize_all = "lowercase")]
#[static_dispatch::setup]
pub enum LinuxTool {
    Zenity(Zenity),
    KDialog(KDialog),
}

static_dispatch::implementation!(Tool for LinuxTool);

impl LinuxTool {
    /// Whether the tool's program is in `PATH`.
    #[must_use]
    pub fn is_installed(self) -> bool {
        env::var_os("PATH").is_some_and(|paths| {
            env::split_paths(&paths).any(|directory| is_executable(&directory.join(self.as_ref())))
        })
    }
}

/// Backends used to show dialogs on Linux.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LinuxBackends {
    /// Whether file dialogs use the xdg-desktop-portal file chooser when it is available.
    pub portal: bool,
    /// Tool used to show dialogs, or `None` if neither is installed.
    pub tool: Option<LinuxTool>,
}

impl LinuxBackends {
    /// Uses the backend forced by `ACTIONA_DIALOGS_BACKEND` if set. Otherwise uses the portal for
    /// file dialogs, and the installed tool that matches the desktop: kdialog on KDE, zenity
    /// elsewhere.
    fn detect() -> Self {
        if let Some(value) = env::var_os(BACKEND_ENV_VAR) {
            match value.to_str() {
                Some("portal") => {
                    return Self {
                        portal: true,
                        tool: None,
                    };
                }
                Some(tool) => {
                    if let Ok(tool) = tool.parse() {
                        return Self {
                            portal: false,
                            tool: Some(tool),
                        };
                    }
                }
                None => {}
            }
            warn!(
                "ignoring unknown {BACKEND_ENV_VAR} value {value:?}; expected zenity, kdialog or portal"
            );
        }

        let is_kde = env::var("XDG_CURRENT_DESKTOP").is_ok_and(|desktops| {
            desktops
                .split(':')
                .any(|desktop| desktop.eq_ignore_ascii_case("KDE"))
        });
        let preference = if is_kde {
            [LinuxTool::KDialog(KDialog), LinuxTool::Zenity(Zenity)]
        } else {
            [LinuxTool::Zenity(Zenity), LinuxTool::KDialog(KDialog)]
        };

        Self {
            portal: true,
            tool: preference.into_iter().find(|tool| tool.is_installed()),
        }
    }
}

/// Linux backends, and the portal connection once made.
#[derive(Clone, Debug)]
pub struct Linux {
    backends: LinuxBackends,
    /// `None` inside once connecting failed, so it is only attempted once.
    portal: Arc<OnceCell<Option<Portal>>>,
}

impl Linux {
    pub fn detect() -> Self {
        Self::new(LinuxBackends::detect())
    }

    pub fn new(backends: LinuxBackends) -> Self {
        Self {
            backends,
            portal: Arc::default(),
        }
    }

    fn tool(&self) -> Result<LinuxTool> {
        self.backends.tool.ok_or(Error::NoBackend)
    }

    pub async fn message_box(&self, options: &MessageBoxOptions) -> Result<MessageBoxResult> {
        self.tool()?.message_box(options).await
    }

    pub async fn text_input(&self, options: &TextInputOptions) -> Result<Option<String>> {
        self.tool()?.text_input(options).await
    }

    pub async fn color_picker(&self, options: &ColorPickerOptions) -> Result<Option<Color>> {
        self.tool()?.color_picker(options).await
    }

    pub async fn select(
        &self,
        options: &SelectOptions,
        multiple: bool,
    ) -> Result<Option<Vec<usize>>> {
        self.tool()?.select(options, multiple).await
    }

    pub async fn date(&self, options: &DateOptions) -> Result<Option<Date>> {
        self.tool()?.date(options).await
    }

    pub async fn progress(&self, options: &ProgressOptions) -> Result<Progress> {
        self.tool()?.progress(options).await
    }

    async fn portal(&self) -> Option<&Portal> {
        if !self.backends.portal {
            return None;
        }

        self.portal.get_or_init(Portal::connect).await.as_ref()
    }

    pub async fn open(
        &self,
        options: &FileDialogOptions,
        mode: OpenMode,
    ) -> Result<Option<Vec<PathBuf>>> {
        if let Some(portal) = self.portal().await
            && portal.supports(mode)
        {
            return portal.open(options, mode).await;
        }

        self.tool()?.open(options, mode).await
    }

    pub async fn save(&self, options: &FileDialogOptions) -> Result<Option<PathBuf>> {
        if let Some(portal) = self.portal().await {
            return portal.save(options).await;
        }

        self.tool()?.save(options).await
    }
}

fn is_executable(path: &Path) -> bool {
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

/// Exit status and standard output of a finished tool.
#[derive(Debug)]
struct ToolOutput {
    /// `None` if the process was terminated by a signal.
    status: Option<i32>,
    /// Standard output, without the trailing newline the tools print.
    stdout: String,
    /// The same, as bytes: paths printed by the tools need not be valid UTF-8.
    stdout_bytes: Vec<u8>,
}

impl ToolOutput {
    #[cfg(test)]
    fn new(status: i32, stdout: &str) -> Self {
        Self {
            status: Some(status),
            stdout: stdout.to_owned(),
            stdout_bytes: stdout.as_bytes().to_vec(),
        }
    }

    fn unexpected(&self, tool: LinuxTool) -> Error {
        match self.status {
            Some(status) => Error::Backend(format!("{tool} exited with status {status}")),
            None => Error::Backend(format!("{tool} was terminated by a signal")),
        }
    }
}

impl ToolOutput {
    /// The path printed by a dialog that selects one: `None` if the user cancelled. The whole
    /// output is the path, which can contain line breaks.
    fn path(self, tool: LinuxTool) -> Result<Option<PathBuf>> {
        match self.status {
            Some(0) => Ok(Some(OsString::from_vec(self.stdout_bytes).into())),
            Some(1) => Ok(None),
            _ => Err(self.unexpected(tool)),
        }
    }

    /// Paths printed one per line: `None` if the user cancelled. A path containing a line break
    /// cannot be told apart from several paths.
    fn paths(self, tool: LinuxTool) -> Result<Option<Vec<PathBuf>>> {
        match self.status {
            Some(0) => Ok(Some(
                self.stdout_bytes
                    .split(|byte| *byte == b'\n')
                    .filter(|line| !line.is_empty())
                    .map(|line| OsString::from_vec(line.to_vec()).into())
                    .collect(),
            )),
            Some(1) => Ok(None),
            _ => Err(self.unexpected(tool)),
        }
    }

    /// Indices below `count` printed one per line: `None` if the user cancelled.
    fn indices(self, tool: LinuxTool, count: usize) -> Result<Option<Vec<usize>>> {
        match self.status {
            Some(0) => self
                .stdout
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| {
                    line.trim()
                        .parse()
                        .ok()
                        .filter(|index| *index < count)
                        .ok_or_else(|| {
                            Error::Backend(format!("{tool} returned an invalid item: {line:?}"))
                        })
                })
                .collect::<Result<Vec<_>>>()
                .map(Some),
            Some(1) => Ok(None),
            _ => Err(self.unexpected(tool)),
        }
    }

    /// A date printed as `YYYY-MM-DD`: `None` if the user cancelled.
    fn date(&self, tool: LinuxTool) -> Result<Option<Date>> {
        match self.status {
            Some(0) => self.stdout.trim().parse().map(Some).map_err(|_| {
                Error::Backend(format!(
                    "{tool} returned an invalid date: {:?}",
                    self.stdout
                ))
            }),
            Some(1) => Ok(None),
            _ => Err(self.unexpected(tool)),
        }
    }
}

/// Whether each item starts out selected. A single selection always starts with an item
/// selected: the first one, unless told otherwise.
fn preselected(options: &SelectOptions, multiple: bool) -> Vec<bool> {
    let single = options.selected.first().copied().unwrap_or(0);
    (0..options.items.len())
        .map(|index| {
            if multiple {
                options.selected.contains(&index)
            } else {
                index == single
            }
        })
        .collect()
}

fn spawn_error(error: io::Error) -> Error {
    match error.kind() {
        io::ErrorKind::NotFound => Error::NoBackend,
        _ => error.into(),
    }
}

/// Starts a program that keeps running while its dialog is open, with its standard input piped
/// if `stdin` is set. The process is killed when the returned `Child` is dropped.
fn spawn(program: &str, args: Vec<OsString>, stdin: bool) -> Result<Child> {
    Command::new(program)
        .args(args)
        .stdin(if stdin { Stdio::piped() } else { Stdio::null() })
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(spawn_error)
}

/// Runs a tool to completion, writing `stdin` to its standard input if given.
///
/// The process is killed if the returned future is dropped.
async fn run(tool: LinuxTool, args: Vec<OsString>, stdin: Option<String>) -> Result<ToolOutput> {
    run_program(tool.as_ref(), args, stdin).await
}

async fn run_program(
    program: &str,
    args: Vec<OsString>,
    stdin: Option<String>,
) -> Result<ToolOutput> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(spawn_error)?;

    if let Some(input) = stdin
        && let Some(mut pipe) = child.stdin.take()
    {
        pipe.write_all(input.as_bytes()).await?;
        // Dropping the pipe closes it, which tells the tool the input is complete.
    }

    let output = child.wait_with_output().await?;

    if !output.stderr.is_empty() {
        debug!(
            "{program} stderr: {}",
            String::from_utf8_lossy(&output.stderr).trim_end()
        );
    }

    let mut stdout_bytes = output.stdout;
    if stdout_bytes.ends_with(b"\n") {
        stdout_bytes.pop();
    }

    Ok(ToolOutput {
        status: output.status.code(),
        stdout: String::from_utf8_lossy(&stdout_bytes).into_owned(),
        stdout_bytes,
    })
}

/// Builds a `--name=value` argument, a form that cannot be mistaken for another option even when
/// the value starts with `-`.
fn option(name: &str, value: impl AsRef<OsStr>) -> OsString {
    let mut argument = OsString::from(name);
    argument.push("=");
    argument.push(value);
    argument
}

/// Formats a colour as `#RRGGBB`, ignoring alpha.
fn format_color(color: Color) -> String {
    format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2])
}

/// Parses `#RRGGBB`, `#RRGGBBAA`, `rgb(r, g, b)` or `rgba(r, g, b, a)`. The result is always
/// opaque.
fn parse_color(value: &str) -> Option<Color> {
    let value = value.trim();

    if let Some(channels) = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))
        .and_then(|channels| channels.strip_suffix(')'))
    {
        let mut channels = channels.split(',').map(str::trim);
        let red = channels.next()?.parse().ok()?;
        let green = channels.next()?.parse().ok()?;
        let blue = channels.next()?.parse().ok()?;
        return Some(Color::new(red, green, blue, 255));
    }

    let hex = value.strip_prefix('#')?;
    if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let value = u32::from_str_radix(hex, 16).ok()?;
    let [red, green, blue] = match hex.len() {
        6 => {
            let [_, red, green, blue] = value.to_be_bytes();
            [red, green, blue]
        }
        8 => {
            let [red, green, blue, _] = value.to_be_bytes();
            [red, green, blue]
        }
        _ => return None,
    };

    Some(Color::new(red, green, blue, 255))
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

    use jiff::civil::date;
    use types::Color;

    use super::{
        KDialog, LinuxTool, ToolOutput, Zenity, format_color, parse_color, preselected, run_program,
    };
    use crate::{Error, SelectOptions};

    const TOOL: LinuxTool = LinuxTool::Zenity(Zenity);

    fn bytes_output(stdout: &[u8]) -> ToolOutput {
        ToolOutput {
            status: Some(0),
            stdout: String::from_utf8_lossy(stdout).into_owned(),
            stdout_bytes: stdout.to_vec(),
        }
    }

    fn path(bytes: &[u8]) -> PathBuf {
        OsString::from_vec(bytes.to_vec()).into()
    }

    #[test]
    fn single_paths_are_kept_whole() {
        assert_eq!(
            bytes_output(b"/tmp/a\nb\xff").path(TOOL).unwrap(),
            Some(path(b"/tmp/a\nb\xff"))
        );
        assert_eq!(ToolOutput::new(1, "").path(TOOL).unwrap(), None);
        assert!(ToolOutput::new(5, "").path(TOOL).is_err());
    }

    #[test]
    fn several_paths_keep_their_bytes() {
        assert_eq!(
            bytes_output(b"/tmp/a\xff\n/tmp/b").paths(TOOL).unwrap(),
            Some(vec![path(b"/tmp/a\xff"), path(b"/tmp/b")])
        );
    }

    #[test]
    fn parses_indices() {
        assert_eq!(
            ToolOutput::new(0, "2\n0\n").indices(TOOL, 3).unwrap(),
            Some(vec![2, 0])
        );
        assert_eq!(
            ToolOutput::new(0, "").indices(TOOL, 3).unwrap(),
            Some(vec![])
        );
        assert_eq!(ToolOutput::new(1, "").indices(TOOL, 3).unwrap(), None);
        assert!(ToolOutput::new(0, "3").indices(TOOL, 3).is_err());
        assert!(ToolOutput::new(0, "x").indices(TOOL, 3).is_err());
    }

    #[test]
    fn parses_dates() {
        assert_eq!(
            ToolOutput::new(0, "2026-10-06").date(TOOL).unwrap(),
            Some(date(2026, 10, 6))
        );
        assert_eq!(ToolOutput::new(1, "").date(TOOL).unwrap(), None);
        assert!(ToolOutput::new(0, "06/10/2026").date(TOOL).is_err());
    }

    #[test]
    fn preselects_items() {
        let mut options = SelectOptions {
            items: vec!["a".to_owned(), "b".to_owned(), "c".to_owned()],
            ..SelectOptions::default()
        };

        assert_eq!(preselected(&options, false), [true, false, false]);
        assert_eq!(preselected(&options, true), [false, false, false]);

        options.selected = vec![2, 1];
        assert_eq!(preselected(&options, false), [false, false, true]);
        assert_eq!(preselected(&options, true), [false, true, true]);
    }

    #[test]
    fn tool_names_are_program_names() {
        assert_eq!(LinuxTool::Zenity(Zenity).as_ref(), "zenity");
        assert_eq!(LinuxTool::KDialog(KDialog).as_ref(), "kdialog");
        assert_eq!(
            "kdialog".parse::<LinuxTool>(),
            Ok(LinuxTool::KDialog(KDialog))
        );
        assert!("KDialog".parse::<LinuxTool>().is_err());
    }

    #[test]
    fn formats_color_without_alpha() {
        assert_eq!(format_color(Color::new(1, 171, 255, 10)), "#01ABFF");
    }

    #[test]
    fn parses_colors() {
        let expected = Some(Color::new(1, 171, 255, 255));

        assert_eq!(parse_color("#01abff"), expected);
        assert_eq!(parse_color("#01ABFF80"), expected);
        assert_eq!(parse_color("rgb(1,171,255)"), expected);
        assert_eq!(parse_color(" rgba(1, 171, 255, 0.5)\n"), expected);
        assert_eq!(parse_color("01abff"), None);
        assert_eq!(parse_color("#01abf"), None);
        assert_eq!(parse_color("#+1abff"), None);
        assert_eq!(parse_color("rgb(1,171)"), None);
    }

    #[tokio::test]
    async fn run_reports_status_and_stdout() {
        let output = run_program(
            "sh",
            vec!["-c".into(), "cat; printf ' out\\n'; exit 3".into()],
            Some("in".to_owned()),
        )
        .await
        .unwrap();

        assert_eq!(output.status, Some(3));
        assert_eq!(output.stdout, "in out");
    }

    #[tokio::test]
    async fn run_keeps_stdout_bytes() {
        let output = run_program("sh", vec!["-c".into(), "printf '/a\\377\\n'".into()], None)
            .await
            .unwrap();

        assert_eq!(output.stdout_bytes, b"/a\xff");
    }

    #[tokio::test]
    async fn run_reports_missing_tool_as_no_backend() {
        let error = run_program("actiona-dialogs-missing-tool", Vec::new(), None)
            .await
            .unwrap_err();

        assert!(matches!(error, Error::NoBackend));
    }
}
