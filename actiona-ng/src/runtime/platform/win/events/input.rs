#![allow(unsafe_code)]

use std::{
    cell::RefCell,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use tokio::select;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, HHOOK, MSLLHOOKSTRUCT, SetWindowsHookExW, UnhookWindowsHookEx, WH_MOUSE_LL,
        WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEMOVE,
        WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN, WM_XBUTTONUP, XBUTTON1,
        XBUTTON2,
    },
};

use crate::{
    core::point::Point,
    runtime::events::{
        AllSignals, Control, LatestOnlySignals, MouseButtonEvent, ReceiverGuard, Topic,
    },
};

thread_local! {
    static HOOK_DISPATCH: RefCell<Option<Arc<>>> = const { RefCell::new(None) };
}

struct SafeHook(HHOOK);

#[allow(unsafe_code)]
impl Drop for SafeHook {
    fn drop(&mut self) {
        unsafe {
            if let Err(err) = UnhookWindowsHookEx(self.0) {
                error!("UnhookWindowsHookEx failed: {err}");
            }
        }
    }
}

const fn get_xbutton_wparam(mouse_data: u32) -> u16 {
    ((mouse_data >> 16) & 0xFFFF) as u16
}

unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code < 0 {
        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
    }

    let mouse_struct = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };

    let event = match w_param.0 as u32 {
        WM_LBUTTONDOWN => Some((Button::Left, Direction::Press)),
        WM_RBUTTONDOWN => Some((Button::Right, Direction::Press)),
        WM_MBUTTONDOWN => Some((Button::Middle, Direction::Press)),
        WM_XBUTTONDOWN => match get_xbutton_wparam(mouse_struct.mouseData) {
            XBUTTON1 => Some((Button::Back, Direction::Press)),
            XBUTTON2 => Some((Button::Forward, Direction::Press)),
            _ => None,
        },
        WM_LBUTTONUP => Some((Button::Left, Direction::Release)),
        WM_RBUTTONUP => Some((Button::Right, Direction::Release)),
        WM_MBUTTONUP => Some((Button::Middle, Direction::Release)),
        WM_XBUTTONUP => match get_xbutton_wparam(mouse_struct.mouseData) {
            XBUTTON1 => Some((Button::Back, Direction::Release)),
            XBUTTON2 => Some((Button::Forward, Direction::Release)),
            _ => None,
        },
        WM_MOUSEMOVE => {
            // TODO
        }
        WM_MOUSEWHEEL => {
            // TODO
        }
        WM_MOUSEHWHEEL => {
            // TODO
        }
        _ => None,
    };

    if let Some(button_event) = event {
        let _ = EVENT_SENDER
            .get()
            .unwrap()
            .send(RecordEvent::MouseButton(button_event.0, button_event.1));
    }

    unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
}

pub struct InputDispatcher {
    hook_handle: Option<SafeHook>,
    subscriptions: Arc<AtomicUsize>,
    mouse_buttons_topic: Arc<Topic<MouseButtonEvent, AllSignals<MouseButtonEvent>>>,
    mouse_move_topic: Arc<Topic<Point, LatestOnlySignals<Point>>>,
}

impl InputDispatcher {
    pub fn new(task_tracker: TaskTracker, cancellation_token: CancellationToken) -> Self {
        let (mouse_buttons_topic, mut mouse_buttons_control_receiver) =
            Topic::new(AllSignals::new());
        let (mouse_move_topic, mut mouse_move_control_receiver) =
            Topic::new(LatestOnlySignals::new());

        task_tracker.spawn(async move {
            loop {
                select! {
                    _ = cancellation_token.cancelled() => { break; }
                    changed = mouse_buttons_control_receiver.changed() => {
                        if changed.is_err() { break; } // sender dropped
                    }
                                        changed = mouse_move_control_receiver.changed() => {
                        if changed.is_err() { break; } // sender dropped
                    }
                }

                let control = *control_receiver.borrow_and_update();

                info!("{name}: {}", control);

                match control {
                    Control::Enable => {
                        if activation_counter.fetch_add(1, Ordering::Relaxed) == 0 {
                            // Enable
                            let hook = unsafe {
                                SetWindowsHookExW(WH_MOUSE_LL, Some(low_level_mouse_proc), None, 0)
                                    .unwrap();
                            };

                            if hook.is_invalid() {
                                error!("invalid Windows hook");
                                continue;
                            }

                            // Ok(SafeHook(hook))
                        }
                    }
                    Control::Disable => {
                        if activation_counter.fetch_sub(1, Ordering::Relaxed) == 1 {
                            // TODO: what if already 0?
                            // Disable
                            unsafe { UnhookWindowsHookEx(self.0).unwrap() }
                        }
                    }
                }
            }
        });

        Self {
            hook_handle: None,
            subscriptions: Arc::new(AtomicUsize::new(0)),
            mouse_buttons_topic: Arc::new(mouse_button_topic),
            mouse_move_topic: Arc::new(mouse_move_topic),
        }
    }
}

#[derive(Debug)]
pub struct MouseButtonsTopic {
    topic: Topic<MouseButtonEvent, AllSignals<MouseButtonEvent>>,
}

impl MouseButtonsTopic {
    pub fn new(
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
        activation_counter: Arc<AtomicUsize>,
    ) -> Self {
        let (topic, mut control_receiver) = Topic::new(AllSignals::new());
        //
        task_tracker.spawn(async move {
            loop {
                select! {
                    _ = cancellation_token.cancelled() => { break; }
                    changed = control_receiver.changed() => {
                        if changed.is_err() { break; } // sender dropped
                    }
                }

                let control = *control_receiver.borrow_and_update();

                info!("{name}: {}", control);

                match control {
                    Control::Enable => {
                        if activation_counter.fetch_add(1, Ordering::Relaxed) == 0 {
                            // Enable
                            let hook = unsafe {
                                SetWindowsHookExW(WH_MOUSE_LL, Some(low_level_mouse_proc), None, 0)
                                    .unwrap();
                            };

                            if hook.is_invalid() {
                                error!("invalid Windows hook");
                                continue;
                            }

                            // Ok(SafeHook(hook))
                        }
                    }
                    Control::Disable => {
                        if activation_counter.fetch_sub(1, Ordering::Relaxed) == 1 {
                            // TODO: what if already 0?
                            // Disable
                            unsafe { UnhookWindowsHookEx(self.0).unwrap() }
                        }
                    }
                }
            }
        });
        Self {
            topic,
            activation_counter,
        }
    }

    pub fn publish(&self, value: MouseButtonEvent) {
        self.topic.publish(value);
    }

    pub fn subscribe(&self) -> ReceiverGuard<MouseButtonEvent, AllSignals<MouseButtonEvent>> {
        self.topic.subscribe()
    }
}

#[derive(Debug)]
pub struct MouseMoveTopic {
    topic: Topic<Point, LatestOnlySignals<Point>>,
    activation_counter: Arc<AtomicUsize>,
}

impl MouseMoveTopic {
    pub fn new(
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
        activation_counter: Arc<AtomicUsize>,
    ) -> Self {
        let (topic, control_receiver) = Topic::new(LatestOnlySignals::new());
        //
        Self {
            topic,
            activation_counter,
        }
    }

    pub fn publish(&self, value: Point) {
        self.topic.publish(value);
    }

    pub fn subscribe(&self) -> ReceiverGuard<Point, LatestOnlySignals<Point>> {
        self.topic.subscribe()
    }
}
