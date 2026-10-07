//! Runs each dialog on a thread of its own, so that its modal loop never blocks the async
//! runtime, and closes the dialog when its future is dropped.

use std::{sync::Arc, thread};

use parking_lot::Mutex;
use tokio::sync::oneshot;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        System::Threading::GetCurrentThreadId,
        UI::WindowsAndMessaging::{
            EnumThreadWindows, MSG, PM_NOREMOVE, PeekMessageW, PostMessageW, PostThreadMessageW,
            WM_CLOSE, WM_QUIT, WM_USER,
        },
    },
    core::BOOL,
};

use crate::{Error, Result};

#[derive(Debug, Default)]
struct State {
    /// Whether the dialog's future was dropped.
    closed: bool,
    /// The dialog's thread, while it is running.
    thread_id: Option<u32>,
}

/// State shared between a dialog's thread and its future.
#[derive(Debug, Default)]
pub struct DialogThread {
    state: Mutex<State>,
}

impl DialogThread {
    /// Whether the dialog's future was dropped.
    pub fn is_closed(&self) -> bool {
        self.state.lock().closed
    }

    /// Prepares the current thread to show the dialog. Returns `false` if the dialog was already
    /// closed.
    fn start(&self) -> bool {
        let mut message = MSG::default();
        // SAFETY: `message` is valid writable storage. Peeking creates the thread's message queue,
        // which `PostThreadMessageW` needs.
        _ = unsafe { PeekMessageW(&raw mut message, None, WM_USER, WM_USER, PM_NOREMOVE) };

        let mut state = self.state.lock();
        // SAFETY: no preconditions.
        state.thread_id = Some(unsafe { GetCurrentThreadId() });
        !state.closed
    }

    /// Called on the dialog's thread once it is done with the dialog. Holding the lock while
    /// clearing the thread's ID guarantees `close` never posts to it after this, when the ID could
    /// already belong to another thread.
    fn finish(&self) {
        self.state.lock().thread_id = None;
    }

    /// Closes the dialog from another thread: `WM_QUIT` ends a modal dialog's message loop, even
    /// one that has not started yet, since the message waits in the queue, and `WM_CLOSE` cancels
    /// each of the thread's windows. Dialogs can also check [`Self::is_closed`].
    fn close(&self) {
        let mut state = self.state.lock();
        state.closed = true;
        let Some(thread_id) = state.thread_id else {
            return;
        };

        // SAFETY: the thread is running, as `finish` waits for the lock held here. Failures are
        // harmless: the dialog may be closing already.
        unsafe {
            _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            _ = EnumThreadWindows(thread_id, Some(close_window), LPARAM(0));
        }
    }
}

unsafe extern "system" fn close_window(window: HWND, _: LPARAM) -> BOOL {
    // SAFETY: `window` was just enumerated. Failure is harmless: it may be closing already.
    _ = unsafe { PostMessageW(Some(window), WM_CLOSE, WPARAM(0), LPARAM(0)) };
    true.into()
}

struct CloseOnDrop(Arc<DialogThread>);

impl Drop for CloseOnDrop {
    fn drop(&mut self) {
        self.0.close();
    }
}

/// Runs `dialog` on a new thread and returns its result. Dropping the returned future closes the
/// dialog.
pub async fn run<T, F>(dialog: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(&DialogThread) -> Result<T> + Send + 'static,
{
    let shared = Arc::new(DialogThread::default());
    let (sender, receiver) = oneshot::channel();

    let thread_shared = shared.clone();
    thread::Builder::new()
        .name("dialog".to_owned())
        .spawn(move || {
            let result = if thread_shared.start() {
                dialog(&thread_shared)
            } else {
                Err(Error::Backend(
                    "the dialog was closed before it opened".to_owned(),
                ))
            };
            thread_shared.finish();
            _ = sender.send(result);
        })?;

    let _close = CloseOnDrop(shared);
    receiver
        .await
        .map_err(|_| Error::Backend("the dialog's thread ended without a result".to_owned()))?
}
