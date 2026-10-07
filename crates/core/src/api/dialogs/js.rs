#![allow(clippy::needless_pass_by_value)]

use std::{path::PathBuf, sync::Arc};

use dialogs::{
    ButtonLabels, ColorPickerOptions, DateOptions, Dialogs, FileDialogOptions, FileFilter,
    MessageBoxOptions, Progress, ProgressOptions, SelectOptions, TextInputOptions,
};
use itertools::Itertools;
use macros::{FromJsObject, js_class, js_methods, options};
use parking_lot::Mutex;
use rquickjs::{
    Ctx, JsLifetime, Promise, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::Opt,
};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{
    IntoJsResult,
    api::{
        color::js::{JsColor, JsColorLike},
        dialogs::show,
        js::{
            abort_controller::JsAbortSignal,
            classes::{HostClass, SingletonClass, register_enum, register_host_class},
            date::JsDate,
            duration::JsDuration,
            task::task_with_token,
        },
    },
    cancel_on,
    runtime::WithUserData,
};

pub type JsMessageBoxIcon = super::MessageBoxIcon;
pub type JsMessageBoxButtons = super::MessageBoxButtons;
pub type JsMessageBoxResult = super::MessageBoxResult;
pub type JsTextInputMode = super::TextInputMode;

/// Labels replacing the default ones of the message box buttons. A label is only used if its
/// button is shown.
///
/// ```ts
/// await dialogs.messageBox("Save changes?", {
///   buttons: MessageBoxButtons.YesNoCancel,
///   labels: { yes: "Save", no: "Discard" },
/// });
/// ```
/// @category Dialogs
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsMessageBoxLabels {
    /// Label of the OK button.
    pub ok: Option<String>,

    /// Label of the Cancel button.
    pub cancel: Option<String>,

    /// Label of the Yes button.
    pub yes: Option<String>,

    /// Label of the No button.
    pub no: Option<String>,
}

impl From<JsMessageBoxLabels> for ButtonLabels {
    fn from(labels: JsMessageBoxLabels) -> Self {
        Self {
            ok: labels.ok,
            cancel: labels.cancel,
            yes: labels.yes,
            no: labels.no,
        }
    }
}

/// Message box options.
///
/// ```ts
/// await dialogs.messageBox("Delete this file?", {
///   title: "Confirm",
///   buttons: MessageBoxButtons.YesNo,
///   icon: MessageBoxIcon.Warning,
/// });
/// ```
/// @category Dialogs
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsMessageBoxOptions {
    /// Title displayed in the message box title bar.
    pub title: Option<String>,

    /// Buttons displayed in the message box.
    #[default(ts = "MessageBoxButtons.Ok")]
    pub buttons: Option<JsMessageBoxButtons>,

    /// Labels replacing the default ones of the buttons.
    pub labels: Option<JsMessageBoxLabels>,

    /// Icon displayed in the message box.
    #[default(ts = "MessageBoxIcon.Info")]
    pub icon: Option<JsMessageBoxIcon>,

    /// Closes the message box after this duration, which then returns `MessageBoxResult.Timeout`.
    pub timeout: Option<JsDuration>,

    /// Abort signal to close the message box.
    pub signal: Option<JsAbortSignal>,
}

/// A file type filter for file dialogs.
///
/// ```ts
/// const filter = { name: "Images", extensions: ["png", "jpg"] };
/// ```
/// @category Dialogs
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsFileFilter {
    /// Display name of the filter.
    pub name: String,

    /// File extensions matched by this filter (without leading dot).
    pub extensions: Vec<String>,
}

impl From<JsFileFilter> for FileFilter {
    fn from(filter: JsFileFilter) -> Self {
        Self {
            name: filter.name,
            extensions: filter.extensions,
        }
    }
}

/// File dialog options.
///
/// ```ts
/// const path = await dialogs.pickFile({
///   title: "Open Image",
///   filters: [{ name: "Images", extensions: ["png", "jpg"] }],
/// });
/// ```
/// @category Dialogs
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsFileDialogOptions {
    /// Title displayed in the dialog title bar.
    pub title: Option<String>,

    /// Initial directory shown in the dialog.
    pub directory: Option<String>,

    /// Initial file name. Only used by `saveFile`.
    pub file_name: Option<String>,

    /// File type filters shown in the dialog. Ignored when picking folders.
    pub filters: Option<Vec<JsFileFilter>>,

    /// Closes the dialog after this duration, as if the user had cancelled it.
    pub timeout: Option<JsDuration>,

    /// Abort signal to close the dialog.
    pub signal: Option<JsAbortSignal>,
}

impl From<JsFileDialogOptions> for FileDialogOptions {
    fn from(options: JsFileDialogOptions) -> Self {
        Self {
            title: options.title.unwrap_or_default(),
            directory: options.directory.map(PathBuf::from),
            file_name: options.file_name,
            filters: options
                .filters
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

/// Text input dialog options.
///
/// ```ts
/// const name = await dialogs.textInput("Enter your name:", {
///   title: "Name",
///   mode: TextInputMode.SingleLine,
/// });
/// ```
/// @category Dialogs
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsTextInputOptions {
    /// Title displayed in the dialog title bar.
    pub title: Option<String>,

    /// Initial value shown in the text field.
    pub value: Option<String>,

    /// Input mode controlling the dialog style.
    #[default(ts = "TextInputMode.SingleLine")]
    pub mode: Option<JsTextInputMode>,

    /// Closes the dialog after this duration, as if the user had cancelled it.
    pub timeout: Option<JsDuration>,

    /// Abort signal to close the dialog.
    pub signal: Option<JsAbortSignal>,
}

/// Color picker dialog options.
///
/// ```ts
/// const color = await dialogs.colorPicker({
///   title: "Choose a color",
///   value: new Color(255, 0, 0),
/// });
/// ```
/// @category Dialogs
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsColorPickerOptions {
    /// Title displayed in the dialog title bar.
    pub title: Option<String>,

    /// Initial color shown in the picker. Its alpha channel is ignored.
    pub value: Option<JsColorLike>,

    /// Closes the dialog after this duration, as if the user had cancelled it.
    pub timeout: Option<JsDuration>,

    /// Abort signal to close the dialog.
    pub signal: Option<JsAbortSignal>,
}

/// Options for `dialogs.selectOne()`.
///
/// ```ts
/// const fruit = await dialogs.selectOne("Pick a fruit:", ["Apple", "Pear"], {
///   selected: "Pear",
/// });
/// ```
/// @category Dialogs
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsSelectOneOptions {
    /// Title displayed in the dialog title bar.
    pub title: Option<String>,

    /// Initially selected item. The first item if omitted or not one of the items.
    pub selected: Option<String>,

    /// Closes the dialog after this duration, as if the user had cancelled it.
    pub timeout: Option<JsDuration>,

    /// Abort signal to close the dialog.
    pub signal: Option<JsAbortSignal>,
}

/// Options for `dialogs.selectMany()`.
///
/// ```ts
/// const fruits = await dialogs.selectMany("Pick fruits:", ["Apple", "Pear", "Plum"], {
///   selected: ["Apple", "Plum"],
/// });
/// ```
/// @category Dialogs
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsSelectManyOptions {
    /// Title displayed in the dialog title bar.
    pub title: Option<String>,

    /// Initially selected items. Strings that are not one of the items are ignored.
    pub selected: Option<Vec<String>>,

    /// Closes the dialog after this duration, as if the user had cancelled it.
    pub timeout: Option<JsDuration>,

    /// Abort signal to close the dialog.
    pub signal: Option<JsAbortSignal>,
}

/// Date dialog options.
///
/// ```ts
/// const date = await dialogs.date("Pick a date:", { value: new Date(2030, 0, 1) });
/// ```
/// @category Dialogs
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsDateOptions {
    /// Title displayed in the dialog title bar.
    pub title: Option<String>,

    /// Initially selected day; its time of day is ignored. Today if omitted.
    pub value: Option<JsDate>,

    /// Closes the dialog after this duration, as if the user had cancelled it.
    pub timeout: Option<JsDuration>,

    /// Abort signal to close the dialog.
    pub signal: Option<JsAbortSignal>,
}

/// Progress dialog options.
///
/// ```ts
/// const progress = await dialogs.progress("Copying files…", {
///   title: "Copy",
///   cancellable: true,
///   value: 0,
/// });
/// ```
/// @category Dialogs
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsProgressOptions {
    /// Title displayed in the dialog title bar.
    pub title: Option<String>,

    /// Whether the dialog has a Cancel button.
    pub cancellable: bool,

    /// Initial progress, between 0 and 1. A busy bar is shown if omitted.
    pub value: Option<f64>,

    /// Abort signal to close the dialog, both while it opens and once it is open.
    pub signal: Option<JsAbortSignal>,
}

/// Options for `Progress.waitForCancel()`.
/// @category Dialogs
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsWaitForCancelOptions {
    /// Abort signal to stop waiting.
    pub signal: Option<JsAbortSignal>,
}

/// Dialog utilities.
///
/// Every dialog returns a task: cancelling it, aborting its `signal` or stopping the script closes
/// the dialog. Dialogs also accept a `timeout`, after which they close as if the user had
/// cancelled them; a message box then returns `MessageBoxResult.Timeout`.
///
/// ```ts
/// const result = await dialogs.messageBox("Hello, world!");
/// ```
///
/// ```ts
/// const result = await dialogs.messageBox("Delete this file?", {
///   title: "Confirm",
///   buttons: MessageBoxButtons.YesNo,
///   icon: MessageBoxIcon.Warning,
/// });
/// if (result === MessageBoxResult.Yes) {
///   println("Confirmed");
/// }
/// ```
///
/// ```ts
/// // Give up waiting for an answer after 10 seconds
/// const name = await dialogs.textInput("Enter your name:", { timeout: "10s" });
/// ```
///
/// @category Dialogs
/// @singleton
#[derive(Debug, JsLifetime)]
#[js_class]
pub struct JsDialogs {
    inner: Dialogs,
}

impl SingletonClass<'_> for JsDialogs {
    fn register_dependencies(ctx: &Ctx<'_>) -> Result<()> {
        register_enum::<JsMessageBoxButtons>(ctx)?;
        register_enum::<JsMessageBoxIcon>(ctx)?;
        register_enum::<JsMessageBoxResult>(ctx)?;
        register_enum::<JsTextInputMode>(ctx)?;
        register_host_class::<JsProgress>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsDialogs {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl JsDialogs {
    /// @skip
    #[must_use]
    pub const fn new(inner: Dialogs) -> Self {
        Self { inner }
    }

    fn open_file_dialog<'js, R, F, Fut>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsFileDialogOptions>,
        open: F,
    ) -> Result<Promise<'js>>
    where
        F: FnOnce(Dialogs, FileDialogOptions) -> Fut + 'js,
        Fut: Future<Output = dialogs::Result<Option<R>>> + 'js,
        R: PathsResult,
    {
        let mut options = options.0.unwrap_or_default();
        let signal = options.signal.take();
        let timeout = options.timeout.take().map(Into::into);
        let dialogs = self.inner.clone();

        task_with_token(ctx, signal, async move |ctx, token| {
            let paths = show(&token, timeout, open(dialogs, options.into()))
                .await
                .into_js_result(&ctx)?;
            Ok(R::to_js_paths(paths.flatten()))
        })
    }
}

/// Converts what a file dialog returns into what its JS method returns.
trait PathsResult: Sized {
    type Js: for<'js> rquickjs::IntoJs<'js> + 'static;

    /// `None` when the user cancelled or the dialog timed out.
    fn to_js_paths(result: Option<Self>) -> Self::Js;
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

impl PathsResult for PathBuf {
    type Js = Option<String>;

    fn to_js_paths(result: Option<Self>) -> Self::Js {
        result.map(path_to_string)
    }
}

impl PathsResult for Vec<PathBuf> {
    type Js = Vec<String>;

    fn to_js_paths(result: Option<Self>) -> Self::Js {
        result
            .unwrap_or_default()
            .into_iter()
            .map(path_to_string)
            .collect_vec()
    }
}

#[js_methods]
impl JsDialogs {
    /// Displays a message box and returns the button the user pressed.
    ///
    /// Closing the message box without pressing a button returns `MessageBoxResult.Cancel` if
    /// that button is shown, otherwise `MessageBoxResult.No`, otherwise `MessageBoxResult.Ok`.
    ///
    /// ```ts
    /// const result = await dialogs.messageBox("Operation complete");
    /// ```
    ///
    /// ```ts
    /// const result = await dialogs.messageBox("Save changes?", {
    ///   buttons: MessageBoxButtons.YesNoCancel,
    ///   labels: { yes: "Save", no: "Discard" },
    ///   timeout: "30s",
    /// });
    /// if (result === MessageBoxResult.Timeout) {
    ///   println("Nobody answered");
    /// }
    /// ```
    /// @returns Task<MessageBoxResult>
    pub fn message_box<'js>(
        &self,
        ctx: Ctx<'js>,
        text: String,
        options: Opt<JsMessageBoxOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let timeout = options.timeout.map(Into::into);
        let dialogs = self.inner.clone();
        let message_box_options = MessageBoxOptions {
            title: options.title.unwrap_or_default(),
            text,
            icon: options.icon.unwrap_or_default().into(),
            buttons: options.buttons.unwrap_or_default().into(),
            labels: options.labels.map(Into::into).unwrap_or_default(),
        };

        task_with_token(ctx, options.signal, async move |ctx, token| {
            let result = show(&token, timeout, dialogs.message_box(message_box_options))
                .await
                .into_js_result(&ctx)?;
            Ok(result.map_or(JsMessageBoxResult::Timeout, Into::into))
        })
    }

    /// Opens a file picker dialog and returns the selected file path, or `undefined` if
    /// cancelled.
    ///
    /// ```ts
    /// const path = await dialogs.pickFile({ title: "Open File" });
    /// if (path !== undefined) {
    ///   println(path);
    /// }
    /// ```
    /// @returns Task<string | undefined>
    pub fn pick_file<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsFileDialogOptions>,
    ) -> Result<Promise<'js>> {
        self.open_file_dialog(ctx, options, async move |dialogs, options| {
            dialogs.pick_file(options).await
        })
    }

    /// Opens a file picker dialog allowing multiple selections and returns the selected file
    /// paths.
    ///
    /// Returns an empty array if cancelled.
    ///
    /// ```ts
    /// const paths = await dialogs.pickFiles({ title: "Open Files" });
    /// for (const path of paths) {
    ///   println(path);
    /// }
    /// ```
    /// @returns Task<string[]>
    pub fn pick_files<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsFileDialogOptions>,
    ) -> Result<Promise<'js>> {
        self.open_file_dialog(ctx, options, async move |dialogs, options| {
            dialogs.pick_files(options).await
        })
    }

    /// Opens a folder picker dialog and returns the selected folder path, or `undefined` if
    /// cancelled.
    ///
    /// ```ts
    /// const path = await dialogs.pickFolder({ title: "Select Folder" });
    /// ```
    /// @returns Task<string | undefined>
    pub fn pick_folder<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsFileDialogOptions>,
    ) -> Result<Promise<'js>> {
        self.open_file_dialog(ctx, options, async move |dialogs, options| {
            dialogs.pick_folder(options).await
        })
    }

    /// Opens a folder picker dialog allowing multiple selections and returns the selected
    /// folder paths.
    ///
    /// Returns an empty array if cancelled. Not supported by kdialog unless the
    /// xdg-desktop-portal file chooser is available.
    ///
    /// ```ts
    /// const paths = await dialogs.pickFolders({ title: "Select Folders" });
    /// ```
    /// @returns Task<string[]>
    pub fn pick_folders<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsFileDialogOptions>,
    ) -> Result<Promise<'js>> {
        self.open_file_dialog(ctx, options, async move |dialogs, options| {
            dialogs.pick_folders(options).await
        })
    }

    /// Opens a save file dialog and returns the chosen file path, or `undefined` if cancelled.
    ///
    /// ```ts
    /// const path = await dialogs.saveFile({
    ///   title: "Save As",
    ///   fileName: "report.txt",
    ///   filters: [{ name: "Text Files", extensions: ["txt"] }],
    /// });
    /// ```
    /// @returns Task<string | undefined>
    pub fn save_file<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsFileDialogOptions>,
    ) -> Result<Promise<'js>> {
        self.open_file_dialog(ctx, options, async move |dialogs, options| {
            dialogs.save_file(options).await
        })
    }

    /// Opens a text input dialog and returns the entered text, or `undefined` if cancelled.
    ///
    /// ```ts
    /// const name = await dialogs.textInput("Enter your name:", {
    ///   title: "Name",
    ///   mode: TextInputMode.SingleLine,
    /// });
    /// ```
    /// @returns Task<string | undefined>
    pub fn text_input<'js>(
        &self,
        ctx: Ctx<'js>,
        text: String,
        options: Opt<JsTextInputOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let timeout = options.timeout.map(Into::into);
        let dialogs = self.inner.clone();
        let text_input_options = TextInputOptions {
            title: options.title.unwrap_or_default(),
            text,
            value: options.value.unwrap_or_default(),
            mode: options.mode.unwrap_or_default().into(),
        };

        task_with_token(ctx, options.signal, async move |ctx, token| {
            let text = show(&token, timeout, dialogs.text_input(text_input_options))
                .await
                .into_js_result(&ctx)?;
            Ok(text.flatten())
        })
    }

    /// Opens a color picker dialog and returns the selected color, or `undefined` if cancelled.
    ///
    /// ```ts
    /// const color = await dialogs.colorPicker({
    ///   title: "Choose a color",
    ///   value: new Color(255, 0, 0),
    /// });
    /// if (color !== undefined) {
    ///   println(`${color}`);
    /// }
    /// ```
    /// @returns Task<Color | undefined>
    pub fn color_picker<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsColorPickerOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let timeout = options.timeout.map(Into::into);
        let dialogs = self.inner.clone();
        let mut color_picker_options = ColorPickerOptions {
            title: options.title.unwrap_or_default(),
            ..ColorPickerOptions::default()
        };
        if let Some(value) = options.value {
            color_picker_options.value = value.0;
        }

        task_with_token(ctx, options.signal, async move |ctx, token| {
            let color = show(&token, timeout, dialogs.color_picker(color_picker_options))
                .await
                .into_js_result(&ctx)?;
            Ok(color.flatten().map(JsColor::from))
        })
    }

    /// Asks the user to pick one of `items`, and returns it, or `undefined` if cancelled.
    ///
    /// ```ts
    /// const fruit = await dialogs.selectOne("Pick a fruit:", ["Apple", "Pear", "Plum"]);
    /// if (fruit !== undefined) {
    ///   println(`You picked ${fruit}`);
    /// }
    /// ```
    /// @returns Task<string | undefined>
    pub fn select_one<'js>(
        &self,
        ctx: Ctx<'js>,
        text: String,
        items: Vec<String>,
        options: Opt<JsSelectOneOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let timeout = options.timeout.map(Into::into);
        let dialogs = self.inner.clone();
        let select_options = select_options(options.title, text, items, options.selected);

        task_with_token(ctx, options.signal, async move |ctx, token| {
            let items = select_options.items.clone();
            let index = show(&token, timeout, dialogs.select_one(select_options))
                .await
                .into_js_result(&ctx)?;
            Ok(index.flatten().map(|index| items[index].clone()))
        })
    }

    /// Asks the user to pick any number of `items`, and returns them, or `undefined` if
    /// cancelled. Accepting without picking any item returns an empty array.
    ///
    /// ```ts
    /// const fruits = await dialogs.selectMany("Pick fruits:", ["Apple", "Pear", "Plum"], {
    ///   selected: ["Apple"],
    /// });
    /// ```
    /// @returns Task<string[] | undefined>
    pub fn select_many<'js>(
        &self,
        ctx: Ctx<'js>,
        text: String,
        items: Vec<String>,
        options: Opt<JsSelectManyOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let timeout = options.timeout.map(Into::into);
        let dialogs = self.inner.clone();
        let select_options = select_options(
            options.title,
            text,
            items,
            options.selected.unwrap_or_default(),
        );

        task_with_token(ctx, options.signal, async move |ctx, token| {
            let items = select_options.items.clone();
            let indices = show(&token, timeout, dialogs.select_many(select_options))
                .await
                .into_js_result(&ctx)?;
            Ok(indices.flatten().map(|indices| {
                indices
                    .into_iter()
                    .map(|index| items[index].clone())
                    .collect_vec()
            }))
        })
    }

    /// Asks the user to pick a date, and returns it at local midnight, or `undefined` if
    /// cancelled.
    ///
    /// ```ts
    /// const date = await dialogs.date("Pick a date:");
    /// if (date !== undefined) {
    ///   println(date.toDateString());
    /// }
    /// ```
    /// @returns Task<Date | undefined>
    pub fn date<'js>(
        &self,
        ctx: Ctx<'js>,
        text: String,
        options: Opt<JsDateOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let timeout = options.timeout.map(Into::into);
        let dialogs = self.inner.clone();
        let date_options = DateOptions {
            title: options.title.unwrap_or_default(),
            text,
            value: options.value.map(|value| value.0),
        };

        task_with_token(ctx, options.signal, async move |ctx, token| {
            let date = show(&token, timeout, dialogs.date(date_options))
                .await
                .into_js_result(&ctx)?;
            Ok(date.flatten().map(JsDate))
        })
    }

    /// Opens a progress dialog, and returns it once it is shown.
    ///
    /// The dialog stays open until `close()` is called, the `signal` is aborted, or the script
    /// ends.
    ///
    /// ```ts
    /// const progress = await dialogs.progress("Copying files…", { cancellable: true, value: 0 });
    /// for (const [i, file] of files.entries()) {
    ///   if (progress.cancelled) {
    ///     break;
    ///   }
    ///   progress.text = `Copying ${file}`;
    ///   progress.value = i / files.length;
    ///   await copy(file);
    /// }
    /// await progress.close();
    /// ```
    /// @returns Task<Progress>
    pub fn progress<'js>(
        &self,
        ctx: Ctx<'js>,
        text: String,
        options: Opt<JsProgressOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let dialogs = self.inner.clone();
        let progress_options = ProgressOptions {
            title: options.title.unwrap_or_default(),
            text,
            cancellable: options.cancellable,
            value: options.value,
        };

        task_with_token(ctx, options.signal, async move |ctx, token| {
            let progress = show(&token, None, dialogs.progress(progress_options.clone()))
                .await
                .into_js_result(&ctx)?
                .expect("there is no timeout");
            Ok(JsProgress::new(&ctx, progress, &progress_options, token))
        })
    }

    /// Returns a string representation of the `dialogs` singleton.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "Dialogs".to_owned()
    }
}

fn select_options(
    title: Option<String>,
    text: String,
    items: Vec<String>,
    selected: impl IntoIterator<Item = String>,
) -> SelectOptions {
    let selected = selected
        .into_iter()
        .filter_map(|selected| items.iter().position(|item| *item == selected))
        .collect_vec();
    SelectOptions {
        title: title.unwrap_or_default(),
        text,
        items,
        selected,
    }
}

/// An open progress dialog, returned by `dialogs.progress()`.
///
/// Updates are cheap: only the latest value and text are shown, so they can be set in a tight
/// loop.
///
/// ```ts
/// const progress = await dialogs.progress("Working…", { cancellable: true });
/// progress.value = 0.5;
/// progress.text = "Halfway there";
/// progress.value = undefined; // show a busy bar
/// await progress.close();
/// ```
///
/// @category Dialogs
/// @prop value: number | undefined // Progress between 0 and 1, or `undefined` (or `null`) for a busy bar
/// @prop text: string // Text shown above the progress bar
#[derive(JsLifetime)]
#[js_class]
pub struct JsProgress {
    /// `None` once closed.
    progress: Arc<Mutex<Option<Progress>>>,
    /// Cancelled when the user cancels or closes the dialog.
    cancelled: CancellationToken,
    /// Cancelled by `close()` or when this object is dropped.
    closed: CancellationToken,
    value: Option<f64>,
    text: String,
}

impl HostClass<'_> for JsProgress {}

impl<'js> Trace<'js> for JsProgress {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl Drop for JsProgress {
    fn drop(&mut self) {
        self.closed.cancel();
    }
}

impl JsProgress {
    /// Wraps an open dialog, which is closed when `token` is cancelled.
    fn new(
        ctx: &Ctx<'_>,
        progress: Progress,
        options: &ProgressOptions,
        token: CancellationToken,
    ) -> Self {
        let cancelled = progress.cancellation_token();
        let progress = Arc::new(Mutex::new(Some(progress)));
        let closed = CancellationToken::new();

        // Not spawned on the script engine, whose idle() would then wait for it.
        let weak_progress = Arc::downgrade(&progress);
        let local_closed = closed.clone();
        ctx.user_data().task_tracker().spawn(async move {
            select! {
                () = token.cancelled() => {
                    local_closed.cancel();
                    let progress = weak_progress.upgrade().and_then(|progress| progress.lock().take());
                    if let Some(progress) = progress {
                        progress.close().await;
                    }
                }
                () = local_closed.cancelled() => {}
            }
        });

        Self {
            progress,
            cancelled,
            closed,
            value: options.value.map(|value| value.clamp(0.0, 1.0)),
            text: options.text.clone(),
        }
    }
}

#[js_methods]
impl JsProgress {
    /// @skip
    #[get("value")]
    #[must_use]
    pub const fn get_value(&self) -> Option<f64> {
        self.value
    }

    /// @skip
    #[set("value")]
    pub fn set_value(&mut self, value: Option<f64>) {
        self.value = value.map(|value| value.clamp(0.0, 1.0));
        if let Some(progress) = &*self.progress.lock() {
            progress.set_value(self.value);
        }
    }

    /// @skip
    #[get("text")]
    #[must_use]
    pub fn get_text(&self) -> String {
        self.text.clone()
    }

    /// @skip
    #[set("text")]
    pub fn set_text(&mut self, text: String) {
        if let Some(progress) = &*self.progress.lock() {
            progress.set_text(&text);
        }
        self.text = text;
    }

    /// Whether the user cancelled or closed the dialog.
    #[get]
    #[must_use]
    pub fn cancelled(&self) -> bool {
        self.cancelled.is_cancelled()
    }

    /// Waits until the user cancels or closes the dialog, or `close()` is called.
    ///
    /// ```ts
    /// const progress = await dialogs.progress("Waiting…", { cancellable: true });
    /// await progress.waitForCancel();
    /// ```
    /// @returns Task<void>
    pub fn wait_for_cancel<'js>(
        &self,
        ctx: Ctx<'js>,
        options: Opt<JsWaitForCancelOptions>,
    ) -> Result<Promise<'js>> {
        let signal = options.0.and_then(|options| options.signal);
        let cancelled = self.cancelled.clone();
        let closed = self.closed.clone();

        task_with_token(ctx, signal, async move |ctx, token| {
            cancel_on(&token, async {
                select! {
                    () = cancelled.cancelled() => {}
                    () = closed.cancelled() => {}
                }
            })
            .await
            .into_js_result(&ctx)
        })
    }

    /// Closes the dialog, and resolves once it is gone. Does nothing if it is already closed.
    ///
    /// @returns Promise<void>
    pub fn close<'js>(&self, ctx: Ctx<'js>) -> Result<Promise<'js>> {
        self.closed.cancel();
        let progress = self.progress.lock().take();
        Promise::wrap_future(&ctx, async move {
            if let Some(progress) = progress {
                progress.close().await;
            }
            Ok::<_, rquickjs::Error>(())
        })
    }

    /// Returns a string representation of this progress dialog.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "Progress".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::JsMessageBoxResult;
    use crate::runtime::Runtime;

    #[test]
    fn select_from_no_items() {
        Runtime::test_with_script_engine(|script_engine| async move {
            for method in ["selectOne", "selectMany"] {
                let error = script_engine
                    .eval_async::<()>(&format!(r#"await dialogs.{method}("Pick", []);"#))
                    .await
                    .unwrap_err();
                assert!(
                    error.to_string().contains("no items"),
                    "{method}: unexpected error {error}"
                );
            }
        });
    }

    #[test]
    #[ignore]
    fn message_box() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<JsMessageBoxResult>(
                    r#"
                    await dialogs.messageBox("Actiona message box JS test", {
                        title: "dialogs.messageBox test",
                        buttons: MessageBoxButtons.OkCancel,
                        labels: { ok: "Save", cancel: "Discard" },
                        icon: MessageBoxIcon.Info,
                    });
                    "#,
                )
                .await
                .unwrap();
            println!("message_box result: {result:?}");
        });
    }

    #[test]
    #[ignore]
    fn message_box_timeout() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<JsMessageBoxResult>(
                    r#"
                    await dialogs.messageBox("This closes by itself after 2 seconds", {
                        title: "dialogs.messageBox timeout test",
                        timeout: "2s",
                    });
                    "#,
                )
                .await
                .unwrap();
            assert_eq!(result, JsMessageBoxResult::Timeout);
        });
    }

    #[test]
    #[ignore]
    fn pick_file() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let path = script_engine
                .eval_async::<Option<String>>(
                    r#"
                    await dialogs.pickFile({
                        title: "dialogs.pickFile test",
                        filters: [{ name: "Text Files", extensions: ["txt"] }],
                    });
                    "#,
                )
                .await
                .unwrap();
            println!("pick_file result: {path:?}");
        });
    }

    #[test]
    #[ignore]
    fn pick_files() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let paths = script_engine
                .eval_async::<Vec<String>>(
                    r#"
                    await dialogs.pickFiles({ title: "dialogs.pickFiles test" });
                    "#,
                )
                .await
                .unwrap();
            println!("pick_files result: {paths:?}");
        });
    }

    #[test]
    #[ignore]
    fn pick_folder() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let path = script_engine
                .eval_async::<Option<String>>(
                    r#"
                    await dialogs.pickFolder({ title: "dialogs.pickFolder test" });
                    "#,
                )
                .await
                .unwrap();
            println!("pick_folder result: {path:?}");
        });
    }

    #[test]
    #[ignore]
    fn pick_folders() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let paths = script_engine
                .eval_async::<Vec<String>>(
                    r#"
                    await dialogs.pickFolders({ title: "dialogs.pickFolders test" });
                    "#,
                )
                .await
                .unwrap();
            println!("pick_folders result: {paths:?}");
        });
    }

    #[test]
    #[ignore]
    fn save_file() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let path = script_engine
                .eval_async::<Option<String>>(
                    r#"
                    await dialogs.saveFile({
                        title: "dialogs.saveFile test",
                        fileName: "report.txt",
                        filters: [{ name: "Text Files", extensions: ["txt"] }],
                    });
                    "#,
                )
                .await
                .unwrap();
            println!("save_file result: {path:?}");
        });
    }

    #[test]
    #[ignore]
    fn text_input() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<Option<String>>(
                    r#"
                    await dialogs.textInput("Enter your name:", {
                        title: "dialogs.textInput test",
                        mode: TextInputMode.SingleLine,
                    });
                    "#,
                )
                .await
                .unwrap();
            println!("text_input result: {result:?}");
        });
    }

    #[test]
    #[ignore]
    fn color_picker() {
        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine
                .eval_async::<()>(
                    r#"
                    const color = await dialogs.colorPicker({
                        title: "dialogs.colorPicker test",
                        value: new Color(255, 128, 0),
                    });
                    println(`color_picker result: ${color}`);
                    "#,
                )
                .await
                .unwrap();
        });
    }

    #[test]
    #[ignore]
    fn select_one() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<Option<String>>(
                    r#"
                    await dialogs.selectOne("Pick a fruit:", ["Apple", "Pear", "Plum"], {
                        title: "dialogs.selectOne test",
                        selected: "Pear",
                    });
                    "#,
                )
                .await
                .unwrap();
            println!("select_one result: {result:?}");
        });
    }

    #[test]
    #[ignore]
    fn select_many() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval_async::<Option<Vec<String>>>(
                    r#"
                    await dialogs.selectMany("Pick fruits:", ["Apple", "Pear", "Plum"], {
                        title: "dialogs.selectMany test",
                        selected: ["Apple", "Plum"],
                    });
                    "#,
                )
                .await
                .unwrap();
            println!("select_many result: {result:?}");
        });
    }

    #[test]
    #[ignore]
    fn date() {
        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine
                .eval_async::<()>(
                    r#"
                    const date = await dialogs.date("Pick a date:", {
                        title: "dialogs.date test",
                        value: new Date(2030, 0, 15),
                    });
                    println(`date result: ${date}`);
                    "#,
                )
                .await
                .unwrap();
        });
    }

    #[test]
    #[ignore]
    fn progress() {
        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine
                .eval_async::<()>(
                    r#"
                    const progress = await dialogs.progress("Starting…", {
                        title: "dialogs.progress test",
                        cancellable: true,
                    });
                    await sleep("1s");
                    for (let i = 0; i <= 100 && !progress.cancelled; i++) {
                        progress.value = i / 100;
                        progress.text = `Step ${i}`;
                        await sleep("30ms");
                    }
                    println(`progress cancelled: ${progress.cancelled}`);
                    await progress.close();
                    "#,
                )
                .await
                .unwrap();
        });
    }
}
