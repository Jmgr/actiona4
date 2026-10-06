//! Dialogs shown by `zenity`.
//!
//! Exit status 0 means the user accepted the dialog and 1 that they cancelled or closed it. In a
//! message box, extra buttons also exit with status 1, but print their label.

use std::{
    ffi::OsString,
    fmt::Write,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use jiff::civil::Date;
use tokio::{
    io::AsyncWriteExt,
    process::{Child, ChildStdin},
    select,
    sync::{OnceCell, watch},
};
use tokio_util::sync::CancellationToken;
use tracing::debug;
use types::Color;

use super::{
    LinuxTool, Tool, ToolOutput, format_color, option, parse_color, preselected, run, spawn,
};
use crate::{
    ButtonLabels, ColorPickerOptions, DateOptions, Error, FileDialogOptions, FileFilter,
    MessageBoxIcon, MessageBoxOptions, MessageBoxResult, Progress, ProgressOptions, Result,
    SelectOptions, TextInputMode, TextInputOptions, options::OpenMode, progress::ProgressState,
};

/// Shows dialogs with `zenity`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Zenity;

const TOOL: LinuxTool = LinuxTool::Zenity(Zenity);

impl Tool for Zenity {
    async fn message_box(&self, options: &MessageBoxOptions) -> Result<MessageBoxResult> {
        let output = run(TOOL, message_box_args(options), None).await?;
        message_box_result(options, &output)
    }

    async fn text_input(&self, options: &TextInputOptions) -> Result<Option<String>> {
        let (args, stdin) = text_input_args(options);
        let output = run(TOOL, args, stdin).await?;
        text_input_result(output)
    }

    async fn color_picker(&self, options: &ColorPickerOptions) -> Result<Option<Color>> {
        let output = run(TOOL, color_picker_args(options), None).await?;
        color_picker_result(&output)
    }

    async fn open(
        &self,
        options: &FileDialogOptions,
        mode: OpenMode,
    ) -> Result<Option<Vec<PathBuf>>> {
        let args = open_args(options, mode, version().await);
        run(TOOL, args, None).await?.paths(TOOL)
    }

    async fn save(&self, options: &FileDialogOptions) -> Result<Option<PathBuf>> {
        let args = save_args(options, version().await);
        let paths = run(TOOL, args, None).await?.paths(TOOL)?;
        Ok(paths.and_then(|paths| paths.into_iter().next()))
    }

    async fn select(&self, options: &SelectOptions, multiple: bool) -> Result<Option<Vec<usize>>> {
        let (args, rows) = select_args(options, multiple);
        run(TOOL, args, Some(rows))
            .await?
            .indices(TOOL, options.items.len())
    }

    async fn date(&self, options: &DateOptions) -> Result<Option<Date>> {
        run(TOOL, date_args(options), None).await?.date(TOOL)
    }

    async fn progress(&self, options: &ProgressOptions) -> Result<Progress> {
        let version = version().await;
        let mut child = spawn(TOOL.as_ref(), progress_args(options), true)?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::Backend(format!("{TOOL} has no standard input")))?;

        let shown = ProgressState::initial(options);
        let (state, requested) = watch::channel(shown.clone());
        let cancelled = CancellationToken::new();
        let task = tokio::spawn(drive_progress(
            child,
            stdin,
            requested,
            shown,
            cancelled.clone(),
            version,
        ));

        Ok(Progress::new(state, cancelled, task))
    }
}

fn no_label(labels: &ButtonLabels) -> &str {
    labels.no.as_deref().unwrap_or("No")
}

fn cancel_label(labels: &ButtonLabels) -> &str {
    labels.cancel.as_deref().unwrap_or("Cancel")
}

/// zenity has no OK/Cancel or Yes/No message box with a warning or error icon, so every message
/// box is an info, warning or error dialog whose OK button is relabelled as needed, with No and
/// Cancel added as extra buttons.
fn message_box_args(options: &MessageBoxOptions) -> Vec<OsString> {
    let kind = match options.icon {
        MessageBoxIcon::Info => "--info",
        MessageBoxIcon::Warning => "--warning",
        MessageBoxIcon::Error => "--error",
    };
    let labels = &options.labels;
    let buttons = options.buttons;

    let mut args = vec![
        kind.into(),
        "--no-markup".into(),
        option("--title", &options.title),
        option("--text", &options.text),
    ];

    let accept_label = if buttons.has_no() {
        Some(labels.yes.as_deref().unwrap_or("Yes"))
    } else {
        labels.ok.as_deref()
    };
    if let Some(label) = accept_label {
        args.push(option("--ok-label", label));
    }
    if buttons.has_no() {
        args.push(option("--extra-button", no_label(labels)));
    }
    if buttons.has_cancel() {
        args.push(option("--extra-button", cancel_label(labels)));
    }

    args
}

fn message_box_result(
    options: &MessageBoxOptions,
    output: &ToolOutput,
) -> Result<MessageBoxResult> {
    let buttons = options.buttons;
    let labels = &options.labels;

    match output.status {
        Some(0) => Ok(buttons.accepted_result()),
        Some(1) if buttons.has_no() && output.stdout == no_label(labels) => {
            Ok(MessageBoxResult::No)
        }
        Some(1) if buttons.has_cancel() && output.stdout == cancel_label(labels) => {
            Ok(MessageBoxResult::Cancel)
        }
        Some(1) => Ok(buttons.dismissed_result()),
        _ => Err(output.unexpected(TOOL)),
    }
}

/// zenity unescapes C escape sequences (`\n`, `\t`, `\\`...) in an entry dialog's prompt, then
/// treats `_` as a keyboard shortcut marker. Doubling both shows the text as is.
fn escape_entry_text(text: &str) -> String {
    text.replace('\\', "\\\\").replace('_', "__")
}

fn escape_markup(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// zenity unescapes C escape sequences in the prompt of list, calendar and progress dialogs,
/// then reads it as Pango markup.
fn escape_markup_text(text: &str) -> String {
    escape_markup(text).replace('\\', "\\\\")
}

/// Returns the arguments and the standard input. Multi-line input is a `--text-info` dialog that
/// reads its initial text from standard input; it cannot show the prompt.
fn text_input_args(options: &TextInputOptions) -> (Vec<OsString>, Option<String>) {
    let title = option("--title", &options.title);

    match options.mode {
        TextInputMode::SingleLine => (
            vec![
                "--entry".into(),
                title,
                option("--text", escape_entry_text(&options.text)),
                option("--entry-text", &options.value),
            ],
            None,
        ),
        TextInputMode::Password => (
            vec![
                "--entry".into(),
                "--hide-text".into(),
                title,
                option("--text", escape_entry_text(&options.text)),
                option("--entry-text", &options.value),
            ],
            None,
        ),
        TextInputMode::MultiLine => (
            vec!["--text-info".into(), "--editable".into(), title],
            Some(options.value.clone()),
        ),
    }
}

fn text_input_result(output: ToolOutput) -> Result<Option<String>> {
    match output.status {
        Some(0) => Ok(Some(output.stdout)),
        Some(1) => Ok(None),
        _ => Err(output.unexpected(TOOL)),
    }
}

fn color_picker_args(options: &ColorPickerOptions) -> Vec<OsString> {
    vec![
        "--color-selection".into(),
        option("--title", &options.title),
        option("--color", format_color(options.value)),
    ]
}

fn color_picker_result(output: &ToolOutput) -> Result<Option<Color>> {
    match output.status {
        Some(0) => parse_color(&output.stdout).map(Some).ok_or_else(|| {
            Error::Backend(format!(
                "{TOOL} returned an invalid colour: {:?}",
                output.stdout
            ))
        }),
        Some(1) => Ok(None),
        _ => Err(output.unexpected(TOOL)),
    }
}

/// A zenity version: major, minor and patch.
type Version = (u32, u32, u32);

/// The installed zenity's version, read once. `None` if it could not be read.
async fn version() -> Option<Version> {
    static VERSION: OnceCell<Option<Version>> = OnceCell::const_new();

    *VERSION
        .get_or_init(async || {
            let output = run(TOOL, vec!["--version".into()], None).await.ok()?;
            parse_version(&output.stdout)
        })
        .await
}

fn parse_version(text: &str) -> Option<Version> {
    let mut parts = text.trim().split('.').map(str::parse);
    let major = parts.next()?.ok()?;
    let minor = parts.next().transpose().ok()?.unwrap_or(0);
    let patch = parts.next().transpose().ok()?.unwrap_or(0);
    Some((major, minor, patch))
}

/// Name of a file that should not exist, used to open a folder with zenity versions from 4.0 to
/// before 4.1.99: given `--filename=DIR/`, these select the folder inside its parent and drop
/// the filters, but given a missing file inside the folder, they open the folder. Other versions
/// open `DIR/` as expected, and would show this name in a save dialog.
const MISSING_FILE: &str = "actiona-missing-file";

const fn opens_folder_through_missing_file(version: Option<Version>) -> bool {
    match version {
        Some((major, minor, patch)) => major == 4 && (minor == 0 || (minor == 1 && patch < 99)),
        None => false,
    }
}

/// The `--filename` value that opens `directory`.
fn folder_argument(directory: &Path, version: Option<Version>) -> OsString {
    if opens_folder_through_missing_file(version) {
        return directory.join(MISSING_FILE).into_os_string();
    }

    let mut folder = directory.as_os_str().to_owned();
    if !folder.as_bytes().ends_with(b"/") {
        folder.push("/");
    }
    folder
}

fn filter_argument(filter: &FileFilter) -> OsString {
    let patterns = filter
        .extensions
        .iter()
        .map(|extension| format!("*.{extension}"))
        .collect::<Vec<_>>()
        .join(" ");
    option("--file-filter", format!("{} | {patterns}", filter.name))
}

fn file_selection_args(
    options: &FileDialogOptions,
    filename: Option<OsString>,
    filters: bool,
) -> Vec<OsString> {
    let mut args = vec!["--file-selection".into(), option("--title", &options.title)];
    if let Some(filename) = filename {
        args.push(option("--filename", filename));
    }
    if filters {
        args.extend(options.filters.iter().map(filter_argument));
    }
    args
}

fn open_args(
    options: &FileDialogOptions,
    mode: OpenMode,
    version: Option<Version>,
) -> Vec<OsString> {
    let filename = options
        .directory
        .as_deref()
        .map(|directory| folder_argument(directory, version));
    let mut args = file_selection_args(options, filename, !mode.directory);
    if mode.directory {
        args.push("--directory".into());
    }
    if mode.multiple {
        args.push("--multiple".into());
        args.push(option("--separator", "\n"));
    }
    args
}

/// zenity versions from 4.0 to before 4.1.99 cannot preset the name of a file that does not exist
/// yet, but still open its folder.
fn save_args(options: &FileDialogOptions, version: Option<Version>) -> Vec<OsString> {
    let filename = match (&options.directory, &options.file_name) {
        (Some(directory), Some(name)) => Some(directory.join(name).into_os_string()),
        (Some(directory), None) => Some(folder_argument(directory, version)),
        (None, Some(name)) => Some(name.into()),
        (None, None) => None,
    };
    let mut args = file_selection_args(options, filename, true);
    args.push("--save".into());
    args
}

/// A radio list or checklist, with a hidden column holding each item's index, which is what
/// zenity prints for the selected items. Returns the arguments and the rows, which are read from
/// standard input, one value per line: as arguments, an item starting with `-` would be dropped
/// even after `--`. Line breaks in items are shown as spaces.
fn select_args(options: &SelectOptions, multiple: bool) -> (Vec<OsString>, String) {
    let args = vec![
        "--list".into(),
        if multiple {
            "--checklist"
        } else {
            "--radiolist"
        }
        .into(),
        option("--title", &options.title),
        option("--text", escape_markup_text(&options.text)),
        "--hide-header".into(),
        option("--column", "Selected"),
        option("--column", "Index"),
        option("--column", "Item"),
        option("--hide-column", "2"),
        option("--print-column", "2"),
        option("--separator", "\n"),
    ];

    let mut rows = String::new();
    for (index, (item, selected)) in options
        .items
        .iter()
        .zip(preselected(options, multiple))
        .enumerate()
    {
        let selected = if selected { "TRUE" } else { "FALSE" };
        let item = item.replace(['\n', '\r'], " ");
        _ = write!(rows, "{selected}\n{index}\n{item}\n");
    }

    (args, rows)
}

fn date_args(options: &DateOptions) -> Vec<OsString> {
    let mut args = vec![
        "--calendar".into(),
        option("--title", &options.title),
        option("--text", escape_markup_text(&options.text)),
        option("--date-format", "%Y-%m-%d"),
    ];
    if let Some(date) = options.value {
        args.push(option("--day", date.day().to_string()));
        args.push(option("--month", date.month().to_string()));
        args.push(option("--year", date.year().to_string()));
    }
    args
}

fn progress_args(options: &ProgressOptions) -> Vec<OsString> {
    let mut args = vec![
        "--progress".into(),
        option("--title", &options.title),
        option("--text", escape_markup_text(&options.text)),
        // Unlike updates, the initial percentage must be a whole number.
        option(
            "--percentage",
            format!(
                "{:.0}",
                options.value.unwrap_or(0.0).clamp(0.0, 1.0) * 100.0
            ),
        ),
    ];
    if options.value.is_none() {
        args.push("--pulsate".into());
    }
    if !options.cancellable {
        args.push("--no-cancel".into());
    }
    args
}

/// zenity versions from 4.0.3 read progress text updates as Pango markup; earlier ones show
/// them as is.
const fn reads_progress_text_as_markup(version: Option<Version>) -> bool {
    match version {
        Some((major, minor, patch)) => major > 4 || (major == 4 && (minor > 0 || patch >= 3)),
        None => true,
    }
}

/// A progress text update fits on one line. zenity unescapes C escape sequences in it, so
/// newlines are sent as `\n`.
fn escape_progress_text(text: &str, version: Option<Version>) -> String {
    let text = if reads_progress_text_as_markup(version) {
        escape_markup(text)
    } else {
        text.to_owned()
    };
    text.replace('\\', "\\\\").replace('\n', "\\n")
}

fn percentage(value: f64) -> String {
    format!("{:.1}", value.clamp(0.0, 1.0) * 100.0)
}

/// The lines of standard input that change what zenity shows from `from` to `to`.
fn progress_updates(from: &ProgressState, to: &ProgressState, version: Option<Version>) -> String {
    let mut lines = String::new();
    match (from.value, to.value) {
        (Some(_), None) => _ = writeln!(lines, "pulsate:true"),
        (None, Some(value)) => _ = writeln!(lines, "pulsate:false\n{}", percentage(value)),
        (Some(previous), Some(value)) if percentage(previous) != percentage(value) => {
            _ = writeln!(lines, "{}", percentage(value));
        }
        _ => {}
    }
    if to.text != from.text {
        _ = writeln!(lines, "#{}", escape_progress_text(&to.text, version));
    }
    lines
}

/// Writes each new state to zenity until the handle is dropped, then closes the dialog. Stops
/// early if the user closes the dialog: with OK once the progress reached 100%, or by
/// cancelling it.
async fn drive_progress(
    mut child: Child,
    mut stdin: ChildStdin,
    mut requested: watch::Receiver<ProgressState>,
    mut shown: ProgressState,
    cancelled: CancellationToken,
    version: Option<Version>,
) {
    loop {
        select! {
            changed = requested.changed() => {
                if changed.is_err() {
                    break;
                }
                let state = requested.borrow_and_update().clone();
                let updates = progress_updates(&shown, &state, version);
                shown = state;
                if let Err(error) = stdin.write_all(updates.as_bytes()).await {
                    debug!("updating the {TOOL} progress dialog failed: {error}");
                }
            }
            status = child.wait() => {
                if !status.is_ok_and(|status| status.success()) {
                    cancelled.cancel();
                }
                return;
            }
        }
    }

    _ = child.kill().await;
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, path::PathBuf};

    use jiff::civil::date;
    use types::Color;

    use super::{
        TOOL, color_picker_args, color_picker_result, date_args, message_box_args,
        message_box_result, open_args, parse_version, progress_args, progress_updates, save_args,
        select_args, text_input_args, text_input_result,
    };
    use crate::{
        ButtonLabels, ColorPickerOptions, DateOptions, FileDialogOptions, FileFilter,
        MessageBoxButtons, MessageBoxIcon, MessageBoxOptions, MessageBoxResult, ProgressOptions,
        SelectOptions, TextInputMode, TextInputOptions, linux::ToolOutput, options::OpenMode,
        progress::ProgressState,
    };

    fn strings(args: &[OsString]) -> Vec<&str> {
        args.iter().map(|arg| arg.to_str().unwrap()).collect()
    }

    fn message_box_options(buttons: MessageBoxButtons) -> MessageBoxOptions {
        MessageBoxOptions {
            title: "Title".to_owned(),
            text: r"-C:\path_name".to_owned(),
            icon: MessageBoxIcon::Warning,
            buttons,
            labels: ButtonLabels::default(),
        }
    }

    #[test]
    fn message_box_ok() {
        let options = message_box_options(MessageBoxButtons::Ok);

        assert_eq!(
            strings(&message_box_args(&options)),
            [
                "--warning",
                "--no-markup",
                "--title=Title",
                r"--text=-C:\path_name"
            ]
        );
        assert_eq!(
            message_box_result(&options, &ToolOutput::new(1, "")).unwrap(),
            MessageBoxResult::Ok
        );
    }

    #[test]
    fn message_box_yes_no_cancel_with_labels() {
        let mut options = message_box_options(MessageBoxButtons::YesNoCancel);
        options.labels = ButtonLabels {
            ok: Some("Ignored".to_owned()),
            cancel: Some("Stop".to_owned()),
            yes: Some("Proceed".to_owned()),
            no: Some("Skip".to_owned()),
        };

        assert_eq!(
            strings(&message_box_args(&options)),
            [
                "--warning",
                "--no-markup",
                "--title=Title",
                r"--text=-C:\path_name",
                "--ok-label=Proceed",
                "--extra-button=Skip",
                "--extra-button=Stop",
            ]
        );

        let result = |status, stdout| {
            message_box_result(&options, &ToolOutput::new(status, stdout)).unwrap()
        };
        assert_eq!(result(0, ""), MessageBoxResult::Yes);
        assert_eq!(result(1, "Skip"), MessageBoxResult::No);
        assert_eq!(result(1, "Stop"), MessageBoxResult::Cancel);
        assert_eq!(result(1, ""), MessageBoxResult::Cancel);
    }

    #[test]
    fn message_box_ok_cancel_defaults() {
        let options = message_box_options(MessageBoxButtons::OkCancel);

        assert_eq!(
            strings(&message_box_args(&options)),
            [
                "--warning",
                "--no-markup",
                "--title=Title",
                r"--text=-C:\path_name",
                "--extra-button=Cancel",
            ]
        );
        assert_eq!(
            message_box_result(&options, &ToolOutput::new(0, "")).unwrap(),
            MessageBoxResult::Ok
        );
        assert_eq!(
            message_box_result(&options, &ToolOutput::new(1, "Cancel")).unwrap(),
            MessageBoxResult::Cancel
        );
    }

    #[test]
    fn message_box_dismissed_yes_no_is_no() {
        let options = message_box_options(MessageBoxButtons::YesNo);

        assert_eq!(
            message_box_result(&options, &ToolOutput::new(1, "")).unwrap(),
            MessageBoxResult::No
        );
    }

    #[test]
    fn message_box_unexpected_status_is_an_error() {
        let options = message_box_options(MessageBoxButtons::Ok);

        assert!(message_box_result(&options, &ToolOutput::new(255, "")).is_err());
    }

    #[test]
    fn text_input_modes() {
        let mut options = TextInputOptions {
            title: "Title".to_owned(),
            text: r"C:\path_name:".to_owned(),
            value: "initial".to_owned(),
            mode: TextInputMode::SingleLine,
        };

        let (args, stdin) = text_input_args(&options);
        assert_eq!(
            strings(&args),
            [
                "--entry",
                "--title=Title",
                r"--text=C:\\path__name:",
                "--entry-text=initial"
            ]
        );
        assert_eq!(stdin, None);

        options.mode = TextInputMode::Password;
        let (args, _) = text_input_args(&options);
        assert_eq!(
            strings(&args),
            [
                "--entry",
                "--hide-text",
                "--title=Title",
                r"--text=C:\\path__name:",
                "--entry-text=initial",
            ]
        );

        options.mode = TextInputMode::MultiLine;
        let (args, stdin) = text_input_args(&options);
        assert_eq!(
            strings(&args),
            ["--text-info", "--editable", "--title=Title"]
        );
        assert_eq!(stdin.as_deref(), Some("initial"));
    }

    #[test]
    fn text_input_results() {
        assert_eq!(
            text_input_result(ToolOutput::new(0, "a\nb")).unwrap(),
            Some("a\nb".to_owned())
        );
        assert_eq!(text_input_result(ToolOutput::new(1, "")).unwrap(), None);
        assert!(text_input_result(ToolOutput::new(-1, "")).is_err());
    }

    #[test]
    fn color_picker() {
        let options = ColorPickerOptions {
            title: "Colour".to_owned(),
            value: Color::new(255, 0, 16, 255),
        };

        assert_eq!(
            strings(&color_picker_args(&options)),
            ["--color-selection", "--title=Colour", "--color=#FF0010"]
        );
        assert_eq!(
            color_picker_result(&ToolOutput::new(0, "rgb(1,2,3)")).unwrap(),
            Some(Color::new(1, 2, 3, 255))
        );
        assert_eq!(color_picker_result(&ToolOutput::new(1, "")).unwrap(), None);
        assert!(color_picker_result(&ToolOutput::new(0, "garbage")).is_err());
    }

    fn file_options() -> FileDialogOptions {
        FileDialogOptions {
            title: "Files".to_owned(),
            directory: Some(PathBuf::from("/tmp/dir")),
            file_name: Some("out.png".to_owned()),
            filters: vec![FileFilter {
                name: "Images".to_owned(),
                extensions: vec!["png".to_owned(), "jpg".to_owned()],
            }],
        }
    }

    #[test]
    fn open_files() {
        let mode = OpenMode {
            multiple: true,
            directory: false,
        };

        assert_eq!(
            strings(&open_args(&file_options(), mode, Some((4, 2, 0)))),
            [
                "--file-selection",
                "--title=Files",
                "--filename=/tmp/dir/",
                "--file-filter=Images | *.png *.jpg",
                "--multiple",
                "--separator=\n",
            ]
        );
    }

    #[test]
    fn open_folder_ignores_filters() {
        let mode = OpenMode {
            multiple: false,
            directory: true,
        };
        let mut options = file_options();
        options.directory = Some(PathBuf::from("/tmp/dir/"));

        assert_eq!(
            strings(&open_args(&options, mode, None)),
            [
                "--file-selection",
                "--title=Files",
                "--filename=/tmp/dir/",
                "--directory",
            ]
        );
    }

    #[test]
    fn save_file() {
        let mut options = file_options();

        assert_eq!(
            strings(&save_args(&options, Some((4, 2, 0))))[2],
            "--filename=/tmp/dir/out.png"
        );

        options.directory = None;
        assert_eq!(
            strings(&save_args(&options, Some((4, 2, 0))))[2],
            "--filename=out.png"
        );

        options.file_name = None;
        assert_eq!(
            strings(&save_args(&options, Some((4, 2, 0)))),
            [
                "--file-selection",
                "--title=Files",
                "--file-filter=Images | *.png *.jpg",
                "--save",
            ]
        );
    }

    #[test]
    fn paths_result() {
        assert_eq!(
            ToolOutput::new(0, "/a b\n/c").paths(TOOL).unwrap(),
            Some(vec![PathBuf::from("/a b"), PathBuf::from("/c")])
        );
        assert_eq!(ToolOutput::new(1, "").paths(TOOL).unwrap(), None);
        assert!(ToolOutput::new(5, "").paths(TOOL).is_err());
    }

    #[test]
    fn parses_versions() {
        assert_eq!(parse_version("4.0.1\n"), Some((4, 0, 1)));
        assert_eq!(parse_version("3.44"), Some((3, 44, 0)));
        assert_eq!(parse_version("4.4.beta"), None);
        assert_eq!(parse_version("zenity"), None);
    }

    #[test]
    fn opens_folders_through_a_missing_file_on_zenity_4_0() {
        let mode = OpenMode {
            multiple: false,
            directory: false,
        };
        let filename = |version| strings(&open_args(&file_options(), mode, version))[2].to_owned();

        assert_eq!(filename(Some((3, 44, 5))), "--filename=/tmp/dir/");
        assert_eq!(
            filename(Some((4, 0, 1))),
            "--filename=/tmp/dir/actiona-missing-file"
        );
        assert_eq!(
            filename(Some((4, 1, 90))),
            "--filename=/tmp/dir/actiona-missing-file"
        );
        assert_eq!(filename(Some((4, 1, 99))), "--filename=/tmp/dir/");
        assert_eq!(filename(Some((4, 2, 2))), "--filename=/tmp/dir/");
        assert_eq!(filename(None), "--filename=/tmp/dir/");

        let mut options = file_options();
        options.file_name = None;
        assert_eq!(
            strings(&save_args(&options, Some((4, 0, 1))))[2],
            "--filename=/tmp/dir/actiona-missing-file"
        );
    }

    #[test]
    fn select_lists() {
        let options = SelectOptions {
            title: "Pick".to_owned(),
            text: r"<b> & C:\x".to_owned(),
            items: vec!["-first".to_owned(), "two\nlines".to_owned()],
            selected: vec![1],
        };

        let (args, rows) = select_args(&options, false);
        assert_eq!(
            strings(&args),
            [
                "--list",
                "--radiolist",
                "--title=Pick",
                r"--text=&lt;b&gt; &amp; C:\\x",
                "--hide-header",
                "--column=Selected",
                "--column=Index",
                "--column=Item",
                "--hide-column=2",
                "--print-column=2",
                "--separator=\n",
            ]
        );
        assert_eq!(rows, "FALSE\n0\n-first\nTRUE\n1\ntwo lines\n");

        let (args, _) = select_args(&options, true);
        assert_eq!(strings(&args)[1], "--checklist");
    }

    #[test]
    fn date_dialog() {
        let mut options = DateOptions {
            title: "When".to_owned(),
            text: "Day:".to_owned(),
            value: None,
        };
        let base = [
            "--calendar",
            "--title=When",
            "--text=Day:",
            "--date-format=%Y-%m-%d",
        ];

        assert_eq!(strings(&date_args(&options)), base);

        options.value = Some(date(2026, 2, 3));
        let args = date_args(&options);
        let args = strings(&args);
        assert_eq!(args[..4], base);
        assert_eq!(args[4..], ["--day=3", "--month=2", "--year=2026"]);
    }

    #[test]
    fn progress_dialog() {
        let options = ProgressOptions {
            title: "Work".to_owned(),
            text: "a & b".to_owned(),
            cancellable: false,
            value: None,
        };

        assert_eq!(
            strings(&progress_args(&options)),
            [
                "--progress",
                "--title=Work",
                "--text=a &amp; b",
                "--percentage=0",
                "--pulsate",
                "--no-cancel",
            ]
        );

        let options = ProgressOptions {
            value: Some(0.456),
            cancellable: true,
            ..options
        };
        assert_eq!(strings(&progress_args(&options))[3], "--percentage=46");
    }

    #[test]
    fn progress_update_lines() {
        let state = |value, text: &str| ProgressState {
            value,
            text: text.to_owned(),
        };
        let current = Some((4, 2, 0));
        let updates = |from, to| progress_updates(&from, &to, current);

        assert_eq!(updates(state(Some(0.0), "a"), state(Some(0.0), "a")), "");
        assert_eq!(
            updates(state(Some(0.0), "a"), state(Some(0.255), "a")),
            "25.5\n"
        );
        assert_eq!(
            updates(state(Some(0.5), "a"), state(None, "a")),
            "pulsate:true\n"
        );
        assert_eq!(
            updates(state(None, "a"), state(Some(0.5), "a")),
            "pulsate:false\n50.0\n"
        );
        assert_eq!(updates(state(None, "a"), state(None, "a")), "");
        assert_eq!(
            updates(state(None, "a"), state(None, "x < y\n\\z")),
            "#x &lt; y\\n\\\\z\n"
        );
        assert_eq!(
            progress_updates(&state(None, "a"), &state(None, "x < y"), Some((4, 0, 1))),
            "#x < y\n"
        );
    }
}
