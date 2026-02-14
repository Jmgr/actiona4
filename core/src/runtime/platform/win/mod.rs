#![allow(unsafe_code)]

use std::{
    sync::{Arc, Mutex, Weak},
    thread::{self, JoinHandle},
};

use color_eyre::Result;
use once_cell::sync::Lazy;
use tokio::sync::oneshot;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, instrument};
use windows::{
    Win32::{
        Foundation::{ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::WindowsAndMessaging::{
            CS_NOCLOSE, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW,
            GetMessageW, MSG, PostQuitMessage, PostThreadMessageW, RegisterClassW,
            TranslateMessage, WINDOW_EX_STYLE, WM_DESTROY, WM_DISPLAYCHANGE, WM_QUIT, WNDCLASSW,
            WS_POPUP,
        },
    },
    core::{Error, PCWSTR, w},
};

use crate::{
    api::displays::Displays,
    cancel_on,
    platform::win::safe_handle::SafeWindowHandle,
    runtime::{
        events::Guard,
        platform::win::{
            events::input::{
                keyboard::{KeyboardInputDispatcher, KeyboardKeysTopic, KeyboardTextTopic},
                mouse::{MouseButtonsTopic, MouseInputDispatcher, MouseMoveTopic},
            },
            notification::ensure_notification_registration,
        },
    },
};

pub mod events;
mod notification;

static RUNTIME: Lazy<Mutex<Weak<Runtime>>> = Lazy::new(|| Mutex::new(Weak::new()));

#[allow(unsafe_code)]
extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let Some(runtime) = RUNTIME.lock().unwrap_or_else(|e| e.into_inner()).upgrade() else {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    };

    match msg {
        WM_DISPLAYCHANGE => {
            runtime.displays.refresh();
        }
        WM_DESTROY => unsafe {
            PostQuitMessage(0);
        },
        _ => return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }

    LRESULT(0)
}

struct DisplayRunner {
    _window: SafeWindowHandle,
}

impl MessagePumpRunner for DisplayRunner {
    fn new() -> Result<Self> {
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
        let atom = unsafe { RegisterClassW(&wnd_class) };
        if atom == 0 {
            let err = Error::from_thread();
            if err.code() != ERROR_CLASS_ALREADY_EXISTS.to_hresult() {
                return Err(err.into());
            }
        }

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

        Ok(Self {
            _window: SafeWindowHandle::try_new(hwnd)?,
        })
    }

    fn on_message(&mut self, msg: &MSG) {
        unsafe {
            _ = TranslateMessage(msg);
        }
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

        ensure_notification_registration();

        let mouse_input_dispatcher =
            MouseInputDispatcher::new(cancellation_token.clone(), task_tracker.clone()).await?;
        let keyboard_input_dispatcher =
            KeyboardInputDispatcher::new(cancellation_token.clone(), task_tracker.clone()).await?;

        Ok(Arc::new_cyclic(|me| {
            *RUNTIME.lock().unwrap_or_else(|e| e.into_inner()) = me.clone();

            Self {
                mouse_input_dispatcher,
                keyboard_input_dispatcher,
                _message_pump: message_pump,
                displays,
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
    pub fn keyboard_keys(&self) -> Guard<KeyboardKeysTopic> {
        self.keyboard_input_dispatcher.subscribe_keyboard_keys()
    }

    #[must_use]
    pub fn keyboard_text(&self) -> Guard<KeyboardTextTopic> {
        self.keyboard_input_dispatcher.subscribe_keyboard_text()
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
        unsafe {
            _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        _ = self.join_handle.take().unwrap().join();
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
            .name(name.to_string())
            .spawn(move || {
                let mut runner = R::new().unwrap(); // TODO

                let thread_id = unsafe { GetCurrentThreadId() };
                _ = thread_id_sender.send(thread_id);

                let mut msg = MSG::default();
                loop {
                    let message = unsafe { GetMessageW(&mut msg, None, 0, 0).0 };
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
        unsafe {
            _ = PostThreadMessageW(self.thread_id, message, WPARAM(0), LPARAM(0));
        }
    }
}
