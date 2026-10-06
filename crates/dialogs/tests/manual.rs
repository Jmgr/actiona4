//! Interactive tests: each one shows real dialogs and prints what they return. Run them with
//! `cargo make dialogs-manual`.
//!
//! On Linux, they run with every installed tool in turn, and with the portal for file dialogs.
//! Set `ACTIONA_DIALOGS_BACKEND` to `zenity`, `kdialog`, `portal` or `windows` to test a single
//! backend.

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf, time::Duration};

    use dialogs::{
        ButtonLabels, ColorPickerOptions, Dialogs, FileDialogOptions, FileFilter,
        MessageBoxButtons, MessageBoxIcon, MessageBoxOptions, ProgressOptions,
    };
    #[cfg(unix)]
    use dialogs::{
        DateOptions, LinuxBackends, LinuxTool, SelectOptions, TextInputMode, TextInputOptions,
    };
    #[cfg(unix)]
    use jiff::civil::date;
    #[cfg(unix)]
    use strum::IntoEnumIterator;
    use tokio::time::{sleep, timeout};
    use types::Color;

    /// Whether the backend named `name` should be tested: all of them, unless
    /// `ACTIONA_DIALOGS_BACKEND` names one.
    fn selected(name: &str) -> bool {
        env::var("ACTIONA_DIALOGS_BACKEND").map_or(true, |only| only == name)
    }

    /// Runs `test` with each backend: on Linux, once per installed tool, which shows every
    /// dialog.
    async fn for_each_backend(test: impl AsyncFn(&Dialogs, &str)) {
        #[cfg(unix)]
        for tool in LinuxTool::iter() {
            if !selected(tool.as_ref()) {
                continue;
            }
            if !tool.is_installed() {
                println!("{tool}: not installed, skipped");
                continue;
            }

            let dialogs = Dialogs::with_backends(LinuxBackends {
                portal: false,
                tool: Some(tool),
            });
            test(&dialogs, tool.as_ref()).await;
        }

        #[cfg(windows)]
        if selected("windows") {
            test(&Dialogs::new(), "windows").await;
        }
    }

    /// Runs `test` with each backend, starting on Linux with the portal.
    async fn for_each_file_backend(test: impl AsyncFn(&Dialogs, &str)) {
        #[cfg(unix)]
        if selected("portal") {
            let dialogs = Dialogs::with_backends(LinuxBackends {
                portal: true,
                tool: None,
            });
            test(&dialogs, "portal").await;
        }

        for_each_backend(test).await;
    }

    fn file_options(backend: &str, title: &str) -> FileDialogOptions {
        FileDialogOptions {
            title: format!("{backend}: {title}"),
            directory: env::var_os("HOME")
                .or_else(|| env::var_os("USERPROFILE"))
                .map(PathBuf::from),
            file_name: None,
            filters: vec![
                FileFilter {
                    name: "Images".to_owned(),
                    extensions: vec!["png".to_owned(), "jpg".to_owned()],
                },
                FileFilter {
                    name: "Text".to_owned(),
                    extensions: vec!["txt".to_owned()],
                },
            ],
        }
    }

    fn message_box(
        title: &str,
        text: &str,
        icon: MessageBoxIcon,
        buttons: MessageBoxButtons,
    ) -> MessageBoxOptions {
        MessageBoxOptions {
            title: title.to_owned(),
            text: text.to_owned(),
            icon,
            buttons,
            labels: ButtonLabels::default(),
        }
    }

    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn message_box_buttons_and_icons() {
        let cases = [
            (MessageBoxIcon::Info, MessageBoxButtons::Ok),
            (MessageBoxIcon::Warning, MessageBoxButtons::Ok),
            (MessageBoxIcon::Error, MessageBoxButtons::Ok),
            (MessageBoxIcon::Info, MessageBoxButtons::OkCancel),
            (MessageBoxIcon::Warning, MessageBoxButtons::YesNo),
            (MessageBoxIcon::Error, MessageBoxButtons::YesNoCancel),
        ];

        for_each_backend(async |dialogs, tool| {
            for (icon, buttons) in cases {
                let text = format!("{icon:?} icon with {buttons:?} buttons.\nPress any button.");
                let result = dialogs
                    .message_box(message_box(tool, &text, icon, buttons))
                    .await
                    .unwrap();
                println!("{tool} {icon:?} {buttons:?}: {result:?}");
            }
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn message_box_custom_labels() {
        for_each_backend(async |dialogs, tool| {
            let mut options = message_box(
                tool,
                "OK/Cancel relabelled Save/Discard.",
                MessageBoxIcon::Info,
                MessageBoxButtons::OkCancel,
            );
            options.labels.ok = Some("Save".to_owned());
            options.labels.cancel = Some("Discard".to_owned());
            let result = dialogs.message_box(options).await.unwrap();
            println!("{tool} Save/Discard: {result:?}");

            let mut options = message_box(
                tool,
                "Yes/No/Cancel relabelled Proceed/Skip/Stop.",
                MessageBoxIcon::Warning,
                MessageBoxButtons::YesNoCancel,
            );
            options.labels = ButtonLabels {
                ok: None,
                cancel: Some("Stop".to_owned()),
                yes: Some("Proceed".to_owned()),
                no: Some("Skip".to_owned()),
            };
            let result = dialogs.message_box(options).await.unwrap();
            println!("{tool} Proceed/Skip/Stop: {result:?}");
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn message_box_text_is_shown_verbatim() {
        let text = "Everything below should appear exactly as written.\n\
                    <b>not bold</b> & <i>not italic</i>\n\
                    C:\\path\\name and a literal \\n\n\
                    -starts with a dash";

        for_each_backend(async |dialogs, tool| {
            let result = dialogs
                .message_box(message_box(
                    tool,
                    text,
                    MessageBoxIcon::Info,
                    MessageBoxButtons::Ok,
                ))
                .await
                .unwrap();
            println!("{tool} verbatim text: {result:?}");
        })
        .await;
    }

    #[cfg(unix)]
    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn text_input_modes() {
        for_each_backend(async |dialogs, tool| {
            for mode in [
                TextInputMode::SingleLine,
                TextInputMode::Password,
                TextInputMode::MultiLine,
            ] {
                let result = dialogs
                    .text_input(TextInputOptions {
                        title: tool.to_owned(),
                        text: format!("{mode:?} input with a C:\\path_name & co prompt:"),
                        value: "initial\\value".to_owned(),
                        mode,
                    })
                    .await
                    .unwrap();
                println!("{tool} {mode:?}: {result:?}");
            }
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn color_picker() {
        for_each_backend(async |dialogs, tool| {
            let result = dialogs
                .color_picker(ColorPickerOptions {
                    title: format!("{tool}: initially orange"),
                    value: Color::new(255, 128, 0, 255),
                })
                .await
                .unwrap();
            println!("{tool} colour: {result:?}");
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn dialog_closes_when_dropped() {
        for_each_backend(async |dialogs, tool| {
            let result = timeout(
                Duration::from_secs(3),
                dialogs.message_box(message_box(
                    tool,
                    "Don't touch this one: it should close itself after 3 seconds.",
                    MessageBoxIcon::Info,
                    MessageBoxButtons::OkCancel,
                )),
            )
            .await;
            assert!(
                result.is_err(),
                "the dialog was answered before the timeout"
            );
            println!("{tool}: closed by timeout");
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn pick_files() {
        for_each_file_backend(async |dialogs, backend| {
            let result = dialogs
                .pick_file(file_options(backend, "pick a file (images or text)"))
                .await;
            println!("{backend} pick_file: {result:?}");

            let result = dialogs
                .pick_files(file_options(backend, "pick several files"))
                .await;
            println!("{backend} pick_files: {result:?}");
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn pick_folders() {
        for_each_file_backend(async |dialogs, backend| {
            let result = dialogs
                .pick_folder(file_options(backend, "pick a folder"))
                .await;
            println!("{backend} pick_folder: {result:?}");

            let result = dialogs
                .pick_folders(file_options(backend, "pick several folders"))
                .await;
            println!("{backend} pick_folders: {result:?}");
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn save_file() {
        for_each_file_backend(async |dialogs, backend| {
            let mut options = file_options(backend, "save (nothing is written)");
            options.file_name = Some("actiona test.txt".to_owned());
            let result = dialogs.save_file(options).await;
            println!("{backend} save_file: {result:?}");
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn file_dialog_closes_when_dropped() {
        for_each_file_backend(async |dialogs, backend| {
            let result = timeout(
                Duration::from_secs(3),
                dialogs.pick_file(file_options(
                    backend,
                    "don't touch this one: it closes itself after 3 seconds",
                )),
            )
            .await;
            assert!(
                result.is_err(),
                "{backend}: the dialog ended before the timeout"
            );
            println!("{backend}: closed by timeout");
        })
        .await;
    }

    /// Text that every dialog should show exactly as written.
    const VERBATIM: &str = "<b>not bold</b> & C:\\path_name";

    #[cfg(unix)]
    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn select_lists() {
        for_each_backend(async |dialogs, tool| {
            let mut options = SelectOptions {
                title: tool.to_owned(),
                text: format!("Pick one (the third is preselected). {VERBATIM}"),
                items: vec![
                    "First".to_owned(),
                    "-starts with a dash".to_owned(),
                    VERBATIM.to_owned(),
                    "Fourth".to_owned(),
                ],
                selected: vec![2],
            };
            let result = dialogs.select_one(options.clone()).await;
            println!("{tool} select_one: {result:?}");

            options.text = "Pick several (the first and last are preselected):".to_owned();
            options.selected = vec![0, 3];
            let result = dialogs.select_many(options).await;
            println!("{tool} select_many: {result:?}");
        })
        .await;
    }

    #[cfg(unix)]
    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn date_picker() {
        for_each_backend(async |dialogs, tool| {
            let result = dialogs
                .date(DateOptions {
                    title: tool.to_owned(),
                    text: format!("Initially 3 February 2026. {VERBATIM}"),
                    value: Some(date(2026, 2, 3)),
                })
                .await;
            println!("{tool} date: {result:?}");
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn progress_dialog() {
        for_each_backend(async |dialogs, tool| {
            let progress = dialogs
                .progress(ProgressOptions {
                    title: tool.to_owned(),
                    text: format!("Busy for 2 seconds, then counting to 50. {VERBATIM}"),
                    cancellable: true,
                    value: None,
                })
                .await
                .unwrap();
            sleep(Duration::from_secs(2)).await;

            for step in 0..=50_u32 {
                if progress.is_cancelled() {
                    break;
                }
                progress.set_value(Some(f64::from(step) / 50.0));
                progress.set_text(&format!("Step {step} of 50. {VERBATIM}"));
                sleep(Duration::from_millis(100)).await;
            }

            let cancelled = progress.is_cancelled();
            progress.close().await;
            println!("{tool} progress: cancelled = {cancelled}");
        })
        .await;
    }

    #[tokio::test]
    #[ignore = "shows dialogs"]
    async fn progress_closes_when_dropped() {
        for_each_backend(async |dialogs, tool| {
            let progress = dialogs
                .progress(ProgressOptions {
                    title: tool.to_owned(),
                    text: "Don't touch this one: it closes itself after 3 seconds.".to_owned(),
                    cancellable: true,
                    value: Some(0.5),
                })
                .await
                .unwrap();
            sleep(Duration::from_secs(3)).await;
            assert!(!progress.is_cancelled(), "{tool}: the dialog was cancelled");
            drop(progress);
            println!("{tool}: dropped");
        })
        .await;
    }
}
