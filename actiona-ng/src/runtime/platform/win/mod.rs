use std::sync::{Arc, OnceLock};

use enigo::Direction;
use eyre::{Result, bail};
use tokio::{
    sync::{broadcast::Sender, oneshot},
    task::JoinHandle,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::WindowsAndMessaging::{
            CS_NOCLOSE, CW_USEDEFAULT, CallNextHookEx, CreateWindowExW, DefWindowProcW,
            DispatchMessageW, GetMessageW, HHOOK, MSG, MSLLHOOKSTRUCT, PostQuitMessage,
            PostThreadMessageW, RegisterClassW, SetWindowsHookExW, TranslateMessage,
            UnhookWindowsHookEx, WH_MOUSE_LL, WINDOW_EX_STYLE, WM_DESTROY, WM_DISPLAYCHANGE,
            WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_QUIT, WM_RBUTTONDOWN,
            WM_RBUTTONUP, WM_XBUTTONDOWN, WM_XBUTTONUP, WNDCLASSW, WS_POPUP, XBUTTON1, XBUTTON2,
        },
    },
    core::{Error, PCWSTR, w},
};

use crate::core::mouse::Button;

pub mod events;

#[allow(unsafe_code)]
extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_DISPLAYCHANGE => {
                let infos = display_info::DisplayInfo::all().unwrap();

                // TODO
                /*
                EVENT_SENDER
                    .get()
                    .unwrap()
                    .send(RecordEvent::DisplayChanged(infos.into()))
                    .unwrap();
                */
            }
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    LRESULT(0)
}

#[derive(Debug)]
pub struct Runtime {
    cancellation_token: CancellationToken,
    events_handle: Option<JoinHandle<Result<()>>>,
    thread_id: u32,
}

#[allow(unsafe_code)]
impl Runtime {
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Result<Arc<Self>> {
        let (thread_id_sender, thread_id_receiver) = oneshot::channel();

        let local_cancellation_token = cancellation_token.clone();

        let events_handle = task_tracker.spawn_blocking(move || {
            Self::create_message_receiver_window()?;

            let thread_id_value = unsafe { GetCurrentThreadId() };
            thread_id_sender.send(thread_id_value).unwrap();

            let mut msg = MSG::default();
            while !local_cancellation_token.is_cancelled() {
                let ret = unsafe { GetMessageW(&mut msg, None, 0, 0).0 };
                if ret == 0 {
                    break; // Exit loop when WM_QUIT is received
                }
                if ret == -1 {
                    eprintln!("Error in message loop!");
                    break;
                }

                unsafe {
                    let _ = TranslateMessage(&msg);
                }
                unsafe {
                    DispatchMessageW(&msg);
                }
            }

            Ok(())
        });

        let thread_id = thread_id_receiver.await?;

        let runtime = Arc::new(Self {
            cancellation_token: cancellation_token.clone(),
            events_handle: Some(events_handle),
            events_sender,
            thread_id,
        });

        let local_runtime = runtime.clone();

        task_tracker.spawn(async move {
            cancellation_token.cancelled().await;

            local_runtime.stop();
        });

        Ok(runtime)
    }

    fn create_message_receiver_window() -> Result<()> {
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

        Ok(())
    }

    fn stop(&self) {
        self.cancellation_token.cancel();

        unsafe {
            PostThreadMessageW(
                self.thread_id,
                WM_QUIT,
                WPARAM::default(),
                LPARAM::default(),
            )
            .unwrap();
        }
    }
}
