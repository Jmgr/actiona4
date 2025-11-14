#![allow(unsafe_code)]

use std::{
    sync::{Arc, Weak},
    thread::{self, JoinHandle},
};

use color_eyre::Result;
use once_cell::sync::OnceCell;
use tokio::{
    select,
    sync::{broadcast, oneshot},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::error;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
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
    error::CommonError,
    platform::win::safe_handle::SafeWindowHandle,
    runtime::{
        events::{DisplayInfoVec, Guard, TopicWrapper},
        platform::win::events::input::{
            keyboard::{KeyboardInputDispatcher, KeyboardKeysTopic, KeyboardTextTopic},
            mouse::{MouseButtonsTopic, MouseInputDispatcher, MouseMoveTopic},
        },
    },
};

pub mod events;

static RUNTIME: OnceCell<Weak<Runtime>> = OnceCell::new();

#[allow(unsafe_code)]
extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let Some(runtime) = RUNTIME.get().and_then(|runtime| runtime.upgrade()) else {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    };

    match msg {
        WM_DISPLAYCHANGE => {
            let infos = display_info::DisplayInfo::all().unwrap();

            let _ = runtime.screen_change_sender.send(infos.into());
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
            return Err(Error::from_thread().into());
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
            let _ = TranslateMessage(msg);
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
    screen_change_sender: broadcast::Sender<DisplayInfoVec>,
}

#[allow(unsafe_code)]
impl Runtime {
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Result<Arc<Self>> {
        let message_pump = SafeMessagePump::new::<DisplayRunner>(
            "window",
            cancellation_token.clone(),
            task_tracker.clone(),
        )
        .await?;

        let mouse_input_dispatcher =
            MouseInputDispatcher::new(cancellation_token.clone(), task_tracker.clone()).await?;
        let keyboard_input_dispatcher =
            KeyboardInputDispatcher::new(cancellation_token.clone(), task_tracker.clone()).await?;

        let (screen_change_sender, _) = broadcast::channel(1024); // TODO

        Ok(Arc::new_cyclic(|me| {
            if RUNTIME.set(me.clone()).is_err() {
                panic!("Runtime should only be instantiated once");
            }

            Self {
                mouse_input_dispatcher,
                keyboard_input_dispatcher,
                _message_pump: message_pump,
                screen_change_sender,
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

    #[must_use]
    pub fn subscribe_screen_change(&self) -> broadcast::Receiver<DisplayInfoVec> {
        self.screen_change_sender.subscribe()
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
            let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        let _ = self.join_handle.take().unwrap().join();
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
                let _ = thread_id_sender.send(thread_id);

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

        let thread_id = select! {
            _ = cancellation_token.cancelled() => { return Err(CommonError::Cancelled.into()); }
            thread_id = thread_id_receiver => { thread_id }
        }?;

        task_tracker.spawn(async move {
            cancellation_token.cancelled().await;
            unsafe {
                let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        });

        Ok(Self {
            thread_id,
            join_handle: Some(join_handle),
        })
    }

    pub fn send_message(&self, message: u32) {
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, message, WPARAM(0), LPARAM(0));
        }
    }
}
