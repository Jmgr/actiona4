use macros::{FromJsObject, js_class, js_methods, options};
use rquickjs::{
    Ctx, JsLifetime, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::Opt,
};

use crate::{
    IntoJsResult,
    api::{
        color::js::{JsColor, JsColorLike},
        dialogs::{
            Dialogs, MessageBoxButtons,
            native_dialog::{ColorPickerOptions, TextInputOptions},
        },
        js::classes::{HostClass, SingletonClass, register_enum, register_host_class},
    },
    runtime::WithUserData,
};

pub type JsMessageBoxIcon = super::MessageBoxIcon;
pub type JsMessageBoxResult = super::MessageBoxResult;
pub type JsTextInputMode = super::native_dialog::TextInputMode;

/// Message box options.
///
/// ```ts
/// await dialogs.messageBox("Delete this file?", {
///   title: "Confirm",
///   buttons: MessageBoxButtons.yesNo(),
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
    #[default(ts = "MessageBoxButtons.ok()")]
    pub buttons: Option<JsMessageBoxButtons>,

    /// Icon displayed in the message box.
    #[default(ts = "MessageBoxIcon.Info")]
    pub icon: Option<super::MessageBoxIcon>,
}

impl JsMessageBoxOptions {
    fn into_inner(self) -> super::MessageBoxOptions {
        super::MessageBoxOptions {
            title: self.title,
            buttons: self.buttons,
            icon: self.icon,
        }
    }
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

impl JsFileFilter {
    fn into_inner(self) -> super::file_dialog::FileFilter {
        super::file_dialog::FileFilter {
            name: self.name,
            extensions: self.extensions,
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

    /// File type filters shown in the dialog.
    pub filters: Option<Vec<JsFileFilter>>,
}

impl JsFileDialogOptions {
    fn into_inner(self) -> super::file_dialog::FileDialogOptions {
        super::file_dialog::FileDialogOptions {
            title: self.title,
            directory: self.directory,
            filters: self
                .filters
                .unwrap_or_default()
                .into_iter()
                .map(JsFileFilter::into_inner)
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

    /// Initial color shown in the picker.
    pub value: Option<JsColorLike>,
}

/// Dialog utilities.
///
/// Provides methods for displaying message boxes and file dialogs.
///
/// ```ts
/// const result = await dialogs.messageBox("Hello, world!");
/// ```
///
/// ```ts
/// const result = await dialogs.messageBox("Delete this file?", {
///   title: "Confirm",
///   buttons: MessageBoxButtons.yesNo(),
///   icon: MessageBoxIcon.Warning,
/// });
/// if (result === MessageBoxResult.Yes) {
///   println("Confirmed");
/// }
/// ```
///
/// @category Dialogs
/// @singleton
#[derive(Debug, Default, JsLifetime)]
#[js_class]
pub struct JsDialogs {}

impl SingletonClass<'_> for JsDialogs {
    fn register_dependencies(ctx: &Ctx<'_>) -> Result<()> {
        register_host_class::<JsMessageBoxButtons>(ctx)?;
        register_enum::<JsMessageBoxIcon>(ctx)?;
        register_enum::<JsMessageBoxResult>(ctx)?;
        register_enum::<JsTextInputMode>(ctx)?;
        Ok(())
    }
}

impl<'js> Trace<'js> for JsDialogs {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[js_methods]
impl JsDialogs {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new() -> Result<Self> {
        Ok(Self::default())
    }

    /// Displays a message box and returns the user's response.
    ///
    /// ```ts
    /// const result = await dialogs.messageBox("Operation complete");
    /// ```
    pub async fn message_box(
        &self,
        ctx: Ctx<'_>,
        text: String,
        options: Opt<JsMessageBoxOptions>,
    ) -> Result<JsMessageBoxResult> {
        let options = options.0.unwrap_or_default();
        Dialogs::message_box(text, Some(options.into_inner()))
            .await
            .into_js_result(&ctx)
    }

    /// Opens a file picker dialog and returns the selected file path, or `null` if cancelled.
    ///
    /// ```ts
    /// const path = await dialogs.pickFile({ title: "Open File" });
    /// if (path !== null) {
    ///   print(path);
    /// }
    /// ```
    pub async fn pick_file(
        &self,
        ctx: Ctx<'_>,
        options: Opt<JsFileDialogOptions>,
    ) -> Result<Option<String>> {
        Dialogs::pick_file(options.0.unwrap_or_default().into_inner())
            .await
            .map(|path| path.map(|path| path.to_string_lossy().into_owned()))
            .into_js_result(&ctx)
    }

    /// Opens a file picker dialog allowing multiple selections and returns the selected file paths.
    ///
    /// Returns an empty array if cancelled.
    ///
    /// ```ts
    /// const paths = await dialogs.pickFiles({ title: "Open Files" });
    /// for (const path of paths) {
    ///   console.log(path);
    /// }
    /// ```
    pub async fn pick_files(
        &self,
        ctx: Ctx<'_>,
        options: Opt<JsFileDialogOptions>,
    ) -> Result<Vec<String>> {
        Dialogs::pick_files(options.0.unwrap_or_default().into_inner())
            .await
            .map(|paths| {
                paths
                    .into_iter()
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect()
            })
            .into_js_result(&ctx)
    }

    /// Opens a folder picker dialog and returns the selected folder path, or `null` if cancelled.
    ///
    /// ```ts
    /// const path = await dialogs.pickFolder({ title: "Select Folder" });
    /// ```
    pub async fn pick_folder(
        &self,
        ctx: Ctx<'_>,
        options: Opt<JsFileDialogOptions>,
    ) -> Result<Option<String>> {
        Dialogs::pick_folder(options.0.unwrap_or_default().into_inner())
            .await
            .map(|path| path.map(|path| path.to_string_lossy().into_owned()))
            .into_js_result(&ctx)
    }

    /// Opens a folder picker dialog allowing multiple selections and returns the selected folder paths.
    ///
    /// Returns an empty array if cancelled.
    ///
    /// ```ts
    /// const paths = await dialogs.pickFolders({ title: "Select Folders" });
    /// ```
    pub async fn pick_folders(
        &self,
        ctx: Ctx<'_>,
        options: Opt<JsFileDialogOptions>,
    ) -> Result<Vec<String>> {
        Dialogs::pick_folders(options.0.unwrap_or_default().into_inner())
            .await
            .map(|paths| {
                paths
                    .into_iter()
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect()
            })
            .into_js_result(&ctx)
    }

    /// Opens a save file dialog and returns the chosen file path, or `null` if cancelled.
    ///
    /// ```ts
    /// const path = await dialogs.saveFile({
    ///   title: "Save As",
    ///   filters: [{ name: "Text Files", extensions: ["txt"] }],
    /// });
    /// ```
    pub async fn save_file(
        &self,
        ctx: Ctx<'_>,
        options: Opt<JsFileDialogOptions>,
    ) -> Result<Option<String>> {
        Dialogs::save_file(options.0.unwrap_or_default().into_inner())
            .await
            .map(|path| path.map(|path| path.to_string_lossy().into_owned()))
            .into_js_result(&ctx)
    }

    /// Opens a text input dialog and returns the entered text, or `null` if cancelled.
    ///
    /// ```ts
    /// const name = await dialogs.textInput("Enter your name:", {
    ///   title: "Name",
    ///   mode: TextInputMode.SingleLine,
    /// });
    /// ```
    pub async fn text_input(
        &self,
        ctx: Ctx<'_>,
        message: String,
        options: Opt<JsTextInputOptions>,
    ) -> Result<Option<String>> {
        let options = options.0.unwrap_or_default();
        let task_tracker = ctx.user_data().task_tracker();
        Dialogs::text_input(
            TextInputOptions {
                title: options.title.unwrap_or_default(),
                message,
                value: options.value.unwrap_or_default(),
                mode: options.mode.unwrap_or_default(),
            },
            task_tracker,
        )
        .await
        .into_js_result(&ctx)
    }

    /// Opens a color picker dialog and returns the selected color, or `null` if cancelled.
    ///
    /// ```ts
    /// const color = await dialogs.colorPicker({
    ///   title: "Choose a color",
    ///   value: new Color(255, 0, 0),
    /// });
    /// if (color !== null) {
    ///   print(`${color}`);
    /// }
    /// ```
    pub async fn color_picker(
        &self,
        ctx: Ctx<'_>,
        options: Opt<JsColorPickerOptions>,
    ) -> Result<Option<JsColor>> {
        let options = options.0.unwrap_or_default();
        let task_tracker = ctx.user_data().task_tracker();
        Dialogs::color_picker(
            ColorPickerOptions {
                title: options.title.unwrap_or_default(),
                value: options
                    .value
                    .map_or(crate::api::color::Color::new(0, 0, 0, 255), |color| color.0),
            },
            task_tracker,
        )
        .await
        .map(|color| color.map(JsColor::from))
        .into_js_result(&ctx)
    }

    /// Returns a string representation of the `dialogs` singleton.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "Dialogs".to_string()
    }
}

/// Button configurations for message boxes.
///
/// Use the static factory methods to create button sets.
///
/// ```ts
/// const buttons = MessageBoxButtons.ok();
/// const buttons2 = MessageBoxButtons.yesNoCancel();
/// const buttons3 = MessageBoxButtons.okCancelCustom("Save", "Discard");
/// ```
/// @category Dialogs
#[derive(Clone, Debug, Default, JsLifetime)]
#[js_class]
pub struct JsMessageBoxButtons {
    inner: MessageBoxButtons,
}

impl<'js> Trace<'js> for JsMessageBoxButtons {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl HostClass<'_> for JsMessageBoxButtons {}

impl JsMessageBoxButtons {
    /// @skip
    #[must_use]
    pub fn into_inner(self) -> MessageBoxButtons {
        self.inner
    }
}

#[js_methods]
impl JsMessageBoxButtons {
    /// @constructor
    /// @private
    #[qjs(constructor)]
    pub fn new(ctx: Ctx<'_>) -> Result<Self> {
        Err(rquickjs::Exception::throw_message(
            &ctx,
            "MessageBoxButtons cannot be instantiated directly",
        ))
    }

    /// Creates an OK button.
    #[qjs(static)]
    #[must_use]
    pub const fn ok() -> Self {
        Self {
            inner: MessageBoxButtons::Ok,
        }
    }

    /// Creates an OK button with a custom label.
    #[qjs(static)]
    #[must_use]
    pub const fn ok_custom(ok_label: String) -> Self {
        Self {
            inner: MessageBoxButtons::OkCustom(ok_label),
        }
    }

    /// Creates OK and Cancel buttons.
    #[qjs(static)]
    #[must_use]
    pub const fn ok_cancel() -> Self {
        Self {
            inner: MessageBoxButtons::OkCancel,
        }
    }

    /// Creates OK and Cancel buttons with custom labels.
    #[qjs(static)]
    #[must_use]
    pub const fn ok_cancel_custom(ok_label: String, cancel_label: String) -> Self {
        Self {
            inner: MessageBoxButtons::OkCancelCustom(ok_label, cancel_label),
        }
    }

    /// Creates Yes and No buttons.
    #[qjs(static)]
    #[must_use]
    pub const fn yes_no() -> Self {
        Self {
            inner: MessageBoxButtons::YesNo,
        }
    }

    /// Creates Yes, No, and Cancel buttons.
    #[qjs(static)]
    #[must_use]
    pub const fn yes_no_cancel() -> Self {
        Self {
            inner: MessageBoxButtons::YesNoCancel,
        }
    }

    /// Creates Yes, No, and Cancel buttons with custom labels.
    #[qjs(static)]
    #[must_use]
    pub const fn yes_no_cancel_custom(
        yes_label: String,
        no_label: String,
        cancel_label: String,
    ) -> Self {
        Self {
            inner: MessageBoxButtons::YesNoCancelCustom(yes_label, no_label, cancel_label),
        }
    }

    /// Returns a string representation of this set of message box buttons.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "MessageBoxButtons".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::JsMessageBoxResult;
    use crate::runtime::Runtime;

    #[test]
    #[ignore]
    fn test_message_box() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let _ = script_engine
                .eval_async::<JsMessageBoxResult>(
                    r#"
                    await dialogs.messageBox("Actiona message box JS test", {
                        title: "dialogs.messageBox test",
                        buttons: MessageBoxButtons.okCancelCustom("Save", "Discard"),
                        icon: MessageBoxIcon.Info,
                    });
                    "#,
                )
                .await
                .unwrap();
        });
    }

    #[test]
    #[ignore]
    fn test_pick_file() {
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
    fn test_pick_files() {
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
    fn test_pick_folder() {
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
    fn test_pick_folders() {
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
    fn test_save_file() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let path = script_engine
                .eval_async::<Option<String>>(
                    r#"
                    await dialogs.saveFile({
                        title: "dialogs.saveFile test",
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
    fn test_text_input() {
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
    fn test_color_picker() {
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
}
