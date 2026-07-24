#![allow(unsafe_code)]

use std::{
    sync::{Arc, LazyLock, Weak},
    thread::{self, JoinHandle},
};

use color_eyre::Result;
use installer_tools::notification::ensure_notification_registration;
use parking_lot::Mutex;
use tokio::sync::{broadcast, oneshot};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, instrument, warn};
use windows::{
    Win32::{
        Foundation::{ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::{
            Accessibility::{HWINEVENTHOOK, SetWinEventHook},
            WindowsAndMessaging::{
                CS_NOCLOSE, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW,
                EVENT_OBJECT_DESTROY, GetMessageW, MSG, OBJID_WINDOW, PM_NOREMOVE, PeekMessageW,
                PostQuitMessage, PostThreadMessageW, RegisterClassW, TranslateMessage,
                WINDOW_EX_STYLE, WINEVENT_OUTOFCONTEXT, WM_DESTROY, WM_DISPLAYCHANGE, WM_QUIT,
                WNDCLASSW, WS_POPUP,
            },
        },
    },
    core::{Error, PCWSTR, w},
};

use crate::{
    api::displays::Displays,
    built_info, cancel_on,
    platform::win::safe_handle::{SafeWinEventHook, SafeWindowHandle},
    runtime::{
        events::Guard,
        platform::win::events::{
            WindowEvent, WindowHandle,
            input::{
                keyboard::{KeyboardInputDispatcher, KeyboardKeysTopic, KeyboardTextTopic},
                mouse::{
                    MouseButtonsTopic, MouseInputDispatcher, MouseMoveTopic, MouseScrollTopic,
                },
            },
        },
    },
};

pub mod events;

static RUNTIME: LazyLock<Mutex<Weak<Runtime>>> = LazyLock::new(|| Mutex::new(Weak::new()));

#[allow(unsafe_code)]
extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let Some(runtime) = RUNTIME.lock().upgrade() else {
        // SAFETY: forwarding preserves the window procedure parameters supplied by Windows.
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    };

    match msg {
        WM_DISPLAYCHANGE => {
            runtime.displays.refresh();
        }
        WM_DESTROY => {
            // SAFETY: PostQuitMessage takes a scalar exit code and posts to this thread's queue.
            unsafe {
                PostQuitMessage(0);
            }
        }
        _ => {
            // SAFETY: forwarding preserves the window procedure parameters supplied by Windows.
            return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
        }
    }

    LRESULT(0)
}

#[allow(unsafe_code, clippy::as_conversions)]
extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _id_event_thread: u32,
    _dwms_event_time: u32,
) {
    // Filter to top-level window destroy events only.
    if event != EVENT_OBJECT_DESTROY || id_object != OBJID_WINDOW.0 || id_child != 0 {
        return;
    }

    let Some(runtime) = RUNTIME.lock().upgrade() else {
        return;
    };

    _ = runtime
        .window_event_sender
        .send(WindowEvent::Closed(WindowHandle(hwnd.0 as isize)));
}

struct DisplayRunner {
    _window: SafeWindowHandle,
    _win_event_hook: SafeWinEventHook,
}

impl MessagePumpRunner for DisplayRunner {
    fn new() -> Result<Self> {
        // SAFETY: GetModuleHandleW takes no pointers when passed None and returns the current module.
        let instance = unsafe { GetModuleHandleW(None)? };
        let class_name = w!("MessageReceiver");

        let wnd_class = WNDCLASSW {
            style: CS_NOCLOSE,
            lpfnWndProc: Some(wnd_proc),
            hInstance: instance.into(),
            lpszClassName: class_name,
            ..Default::default()
        };

        // Register the class
        // SAFETY: `wnd_class` remains valid for this synchronous registration call.
        let atom = unsafe { RegisterClassW(&raw const wnd_class) };
        if atom == 0 {
            let err = Error::from_thread();
            if err.code() != ERROR_CLASS_ALREADY_EXISTS.to_hresult() {
                return Err(err.into());
            }
        }

        // SAFETY: all class, instance, and optional pointer arguments are valid for window creation.
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                class_name,
                PCWSTR::null(),
                WS_POPUP,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                Some(instance.into()),
                None,
            )?
        };

        if hwnd.0.is_null() {
            return Err(Error::from_thread().into());
        }

        // SAFETY: the callback has the required ABI and the requested hook uses no caller-owned data.
        let hook = unsafe {
            SetWinEventHook(
                EVENT_OBJECT_DESTROY,
                EVENT_OBJECT_DESTROY,
                None,
                Some(win_event_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        };

        Ok(Self {
            _window: SafeWindowHandle::try_new(hwnd)?,
            _win_event_hook: SafeWinEventHook::try_new(hook)?,
        })
    }

    fn on_message(&mut self, msg: &MSG) {
        // SAFETY: `msg` is supplied by the Win32 message loop and is valid for translation.
        unsafe {
            _ = TranslateMessage(msg);
        }
        // SAFETY: `msg` is supplied by the Win32 message loop and is valid for dispatch.
        unsafe {
            DispatchMessageW(msg);
        }
    }
}

#[derive(Debug)]
pub struct Runtime {
    mouse_input_dispatcher: Arc<MouseInputDispatcher>,
    keyboard_input_dispatcher: Arc<KeyboardInputDispatcher>,
    _message_pump: SafeMessagePump,
    displays: Displays,
    window_event_sender: broadcast::Sender<WindowEvent>,
}

#[allow(unsafe_code)]
impl Runtime {
    #[instrument(name = "WinRuntime::new", skip_all)]
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        displays: Displays,
    ) -> Result<Arc<Self>> {
        let message_pump = SafeMessagePump::new::<DisplayRunner>(
            "window",
            cancellation_token.clone(),
            task_tracker.clone(),
        )
        .await?;

        if let Err(error) = ensure_notification_registration(built_info::AUMID, "Actiona") {
            warn!("Could not register notification app: {error}");
        }

        let mouse_input_dispatcher =
            MouseInputDispatcher::new(cancellation_token.clone(), task_tracker.clone()).await?;
        let keyboard_input_dispatcher =
            KeyboardInputDispatcher::new(cancellation_token.clone(), task_tracker.clone()).await?;

        let (window_event_sender, _) = broadcast::channel(1024);

        Ok(Arc::new_cyclic(|me| {
            *RUNTIME.lock() = me.clone();

            Self {
                mouse_input_dispatcher,
                keyboard_input_dispatcher,
                _message_pump: message_pump,
                displays,
                window_event_sender,
            }
        }))
    }

    #[must_use]
    pub fn mouse_buttons(&self) -> Guard<MouseButtonsTopic> {
        self.mouse_input_dispatcher.subscribe_mouse_buttons()
    }

    #[must_use]
    pub fn mouse_move(&self) -> Guard<MouseMoveTopic> {
        self.mouse_input_dispatcher.subscribe_mouse_move()
    }

    #[must_use]
    pub fn mouse_scroll(&self) -> Guard<MouseScrollTopic> {
        self.mouse_input_dispatcher.subscribe_mouse_scroll()
    }

    #[must_use]
    pub fn keyboard_keys(&self) -> Guard<KeyboardKeysTopic> {
        self.keyboard_input_dispatcher.subscribe_keyboard_keys()
    }

    #[must_use]
    pub fn keyboard_text(&self) -> Guard<KeyboardTextTopic> {
        self.keyboard_input_dispatcher.subscribe_keyboard_text()
    }

    #[must_use]
    pub fn subscribe_window_events(&self) -> broadcast::Receiver<WindowEvent> {
        self.window_event_sender.subscribe()
    }
}

pub trait MessagePumpRunner {
    fn new() -> Result<Self>
    where
        Self: Sized;
    fn on_message(&mut self, msg: &MSG);
}

#[derive(Debug)]
pub struct SafeMessagePump {
    thread_id: u32,
    join_handle: Option<JoinHandle<()>>,
}

impl Drop for SafeMessagePump {
    fn drop(&mut self) {
        let thread_id = self.thread_id;
        // SAFETY: `thread_id` belongs to the message-pump thread created by this wrapper.
        unsafe {
            _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(handle) = self.join_handle.take() {
            _ = handle.join();
        } else {
            error!("SafeMessagePump join_handle was already taken");
        }
    }
}

impl SafeMessagePump {
    pub async fn new<R>(
        name: &str,
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Result<Self>
    where
        R: MessagePumpRunner + Send + 'static,
    {
        let (thread_id_sender, thread_id_receiver) = oneshot::channel();

        let join_handle = thread::Builder::new()
            .name(name.to_owned())
            .spawn(move || {
                let mut runner = match R::new() {
                    Ok(runner) => runner,
                    Err(err) => {
                        error!("failed to create message pump runner: {err}");
                        _ = thread_id_sender.send(0);
                        return;
                    }
                };

                // SAFETY: GetCurrentThreadId takes no pointers and returns this thread's identifier.
                let thread_id = unsafe { GetCurrentThreadId() };
                // Force creation of the thread's message queue before we report the thread ID.
                // Without this, a fast shutdown can race with the first GetMessageW call:
                // PostThreadMessageW(WM_QUIT) fails because the queue does not exist yet, and
                // the subsequent join blocks forever waiting for a thread that will never wake.
                let mut msg = MSG::default();
                // SAFETY: `msg` is valid writable storage used only to initialize this thread's queue.
                unsafe {
                    _ = PeekMessageW(&raw mut msg, None, 0, 0, PM_NOREMOVE);
                }
                _ = thread_id_sender.send(thread_id);

                loop {
                    // SAFETY: `msg` is valid writable storage for the next thread message.
                    let message = unsafe { GetMessageW(&raw mut msg, None, 0, 0).0 };
                    if message == 0 {
                        break; // Exit loop when WM_QUIT is received
                    }
                    if message == -1 {
                        error!("Error in message loop");
                        break;
                    }

                    runner.on_message(&msg);
                }
            })?;

        let thread_id = cancel_on(&cancellation_token, thread_id_receiver).await??;

        task_tracker.spawn(async move {
            cancellation_token.cancelled().await;
            // SAFETY: `thread_id` belongs to the message-pump thread created above.
            unsafe {
                _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        });

        Ok(Self {
            thread_id,
            join_handle: Some(join_handle),
        })
    }

    pub fn send_message(&self, message: u32) {
        // SAFETY: `self.thread_id` belongs to the message-pump thread created by this wrapper.
        unsafe {
            _ = PostThreadMessageW(self.thread_id, message, WPARAM(0), LPARAM(0));
        }
    }
}
