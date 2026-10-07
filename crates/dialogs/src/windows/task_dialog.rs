//! Message boxes and progress dialogs, shown as task dialogs.
//!
//! Task dialogs call back every 200 milliseconds (`TDF_CALLBACK_TIMER`), which is when they check
//! whether they should close, and when progress dialogs show their latest state. Everything runs
//! on the dialog's thread.

use std::{
    cell::{Cell, RefCell},
    mem, ptr,
    sync::OnceLock,
    thread,
};

use tokio::sync::{oneshot, watch};
use tokio_util::sync::CancellationToken;
use tracing::debug;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, S_FALSE, S_OK, WPARAM},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
        UI::{
            Controls::{
                TASKDIALOG_BUTTON, TASKDIALOG_COMMON_BUTTON_FLAGS, TASKDIALOG_MESSAGES,
                TASKDIALOG_NOTIFICATIONS, TASKDIALOGCONFIG, TASKDIALOGCONFIG_0, TD_ERROR_ICON,
                TD_INFORMATION_ICON, TD_WARNING_ICON, TDCBF_CANCEL_BUTTON, TDCBF_NO_BUTTON,
                TDCBF_OK_BUTTON, TDCBF_YES_BUTTON, TDE_CONTENT, TDF_ALLOW_DIALOG_CANCELLATION,
                TDF_CALLBACK_TIMER, TDF_SHOW_MARQUEE_PROGRESS_BAR, TDF_SHOW_PROGRESS_BAR,
                TDM_CLICK_BUTTON, TDM_ENABLE_BUTTON, TDM_SET_ELEMENT_TEXT,
                TDM_SET_MARQUEE_PROGRESS_BAR, TDM_SET_PROGRESS_BAR_MARQUEE,
                TDM_SET_PROGRESS_BAR_POS, TDM_SET_PROGRESS_BAR_RANGE, TDN_BUTTON_CLICKED,
                TDN_CREATED, TDN_TIMER,
            },
            WindowsAndMessaging::{IDCANCEL, IDNO, IDOK, IDYES, MESSAGEBOX_RESULT, SendMessageW},
        },
    },
    core::{BOOL, HRESULT, HSTRING, PCWSTR, s, w},
};

use super::{struct_size, thread::DialogThread};
use crate::{
    ButtonLabels, Error, MessageBoxButtons, MessageBoxIcon, MessageBoxOptions, MessageBoxResult,
    Progress, ProgressOptions, Result, progress::ProgressState,
};

type TaskDialogIndirect =
    unsafe extern "system" fn(*const TASKDIALOGCONFIG, *mut i32, *mut i32, *mut BOOL) -> HRESULT;

/// `TaskDialogIndirect` is only in version 6 of the common controls, which the executable has to
/// request in its manifest. Looking it up at run time, rather than importing it, turns a missing
/// manifest into an error from this dialog, rather than a process that fails to start.
fn task_dialog_indirect() -> Result<TaskDialogIndirect> {
    static FUNCTION: OnceLock<Option<TaskDialogIndirect>> = OnceLock::new();

    let function = FUNCTION.get_or_init(|| {
        // SAFETY: loads a system library by name.
        let module = unsafe { LoadLibraryW(w!("comctl32.dll")) }.ok()?;
        // SAFETY: `module` is a loaded library.
        let address = unsafe { GetProcAddress(module, s!("TaskDialogIndirect")) }?;
        // SAFETY: this is the signature of `TaskDialogIndirect`.
        Some(unsafe {
            mem::transmute::<unsafe extern "system" fn() -> isize, TaskDialogIndirect>(address)
        })
    });

    function.ok_or_else(|| {
        Error::Backend(
            "TaskDialogIndirect is unavailable: the executable needs a manifest that requests \
             Common Controls 6"
                .to_owned(),
        )
    })
}

fn send(window: HWND, message: TASKDIALOG_MESSAGES, wparam: usize, lparam: isize) {
    // SAFETY: `window` is a task dialog, and pointers passed in `lparam` outlive this synchronous
    // call.
    _ = unsafe {
        SendMessageW(
            window,
            message.0 as u32,
            Some(WPARAM(wparam)),
            Some(LPARAM(lparam)),
        )
    };
}

fn click_cancel(window: HWND) {
    send(window, TDM_CLICK_BUTTON, IDCANCEL.0 as usize, 0);
}

/// A task dialog's buttons: common buttons, which the system labels, unless any is relabelled.
/// Then all of them are custom buttons, since task dialogs show custom buttons before common
/// ones, which would mix up their order.
#[derive(Debug, Eq, PartialEq)]
enum Buttons {
    Common(TASKDIALOG_COMMON_BUTTON_FLAGS),
    Custom(Vec<(MESSAGEBOX_RESULT, String)>),
}

fn message_box_buttons(buttons: MessageBoxButtons, labels: &ButtonLabels) -> Buttons {
    let ok = (IDOK, TDCBF_OK_BUTTON, &labels.ok, "OK");
    let cancel = (IDCANCEL, TDCBF_CANCEL_BUTTON, &labels.cancel, "Cancel");
    let yes = (IDYES, TDCBF_YES_BUTTON, &labels.yes, "Yes");
    let no = (IDNO, TDCBF_NO_BUTTON, &labels.no, "No");
    let all = match buttons {
        MessageBoxButtons::Ok => vec![ok],
        MessageBoxButtons::OkCancel => vec![ok, cancel],
        MessageBoxButtons::YesNo => vec![yes, no],
        MessageBoxButtons::YesNoCancel => vec![yes, no, cancel],
    };

    if all.iter().all(|(_, _, label, _)| label.is_none()) {
        return Buttons::Common(all.iter().fold(
            TASKDIALOG_COMMON_BUTTON_FLAGS(0),
            |flags, (_, flag, _, _)| flags | *flag,
        ));
    }

    Buttons::Custom(
        all.into_iter()
            .map(|(id, _, label, default)| (id, label.as_deref().unwrap_or(default).to_owned()))
            .collect(),
    )
}

/// Closing the dialog, which is allowed even without a Cancel button, is reported as Cancel.
const fn message_box_result(buttons: MessageBoxButtons, pressed: i32) -> MessageBoxResult {
    match MESSAGEBOX_RESULT(pressed) {
        IDOK => MessageBoxResult::Ok,
        IDYES => MessageBoxResult::Yes,
        IDNO => MessageBoxResult::No,
        _ => buttons.dismissed_result(),
    }
}

/// Shows a message box on the current thread, closing it once `thread` is closed.
pub fn message_box(options: &MessageBoxOptions, thread: &DialogThread) -> Result<MessageBoxResult> {
    let task_dialog_indirect = task_dialog_indirect()?;
    let title = HSTRING::from(&options.title);
    let text = HSTRING::from(&options.text);
    let icon = match options.icon {
        MessageBoxIcon::Info => TD_INFORMATION_ICON,
        MessageBoxIcon::Warning => TD_WARNING_ICON,
        MessageBoxIcon::Error => TD_ERROR_ICON,
    };

    let mut config = TASKDIALOGCONFIG {
        cbSize: struct_size::<TASKDIALOGCONFIG>(),
        dwFlags: TDF_ALLOW_DIALOG_CANCELLATION | TDF_CALLBACK_TIMER,
        pszWindowTitle: PCWSTR(title.as_ptr()),
        Anonymous1: TASKDIALOGCONFIG_0 { pszMainIcon: icon },
        pszContent: PCWSTR(text.as_ptr()),
        pfCallback: Some(message_box_callback),
        lpCallbackData: ptr::from_ref(thread) as isize,
        ..TASKDIALOGCONFIG::default()
    };

    let labels: Vec<(MESSAGEBOX_RESULT, HSTRING)>;
    let custom_buttons: Vec<TASKDIALOG_BUTTON>;
    match message_box_buttons(options.buttons, &options.labels) {
        Buttons::Common(flags) => config.dwCommonButtons = flags,
        Buttons::Custom(buttons) => {
            labels = buttons
                .into_iter()
                .map(|(id, label)| (id, HSTRING::from(label)))
                .collect();
            custom_buttons = labels
                .iter()
                .map(|(id, label)| TASKDIALOG_BUTTON {
                    nButtonID: id.0,
                    pszButtonText: PCWSTR(label.as_ptr()),
                })
                .collect();
            config.cButtons = u32::try_from(custom_buttons.len()).unwrap_or(0);
            config.pButtons = custom_buttons.as_ptr();
        }
    }

    let mut pressed = 0;
    // SAFETY: `config` and everything it points to outlive the call.
    unsafe {
        task_dialog_indirect(
            &raw const config,
            &raw mut pressed,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    }
    .ok()?;

    Ok(message_box_result(options.buttons, pressed))
}

unsafe extern "system" fn message_box_callback(
    window: HWND,
    notification: TASKDIALOG_NOTIFICATIONS,
    _: WPARAM,
    _: LPARAM,
    data: isize,
) -> HRESULT {
    // SAFETY: `data` is the `DialogThread` passed in the dialog's configuration, which outlives
    // the dialog.
    let thread = unsafe { &*(data as *const DialogThread) };
    if notification == TDN_TIMER && thread.is_closed() {
        click_cancel(window);
    }
    S_OK
}

/// The progress bar's range: values are shown in thousandths.
const PROGRESS_STEPS: u16 = 1000;

/// The range as `TDM_SET_PROGRESS_BAR_RANGE` expects it: the minimum in the low word, and the
/// maximum in the high word.
#[allow(clippy::cast_possible_wrap)]
const PROGRESS_RANGE: isize = (PROGRESS_STEPS as isize) << 16;

#[allow(clippy::cast_possible_truncation)]
fn progress_steps(value: f64) -> usize {
    (value.clamp(0.0, 1.0) * f64::from(PROGRESS_STEPS)).round() as usize
}

/// State of a progress dialog, owned by its thread. The callback can be reentered, for example
/// when clicking Cancel from the timer notification, so it is only ever borrowed briefly.
struct ProgressDialog {
    requested: RefCell<watch::Receiver<ProgressState>>,
    shown: RefCell<ProgressState>,
    cancellable: bool,
    /// Set when the handle closes the dialog, so that its click on Cancel is let through, and is
    /// not reported as the user cancelling.
    closing: Cell<bool>,
    cancelled: CancellationToken,
    created: Cell<Option<oneshot::Sender<Result<()>>>>,
}

impl ProgressDialog {
    fn on_created(&self, window: HWND) {
        send(window, TDM_SET_PROGRESS_BAR_RANGE, 0, PROGRESS_RANGE);
        if !self.cancellable {
            // A task dialog without buttons has an OK button: it is disabled until the dialog
            // closes.
            send(window, TDM_ENABLE_BUTTON, IDOK.0 as usize, 0);
        }
        let shown = self.shown.borrow().clone();
        Self::show(window, None, &shown);

        if let Some(created) = self.created.take() {
            _ = created.send(Ok(()));
        }
    }

    fn on_timer(&self, window: HWND) {
        let requested = {
            let mut requested = self.requested.borrow_mut();
            match requested.has_changed() {
                Ok(false) => return,
                Ok(true) => Some(requested.borrow_and_update().clone()),
                // The handle was closed or dropped.
                Err(_) => None,
            }
        };

        if let Some(state) = requested {
            let previous = self.shown.replace(state.clone());
            Self::show(window, Some(&previous), &state);
        } else {
            self.closing.set(true);
            click_cancel(window);
        }
    }

    /// Lets the dialog close, unless the user tries to close a dialog that cannot be cancelled,
    /// with Escape, Alt+F4 or the title bar's close button.
    fn on_button_clicked(&self) -> HRESULT {
        if self.closing.get() {
            return S_OK;
        }
        if !self.cancellable {
            return S_FALSE;
        }
        self.cancelled.cancel();
        S_OK
    }

    /// Shows `to` in the dialog, which currently shows `from`, or has just been created if `from`
    /// is `None`.
    fn show(window: HWND, from: Option<&ProgressState>, to: &ProgressState) {
        let shown = from.map(|from| from.value.map(progress_steps));
        let steps = to.value.map(progress_steps);

        if shown.is_none_or(|shown| shown.is_some() != steps.is_some()) {
            send(
                window,
                TDM_SET_MARQUEE_PROGRESS_BAR,
                usize::from(steps.is_none()),
                0,
            );
            if steps.is_none() {
                // Starts the animation, at the default speed.
                send(window, TDM_SET_PROGRESS_BAR_MARQUEE, 1, 0);
            } else {
                send(window, TDM_SET_PROGRESS_BAR_RANGE, 0, PROGRESS_RANGE);
            }
        }
        if let Some(steps) = steps
            && shown.is_none_or(|shown| shown != Some(steps))
        {
            send(window, TDM_SET_PROGRESS_BAR_POS, steps, 0);
        }
        if from.is_some_and(|from| from.text != to.text) {
            let text = HSTRING::from(&to.text);
            send(
                window,
                TDM_SET_ELEMENT_TEXT,
                TDE_CONTENT.0 as usize,
                text.as_ptr() as isize,
            );
        }
    }
}

unsafe extern "system" fn progress_callback(
    window: HWND,
    notification: TASKDIALOG_NOTIFICATIONS,
    _: WPARAM,
    _: LPARAM,
    data: isize,
) -> HRESULT {
    // SAFETY: `data` is the `ProgressDialog` passed in the dialog's configuration, which outlives
    // the dialog.
    let dialog = unsafe { &*(data as *const ProgressDialog) };
    match notification {
        TDN_CREATED => dialog.on_created(window),
        TDN_TIMER => dialog.on_timer(window),
        TDN_BUTTON_CLICKED => return dialog.on_button_clicked(),
        _ => {}
    }
    S_OK
}

fn show_progress_dialog(
    task_dialog_indirect: TaskDialogIndirect,
    title: &str,
    dialog: &ProgressDialog,
) -> Result<()> {
    let title = HSTRING::from(title);
    let text = HSTRING::from(&dialog.shown.borrow().text);
    let bar = if dialog.shown.borrow().value.is_some() {
        TDF_SHOW_PROGRESS_BAR
    } else {
        TDF_SHOW_MARQUEE_PROGRESS_BAR
    };

    let config = TASKDIALOGCONFIG {
        cbSize: struct_size::<TASKDIALOGCONFIG>(),
        dwFlags: TDF_ALLOW_DIALOG_CANCELLATION | TDF_CALLBACK_TIMER | bar,
        dwCommonButtons: if dialog.cancellable {
            TDCBF_CANCEL_BUTTON
        } else {
            TASKDIALOG_COMMON_BUTTON_FLAGS(0)
        },
        pszWindowTitle: PCWSTR(title.as_ptr()),
        pszContent: PCWSTR(text.as_ptr()),
        pfCallback: Some(progress_callback),
        lpCallbackData: ptr::from_ref(dialog) as isize,
        ..TASKDIALOGCONFIG::default()
    };

    // SAFETY: `config` and everything it points to outlive the call.
    unsafe {
        task_dialog_indirect(
            &raw const config,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        )
    }
    .ok()?;
    Ok(())
}

/// Shows a progress dialog on a thread of its own, and returns once it is shown.
pub async fn progress(options: &ProgressOptions) -> Result<Progress> {
    let task_dialog_indirect = task_dialog_indirect()?;
    let shown = ProgressState::initial(options);
    let (state, requested) = watch::channel(shown.clone());
    let cancelled = CancellationToken::new();
    let (created_sender, created) = oneshot::channel();
    let (finished_sender, finished) = oneshot::channel::<()>();

    let title = options.title.clone();
    let cancellable = options.cancellable;
    let thread_cancelled = cancelled.clone();
    thread::Builder::new()
        .name("progress dialog".to_owned())
        .spawn(move || {
            let dialog = ProgressDialog {
                requested: RefCell::new(requested),
                shown: RefCell::new(shown),
                cancellable,
                closing: Cell::new(false),
                cancelled: thread_cancelled,
                created: Cell::new(Some(created_sender)),
            };

            let result = show_progress_dialog(task_dialog_indirect, &title, &dialog);
            if !dialog.closing.get() {
                dialog.cancelled.cancel();
            }
            if let Err(error) = result {
                if let Some(created) = dialog.created.take() {
                    _ = created.send(Err(error));
                } else {
                    debug!("the progress dialog failed: {error}");
                }
            }
            _ = finished_sender.send(());
        })?;

    created.await.map_err(|_| {
        Error::Backend("the progress dialog's thread ended without showing it".to_owned())
    })??;

    let task = tokio::spawn(async move {
        _ = finished.await;
    });
    Ok(Progress::new(state, cancelled, task))
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::{
        Controls::{TASKDIALOG_COMMON_BUTTON_FLAGS, TDCBF_CANCEL_BUTTON, TDCBF_OK_BUTTON},
        WindowsAndMessaging::{IDCANCEL, IDNO, IDOK, IDYES},
    };

    use super::{Buttons, message_box_buttons, message_box_result, progress_steps};
    use crate::{ButtonLabels, MessageBoxButtons, MessageBoxResult};

    #[test]
    fn common_buttons_unless_relabelled() {
        assert_eq!(
            message_box_buttons(MessageBoxButtons::OkCancel, &ButtonLabels::default()),
            Buttons::Common(TDCBF_OK_BUTTON | TDCBF_CANCEL_BUTTON)
        );

        let labels = ButtonLabels {
            no: Some("Skip".to_owned()),
            ok: Some("Ignored".to_owned()),
            ..ButtonLabels::default()
        };
        assert_eq!(
            message_box_buttons(MessageBoxButtons::YesNoCancel, &labels),
            Buttons::Custom(vec![
                (IDYES, "Yes".to_owned()),
                (IDNO, "Skip".to_owned()),
                (IDCANCEL, "Cancel".to_owned()),
            ])
        );
        assert_eq!(
            message_box_buttons(MessageBoxButtons::YesNo, &labels),
            Buttons::Custom(vec![(IDYES, "Yes".to_owned()), (IDNO, "Skip".to_owned())])
        );
        assert_eq!(
            message_box_buttons(MessageBoxButtons::Ok, &ButtonLabels::default()),
            Buttons::Common(TASKDIALOG_COMMON_BUTTON_FLAGS(1))
        );
    }

    #[test]
    fn results() {
        assert_eq!(
            message_box_result(MessageBoxButtons::YesNo, IDYES.0),
            MessageBoxResult::Yes
        );
        assert_eq!(
            message_box_result(MessageBoxButtons::YesNo, IDCANCEL.0),
            MessageBoxResult::No
        );
        assert_eq!(
            message_box_result(MessageBoxButtons::OkCancel, IDCANCEL.0),
            MessageBoxResult::Cancel
        );
        assert_eq!(
            message_box_result(MessageBoxButtons::Ok, IDOK.0),
            MessageBoxResult::Ok
        );
    }

    #[test]
    fn steps() {
        assert_eq!(progress_steps(0.4567), 457);
        assert_eq!(progress_steps(2.0), 1000);
    }
}
