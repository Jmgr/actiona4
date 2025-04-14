use std::sync::{Arc, OnceLock};

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

use super::RecordEvent;
use crate::{core::mouse::Button, runtime::Direction};

static EVENT_SENDER: OnceLock<Sender<RecordEvent>> = OnceLock::new();

const fn get_xbutton_wparam(mouse_data: u32) -> u16 {
    ((mouse_data >> 16) & 0xFFFF) as u16
}

unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let mouse_struct = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };

        let button_event = match w_param.0 as u32 {
            WM_LBUTTONDOWN => Some((Button::Left, Direction::Pressed)),
            WM_RBUTTONDOWN => Some((Button::Right, Direction::Pressed)),
            WM_MBUTTONDOWN => Some((Button::Middle, Direction::Pressed)),
            WM_XBUTTONDOWN => match get_xbutton_wparam(mouse_struct.mouseData) {
                XBUTTON1 => Some((Button::Back, Direction::Pressed)),
                XBUTTON2 => Some((Button::Forward, Direction::Pressed)),
                _ => None,
            },
            WM_LBUTTONUP => Some((Button::Left, Direction::Released)),
            WM_RBUTTONUP => Some((Button::Right, Direction::Released)),
            WM_MBUTTONUP => Some((Button::Middle, Direction::Released)),
            WM_XBUTTONUP => match get_xbutton_wparam(mouse_struct.mouseData) {
                XBUTTON1 => Some((Button::Back, Direction::Released)),
                XBUTTON2 => Some((Button::Forward, Direction::Released)),
                _ => None,
            },
            _ => None,
        };

        if let Some(button_event) = button_event {
            let _ = EVENT_SENDER
                .get()
                .unwrap()
                .send(RecordEvent::MouseButton(button_event.0, button_event.1));
        }
    }

    unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
}

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match msg {
            WM_DISPLAYCHANGE => {
                let infos = display_info::DisplayInfo::all().unwrap();

                EVENT_SENDER
                    .get()
                    .unwrap()
                    .send(RecordEvent::DisplayChanged(infos.into()))
                    .unwrap();
            }
            WM_DESTROY => {
                PostQuitMessage(0);
            }
            _ => return DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    LRESULT(0)
}

struct SafeHook(HHOOK);

impl Drop for SafeHook {
    fn drop(&mut self) {
        unsafe { UnhookWindowsHookEx(self.0).unwrap() }
    }
}

#[derive(Debug)]
pub struct Runtime {
    cancellation_token: CancellationToken,
    events_handle: Option<JoinHandle<Result<()>>>,
    events_sender: Sender<RecordEvent>,
    thread_id: u32,
}

impl Runtime {
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
        events_sender: Sender<RecordEvent>,
    ) -> Result<Arc<Self>> {
        let (thread_id_sender, thread_id_receiver) = oneshot::channel();

        EVENT_SENDER.set(events_sender.clone()).unwrap();

        let local_cancellation_token = cancellation_token.clone();

        let events_handle = task_tracker.spawn_blocking(move || {
            Self::create_message_receiver_window()?;

            let thread_id_value = unsafe { GetCurrentThreadId() };
            thread_id_sender.send(thread_id_value).unwrap();

            let _hook = Self::setup_hook()?;

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
            return Err(Error::from_win32().into());
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
            return Err(Error::from_win32().into());
        }

        Ok(())
    }

    fn setup_hook() -> Result<SafeHook> {
        let hook = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(low_level_mouse_proc), None, 0)? };

        if hook.is_invalid() {
            bail!("invalid Windows hook");
        }

        Ok(SafeHook(hook))
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
