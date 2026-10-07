//! Dialogs shown by `kdialog`.
//!
//! Message boxes exit with status 0 for OK, Yes and Continue, 1 for No and 2 for Cancel or when
//! closed. Other dialogs exit with status 0 when accepted and 1 when cancelled.

use std::{
    ffi::OsString,
    path::{self, Path, PathBuf},
};

use futures_util::StreamExt;
use jiff::civil::Date;
use tokio::{process::Child, select, sync::watch};
use tokio_util::sync::CancellationToken;
use tracing::debug;
use types::Color;
use zbus::{
    Connection, Proxy,
    fdo::DBusProxy,
    names::BusName,
    proxy::{Builder, CacheProperties},
};

use super::{
    LinuxTool, Tool, ToolOutput, format_color, option, parse_color, preselected, run, spawn,
};
use crate::{
    ColorPickerOptions, DateOptions, Error, FileDialogOptions, FileFilter, MessageBoxButtons,
    MessageBoxIcon, MessageBoxOptions, MessageBoxResult, Progress, ProgressOptions, Result,
    SelectOptions, TextInputMode, TextInputOptions, options::OpenMode, progress::ProgressState,
};

/// Shows dialogs with `kdialog`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct KDialog;

const TOOL: LinuxTool = LinuxTool::KDialog(KDialog);

impl Tool for KDialog {
    async fn message_box(&self, options: &MessageBoxOptions) -> Result<MessageBoxResult> {
        let output = run(TOOL, message_box_args(options), None).await?;
        message_box_result(options.buttons, &output)
    }

    async fn text_input(&self, options: &TextInputOptions) -> Result<Option<String>> {
        let output = run(TOOL, text_input_args(options), None).await?;
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
        let output = run(TOOL, open_args(options, mode)?, None).await?;
        if mode.multiple {
            output.paths(TOOL)
        } else {
            Ok(output.path(TOOL)?.map(|path| vec![path]))
        }
    }

    async fn save(&self, options: &FileDialogOptions) -> Result<Option<PathBuf>> {
        run(TOOL, save_args(options)?, None).await?.path(TOOL)
    }

    async fn select(&self, options: &SelectOptions, multiple: bool) -> Result<Option<Vec<usize>>> {
        run(TOOL, select_args(options, multiple), None)
            .await?
            .indices(TOOL, options.items.len())
    }

    async fn date(&self, options: &DateOptions) -> Result<Option<Date>> {
        run(TOOL, date_args(options), None).await?.date(TOOL)
    }

    /// `kdialog --progressbar` starts `kdialog_progress_helper` detached, prints its D-Bus
    /// address and exits, so the helper is started directly instead: as a child process, it
    /// is killed if the dialog is dropped without being closed over D-Bus.
    async fn progress(&self, options: &ProgressOptions) -> Result<Progress> {
        let mut child = spawn(PROGRESS_HELPER, progress_args(options), false)?;
        let pid = child.id().ok_or_else(|| {
            Error::Backend(format!(
                "{PROGRESS_HELPER} exited before showing the dialog"
            ))
        })?;
        let service = format!("org.kde.kdialog-{pid}");

        let connection = Connection::session().await?;
        wait_for_service(&connection, &service, &mut child).await?;
        let dialog: Proxy<'static> = Builder::new(&connection)
            .destination(service)?
            .path("/ProgressDialog")?
            .interface("org.kde.kdialog.ProgressDialog")?
            .cache_properties(CacheProperties::No)
            .build()
            .await?;

        let shown = ProgressState::initial(options);
        if !options.cancellable {
            dialog.call_method("showCancelButton", &(false,)).await?;
        }
        show_progress(&dialog, None, &shown).await?;

        let (state, requested) = watch::channel(shown.clone());
        let cancelled = CancellationToken::new();
        let task = tokio::spawn(drive_progress(
            child,
            dialog,
            requested,
            shown,
            cancelled.clone(),
        ));

        Ok(Progress::new(state, cancelled, task))
    }
}

/// kdialog turns `\n` into a newline and `\\` into a backslash in most texts, so backslashes
/// must be doubled to be shown as is.
fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
}

/// Qt labels show text as HTML when it looks like it starts with a tag, or contains `&lt;`
/// (`Qt::mightBeRichText`). A zero-width space after each `<` and inside each `&lt;` keeps it
/// plain text without changing how it looks.
fn plain_label(text: &str) -> String {
    text.replace('<', "<\u{200B}")
        .replace("&lt;", "&\u{200B}lt;")
}

/// A prompt as kdialog shows it: escaped, and kept from being read as HTML.
fn prompt(text: &str) -> String {
    plain_label(&escape(text))
}

/// kdialog has no OK/Cancel message box, so it is a Yes/No one with relabelled buttons. Its
/// question boxes have no error icon, so the warning one is used instead.
fn message_box_args(options: &MessageBoxOptions) -> Vec<OsString> {
    let warning = !options.icon.is_info();
    let kind = match (options.buttons, options.icon) {
        (MessageBoxButtons::Ok, MessageBoxIcon::Info) => "--msgbox",
        (MessageBoxButtons::Ok, MessageBoxIcon::Warning) => "--sorry",
        (MessageBoxButtons::Ok, MessageBoxIcon::Error) => "--error",
        (MessageBoxButtons::OkCancel | MessageBoxButtons::YesNo, _) if warning => "--warningyesno",
        (MessageBoxButtons::OkCancel | MessageBoxButtons::YesNo, _) => "--yesno",
        (MessageBoxButtons::YesNoCancel, _) if warning => "--warningyesnocancel",
        (MessageBoxButtons::YesNoCancel, _) => "--yesnocancel",
    };
    let labels = &options.labels;

    let mut args = vec![
        option("--title", &options.title),
        option(kind, prompt(&options.text)),
    ];

    let mut label = |name: &str, value: Option<&str>| {
        if let Some(value) = value {
            args.push(option(name, value));
        }
    };
    match options.buttons {
        MessageBoxButtons::Ok => label("--ok-label", labels.ok.as_deref()),
        MessageBoxButtons::OkCancel => {
            label("--yes-label", Some(labels.ok.as_deref().unwrap_or("OK")));
            label(
                "--no-label",
                Some(labels.cancel.as_deref().unwrap_or("Cancel")),
            );
        }
        MessageBoxButtons::YesNo => {
            label("--yes-label", labels.yes.as_deref());
            label("--no-label", labels.no.as_deref());
        }
        MessageBoxButtons::YesNoCancel => {
            label("--yes-label", labels.yes.as_deref());
            label("--no-label", labels.no.as_deref());
            label("--cancel-label", labels.cancel.as_deref());
        }
    }

    args
}

fn message_box_result(buttons: MessageBoxButtons, output: &ToolOutput) -> Result<MessageBoxResult> {
    match output.status {
        Some(0) => Ok(buttons.accepted_result()),
        // In an OK/Cancel box, No is the relabelled Cancel button.
        Some(1) if buttons.has_no() => Ok(MessageBoxResult::No),
        Some(1 | 2) => Ok(buttons.dismissed_result()),
        _ => Err(output.unexpected(TOOL)),
    }
}

/// The initial value is passed after `--` so it is never taken for an option. It is escaped in
/// multi-line mode only, because kdialog only unescapes it there. The single-line prompt's label
/// treats `&` as a keyboard shortcut marker, so it is doubled there.
fn text_input_args(options: &TextInputOptions) -> Vec<OsString> {
    let title = option("--title", &options.title);
    let text = escape(&options.text);

    match options.mode {
        TextInputMode::SingleLine => vec![
            title,
            option("--inputbox", plain_label(&text.replace('&', "&&"))),
            "--".into(),
            options.value.as_str().into(),
        ],
        TextInputMode::MultiLine => vec![
            title,
            option("--textinputbox", plain_label(&text)),
            "--".into(),
            escape(&options.value).into(),
        ],
        TextInputMode::Password => vec![title, option("--password", plain_label(&text))],
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
        option("--title", &options.title),
        "--getcolor".into(),
        option("--default", format_color(options.value)),
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

/// The start location: a folder, or a file whose folder is opened and whose name is preselected.
/// kdialog reads it as a URL, so it must be absolute or it could be taken for a web address. An
/// empty location opens the current folder.
fn start_location(directory: Option<&Path>, file_name: Option<&str>) -> Result<OsString> {
    let location = match (directory, file_name) {
        (None, None) => return Ok(OsString::new()),
        (Some(directory), None) => path::absolute(directory)?,
        (directory, Some(name)) => path::absolute(directory.unwrap_or(Path::new(".")))?.join(name),
    };
    Ok(location.into_os_string())
}

/// Qt name filters such as `Images (*.png *.jpg)`, separated by `|`.
fn filter_argument(filters: &[FileFilter]) -> String {
    let filters = filters
        .iter()
        .map(|filter| {
            let patterns = filter
                .extensions
                .iter()
                .map(|extension| format!("*.{extension}"))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{} ({patterns})", filter.name)
        })
        .collect::<Vec<_>>()
        .join("|");
    escape(&filters)
}

fn open_args(options: &FileDialogOptions, mode: OpenMode) -> Result<Vec<OsString>> {
    let start = start_location(options.directory.as_deref(), None)?;
    let title = option("--title", &options.title);

    if mode.directory {
        if mode.multiple {
            return Err(Error::Unsupported {
                dialog: "selecting several folders",
                backend: "kdialog",
            });
        }
        return Ok(vec![
            title,
            "--getexistingdirectory".into(),
            "--".into(),
            start,
        ]);
    }

    let mut args = vec![title, "--getopenfilename".into()];
    if mode.multiple {
        args.push("--multiple".into());
        args.push("--separate-output".into());
    }
    args.extend(["--".into(), start, filter_argument(&options.filters).into()]);
    Ok(args)
}

fn save_args(options: &FileDialogOptions) -> Result<Vec<OsString>> {
    Ok(vec![
        option("--title", &options.title),
        "--getsavefilename".into(),
        "--".into(),
        start_location(options.directory.as_deref(), options.file_name.as_deref())?,
        filter_argument(&options.filters).into(),
    ])
}

/// A radio list or checklist whose tags are the items' indices, which is what kdialog prints
/// for the selected items.
fn select_args(options: &SelectOptions, multiple: bool) -> Vec<OsString> {
    let kind = if multiple {
        "--checklist"
    } else {
        "--radiolist"
    };
    let mut args = vec![
        option("--title", &options.title),
        option(kind, prompt(&options.text)),
    ];
    if multiple {
        args.push("--separate-output".into());
    }
    args.push("--".into());

    for (index, (item, selected)) in options
        .items
        .iter()
        .zip(preselected(options, multiple))
        .enumerate()
    {
        args.push(index.to_string().into());
        args.push(item.into());
        args.push(if selected { "on" } else { "off" }.into());
    }

    args
}

fn date_args(options: &DateOptions) -> Vec<OsString> {
    let mut args = vec![
        option("--title", &options.title),
        option("--calendar", prompt(&options.text)),
        option("--dateformat", "yyyy-MM-dd"),
    ];
    if let Some(date) = options.value {
        args.push(option("--default", date.to_string()));
    }
    args
}

const PROGRESS_HELPER: &str = "kdialog_progress_helper";

/// The progress bar's maximum: values are sent in thousandths.
const PROGRESS_STEPS: i32 = 1000;

fn progress_args(options: &ProgressOptions) -> Vec<OsString> {
    vec![
        option("--title", &options.title),
        option("--progressbar", prompt(&options.text)),
        "--".into(),
        PROGRESS_STEPS.to_string().into(),
    ]
}

#[allow(clippy::cast_possible_truncation)]
fn progress_steps(value: f64) -> i32 {
    (value.clamp(0.0, 1.0) * f64::from(PROGRESS_STEPS)).round() as i32
}

/// Waits until the helper's D-Bus service is up, which is when its dialog can be controlled.
async fn wait_for_service(connection: &Connection, service: &str, child: &mut Child) -> Result<()> {
    let bus = DBusProxy::new(connection).await?;
    let mut registrations = bus
        .receive_name_owner_changed_with_args(&[(0, service)])
        .await?;
    let name = BusName::try_from(service).map_err(zbus::Error::from)?;
    if bus.name_has_owner(name).await.map_err(zbus::Error::from)? {
        return Ok(());
    }

    select! {
        _ = registrations.next() => Ok(()),
        _ = child.wait() => Err(Error::Backend(format!(
            "{PROGRESS_HELPER} exited before showing the dialog"
        ))),
    }
}

/// Shows `to` in the dialog, which currently shows `from`, or has just started if `from` is
/// `None`. A busy indicator is a maximum of 0.
async fn show_progress(
    dialog: &Proxy<'static>,
    from: Option<&ProgressState>,
    to: &ProgressState,
) -> zbus::Result<()> {
    let shown = from.map(|from| from.value.map(progress_steps));
    let steps = to.value.map(progress_steps);

    if shown.is_none_or(|shown| shown.is_some() != steps.is_some()) {
        let maximum = if steps.is_some() { PROGRESS_STEPS } else { 0 };
        dialog.set_property("maximum", maximum).await?;
    }
    if let Some(steps) = steps
        && shown.is_none_or(|shown| shown != Some(steps))
    {
        dialog.set_property("value", steps).await?;
    }
    if from.is_some_and(|from| from.text != to.text) {
        dialog
            .call_method("setLabelText", &(plain_label(&to.text),))
            .await?;
    }
    Ok(())
}

/// Shows each new state until the handle is dropped, then closes the dialog. Stops early if the
/// user cancels the dialog, which makes the helper exit.
async fn drive_progress(
    mut child: Child,
    dialog: Proxy<'static>,
    mut requested: watch::Receiver<ProgressState>,
    mut shown: ProgressState,
    cancelled: CancellationToken,
) {
    loop {
        select! {
            changed = requested.changed() => {
                if changed.is_err() {
                    break;
                }
                let state = requested.borrow_and_update().clone();
                if let Err(error) = show_progress(&dialog, Some(&shown), &state).await {
                    debug!("updating the {TOOL} progress dialog failed: {error}");
                }
                shown = state;
            }
            _ = child.wait() => {
                cancelled.cancel();
                return;
            }
        }
    }

    if let Err(error) = dialog.call_method("close", &()).await {
        debug!("closing the {TOOL} progress dialog failed: {error}");
    }
    _ = child.kill().await;
}

#[cfg(test)]
mod tests {
    use std::{env, ffi::OsString, path::PathBuf};

    use jiff::civil::date;
    use types::Color;

    use super::{
        color_picker_args, color_picker_result, date_args, message_box_args, message_box_result,
        open_args, progress_args, progress_steps, save_args, select_args, text_input_args,
        text_input_result,
    };
    use crate::{
        ButtonLabels, ColorPickerOptions, DateOptions, Error, FileDialogOptions, FileFilter,
        MessageBoxButtons, MessageBoxIcon, MessageBoxOptions, MessageBoxResult, ProgressOptions,
        SelectOptions, TextInputMode, TextInputOptions, linux::ToolOutput, options::OpenMode,
    };

    fn strings(args: &[OsString]) -> Vec<&str> {
        args.iter().map(|arg| arg.to_str().unwrap()).collect()
    }

    fn message_box_options(icon: MessageBoxIcon, buttons: MessageBoxButtons) -> MessageBoxOptions {
        MessageBoxOptions {
            title: "Title".to_owned(),
            text: r"C:\path".to_owned(),
            icon,
            buttons,
            labels: ButtonLabels::default(),
        }
    }

    #[test]
    fn message_box_kinds() {
        let kind = |icon, buttons| {
            message_box_args(&message_box_options(icon, buttons))[1]
                .to_str()
                .unwrap()
                .split_once('=')
                .unwrap()
                .0
                .to_owned()
        };

        assert_eq!(
            kind(MessageBoxIcon::Info, MessageBoxButtons::Ok),
            "--msgbox"
        );
        assert_eq!(
            kind(MessageBoxIcon::Warning, MessageBoxButtons::Ok),
            "--sorry"
        );
        assert_eq!(
            kind(MessageBoxIcon::Error, MessageBoxButtons::Ok),
            "--error"
        );
        assert_eq!(
            kind(MessageBoxIcon::Info, MessageBoxButtons::OkCancel),
            "--yesno"
        );
        assert_eq!(
            kind(MessageBoxIcon::Error, MessageBoxButtons::YesNo),
            "--warningyesno"
        );
        assert_eq!(
            kind(MessageBoxIcon::Info, MessageBoxButtons::YesNoCancel),
            "--yesnocancel"
        );
        assert_eq!(
            kind(MessageBoxIcon::Warning, MessageBoxButtons::YesNoCancel),
            "--warningyesnocancel"
        );
    }

    #[test]
    fn message_box_escapes_text_and_relabels_ok_cancel() {
        let mut options = message_box_options(MessageBoxIcon::Info, MessageBoxButtons::OkCancel);
        options.labels.cancel = Some("Discard".to_owned());

        assert_eq!(
            strings(&message_box_args(&options)),
            [
                "--title=Title",
                r"--yesno=C:\\path",
                "--yes-label=OK",
                "--no-label=Discard",
            ]
        );
    }

    #[test]
    fn message_box_results() {
        let result =
            |buttons, status| message_box_result(buttons, &ToolOutput::new(status, "")).unwrap();

        assert_eq!(result(MessageBoxButtons::Ok, 2), MessageBoxResult::Ok);
        assert_eq!(result(MessageBoxButtons::OkCancel, 0), MessageBoxResult::Ok);
        assert_eq!(
            result(MessageBoxButtons::OkCancel, 1),
            MessageBoxResult::Cancel
        );
        assert_eq!(
            result(MessageBoxButtons::OkCancel, 2),
            MessageBoxResult::Cancel
        );
        assert_eq!(result(MessageBoxButtons::YesNo, 1), MessageBoxResult::No);
        assert_eq!(result(MessageBoxButtons::YesNo, 2), MessageBoxResult::No);
        assert_eq!(
            result(MessageBoxButtons::YesNoCancel, 0),
            MessageBoxResult::Yes
        );
        assert_eq!(
            result(MessageBoxButtons::YesNoCancel, 1),
            MessageBoxResult::No
        );
        assert_eq!(
            result(MessageBoxButtons::YesNoCancel, 2),
            MessageBoxResult::Cancel
        );
        assert!(message_box_result(MessageBoxButtons::Ok, &ToolOutput::new(255, "")).is_err());
    }

    #[test]
    fn text_input_modes() {
        let mut options = TextInputOptions {
            title: "Title".to_owned(),
            text: r"a\b & c".to_owned(),
            value: r"-c\d".to_owned(),
            mode: TextInputMode::SingleLine,
        };

        assert_eq!(
            strings(&text_input_args(&options)),
            ["--title=Title", r"--inputbox=a\\b && c", "--", r"-c\d"]
        );

        options.mode = TextInputMode::MultiLine;
        assert_eq!(
            strings(&text_input_args(&options)),
            ["--title=Title", r"--textinputbox=a\\b & c", "--", r"-c\\d"]
        );

        options.mode = TextInputMode::Password;
        assert_eq!(
            strings(&text_input_args(&options)),
            ["--title=Title", r"--password=a\\b & c"]
        );
    }

    #[test]
    fn text_input_results() {
        assert_eq!(
            text_input_result(ToolOutput::new(0, "text")).unwrap(),
            Some("text".to_owned())
        );
        assert_eq!(text_input_result(ToolOutput::new(1, "")).unwrap(), None);
        assert!(text_input_result(ToolOutput::new(255, "")).is_err());
    }

    #[test]
    fn color_picker() {
        let options = ColorPickerOptions {
            title: "Colour".to_owned(),
            value: Color::new(255, 0, 16, 255),
        };

        assert_eq!(
            strings(&color_picker_args(&options)),
            ["--title=Colour", "--getcolor", "--default=#FF0010"]
        );
        assert_eq!(
            color_picker_result(&ToolOutput::new(0, "#0a0b0c")).unwrap(),
            Some(Color::new(10, 11, 12, 255))
        );
        assert_eq!(color_picker_result(&ToolOutput::new(1, "")).unwrap(), None);
    }

    fn file_options() -> FileDialogOptions {
        FileDialogOptions {
            title: "Files".to_owned(),
            directory: Some(PathBuf::from("/tmp/dir")),
            file_name: Some("out.png".to_owned()),
            filters: vec![
                FileFilter {
                    name: "Images".to_owned(),
                    extensions: vec!["png".to_owned(), "jpg".to_owned()],
                },
                FileFilter {
                    name: r"Text\docs".to_owned(),
                    extensions: vec!["txt".to_owned()],
                },
            ],
        }
    }

    #[test]
    fn open_files() {
        let mode = OpenMode {
            multiple: true,
            directory: false,
        };

        assert_eq!(
            strings(&open_args(&file_options(), mode).unwrap()),
            [
                "--title=Files",
                "--getopenfilename",
                "--multiple",
                "--separate-output",
                "--",
                "/tmp/dir",
                r"Images (*.png *.jpg)|Text\\docs (*.txt)",
            ]
        );
    }

    #[test]
    fn open_folders() {
        let mut mode = OpenMode {
            multiple: false,
            directory: true,
        };

        assert_eq!(
            strings(&open_args(&file_options(), mode).unwrap()),
            ["--title=Files", "--getexistingdirectory", "--", "/tmp/dir"]
        );

        mode.multiple = true;
        assert!(matches!(
            open_args(&file_options(), mode),
            Err(Error::Unsupported { .. })
        ));
    }

    #[test]
    fn save_file() {
        let mut options = file_options();
        options.filters.clear();

        assert_eq!(
            strings(&save_args(&options).unwrap()),
            [
                "--title=Files",
                "--getsavefilename",
                "--",
                "/tmp/dir/out.png",
                ""
            ]
        );

        options.directory = None;
        let current = env::current_dir().unwrap().join("out.png");
        assert_eq!(
            strings(&save_args(&options).unwrap())[3],
            current.to_str().unwrap()
        );

        options.file_name = None;
        assert_eq!(strings(&save_args(&options).unwrap())[3], "");
    }

    #[test]
    fn select_lists() {
        let options = SelectOptions {
            title: "Pick".to_owned(),
            text: r"C:\x".to_owned(),
            items: vec!["-first".to_owned(), "second".to_owned()],
            selected: vec![],
        };

        assert_eq!(
            strings(&select_args(&options, false)),
            [
                "--title=Pick",
                r"--radiolist=C:\\x",
                "--",
                "0",
                "-first",
                "on",
                "1",
                "second",
                "off",
            ]
        );
        assert_eq!(
            strings(&select_args(&options, true))[1..4],
            [r"--checklist=C:\\x", "--separate-output", "--"]
        );
    }

    #[test]
    fn date_dialog() {
        let options = DateOptions {
            title: "When".to_owned(),
            text: "Day:".to_owned(),
            value: Some(date(2026, 2, 3)),
        };

        assert_eq!(
            strings(&date_args(&options)),
            [
                "--title=When",
                "--calendar=Day:",
                "--dateformat=yyyy-MM-dd",
                "--default=2026-02-03",
            ]
        );
    }

    #[test]
    fn progress_dialog() {
        let options = ProgressOptions {
            title: "Work".to_owned(),
            text: r"C:\x".to_owned(),
            ..ProgressOptions::default()
        };

        assert_eq!(
            strings(&progress_args(&options)),
            ["--title=Work", r"--progressbar=C:\\x", "--", "1000"]
        );

        assert_eq!(progress_steps(0.4567), 457);
        assert_eq!(progress_steps(1.5), 1000);
    }

    #[test]
    fn labels_are_kept_plain() {
        let options = TextInputOptions {
            title: "Title".to_owned(),
            text: "<b>x</b> &lt; y & z".to_owned(),
            value: String::new(),
            mode: TextInputMode::SingleLine,
        };

        assert_eq!(
            strings(&text_input_args(&options))[1],
            "--inputbox=<\u{200B}b>x<\u{200B}/b> &&\u{200B}lt; y && z"
        );
    }
}
