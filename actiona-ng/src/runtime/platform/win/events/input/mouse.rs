use std::sync::{
    Arc, Weak,
    atomic::{AtomicUsize, Ordering},
};

use enigo::Direction;
use eyre::Result;
use once_cell::sync::OnceCell;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        CallNextHookEx, HC_ACTION, HOOKPROC, LLMHF_INJECTED, MSLLHOOKSTRUCT, WH_MOUSE_LL,
        WINDOWS_HOOK_ID, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
        WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN,
        WM_XBUTTONUP, XBUTTON1, XBUTTON2,
    },
};

use crate::{
    core::{mouse::Button, point::point},
    runtime::{
        events::{
            AllSignals, Guard, LatestOnlySignals, MouseButtonEvent, MouseMoveEvent, Topic,
            TopicWrapper,
        },
        platform::win::{
            SafeMessagePump,
            events::input::{HookSpec, LowLevelHookRunner, MSG_START, MSG_STOP},
        },
    },
};

static MOUSE_INPUT_DISPATCHER: OnceCell<Weak<MouseInputDispatcher>> = OnceCell::new();

const fn get_xbutton_wparam(mouse_data: u32) -> u16 {
    ((mouse_data >> 16) & 0xFFFF) as u16
}

#[derive(Default)]
pub struct MouseHook {}

impl HookSpec for MouseHook {
    const ID: WINDOWS_HOOK_ID = WH_MOUSE_LL;

    fn proc() -> HOOKPROC {
        Some(low_level_mouse_proc)
    }
}

#[derive(Debug)]
pub struct MouseInputDispatcher {
    mouse_buttons: Arc<TopicWrapper<MouseButtonsTopic>>,
    mouse_move: Arc<TopicWrapper<MouseMoveTopic>>,
    subscribers: Arc<AtomicUsize>,
    message_pump: SafeMessagePump,
}

impl MouseInputDispatcher {
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Result<Arc<Self>> {
        let message_pump = SafeMessagePump::new::<LowLevelHookRunner<MouseHook>>(
            "input_dispatcher",
            cancellation_token.clone(),
            task_tracker.clone(),
        )
        .await?;

        Ok(Arc::new_cyclic(|me| {
            if MOUSE_INPUT_DISPATCHER.set(me.clone()).is_err() {
                panic!("InputDispatcher should only be intantiated once");
            }

            Self {
                mouse_buttons: Arc::new(TopicWrapper::new(
                    MouseButtonsTopic {
                        dispatcher: me.clone(),
                    },
                    cancellation_token.clone(),
                    task_tracker.clone(),
                )),
                mouse_move: Arc::new(TopicWrapper::new(
                    MouseMoveTopic {
                        dispatcher: me.clone(),
                    },
                    cancellation_token.clone(),
                    task_tracker.clone(),
                )),
                subscribers: Arc::new(AtomicUsize::new(0)),
                message_pump,
            }
        }))
    }

    #[must_use]
    pub fn subscribe_mouse_buttons(&self) -> Guard<MouseButtonsTopic> {
        self.mouse_buttons.subscribe()
    }

    #[must_use]
    pub fn subscribe_mouse_move(&self) -> Guard<MouseMoveTopic> {
        self.mouse_move.subscribe()
    }

    pub fn publish_mouse_buttons(&self, value: <MouseButtonsTopic as Topic>::T) {
        self.mouse_buttons.publish(value);
    }

    pub fn publish_mouse_move(&self, value: <MouseMoveTopic as Topic>::T) {
        self.mouse_move.publish(value);
    }

    async fn on_start(&self) {
        if self.subscribers.fetch_add(1, Ordering::Relaxed) == 0 {
            self.message_pump.send_message(MSG_START);
        }
    }

    async fn on_stop(&self) {
        if self.subscribers.fetch_sub(1, Ordering::Relaxed) == 1 {
            self.message_pump.send_message(MSG_STOP);
        }
    }
}

#[derive(Debug, Default)]
pub struct MouseButtonsTopic {
    dispatcher: Weak<MouseInputDispatcher>,
}

impl Topic for MouseButtonsTopic {
    type T = MouseButtonEvent;
    type Signal = AllSignals<Self::T>;

    async fn on_start(&self) {
        if let Some(dispatcher) = self.dispatcher.upgrade() {
            dispatcher.on_start().await;
        }
    }

    async fn on_stop(&self) {
        if let Some(dispatcher) = self.dispatcher.upgrade() {
            dispatcher.on_stop().await;
        }
    }
}

#[derive(Debug, Default)]
pub struct MouseMoveTopic {
    dispatcher: Weak<MouseInputDispatcher>,
}

impl Topic for MouseMoveTopic {
    type T = MouseMoveEvent;
    type Signal = LatestOnlySignals<Self::T>;

    async fn on_start(&self) {
        if let Some(dispatcher) = self.dispatcher.upgrade() {
            dispatcher.on_start().await;
        }
    }

    async fn on_stop(&self) {
        if let Some(dispatcher) = self.dispatcher.upgrade() {
            dispatcher.on_stop().await;
        }
    }
}

unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code != HC_ACTION as i32 {
        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
    }

    let Some(dispatcher) = MOUSE_INPUT_DISPATCHER
        .get()
        .and_then(|dispatcher| dispatcher.upgrade())
    else {
        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
    };

    let mouse_buttons = &dispatcher.mouse_buttons;
    let mouse_move = &dispatcher.mouse_move;

    let mouse_struct = unsafe { *(l_param.0 as *const MSLLHOOKSTRUCT) };
    let injected = mouse_struct.flags & LLMHF_INJECTED == LLMHF_INJECTED;

    match w_param.0 as u32 {
        WM_LBUTTONDOWN => mouse_buttons.publish(MouseButtonEvent::new(
            Button::Left,
            Direction::Press,
            injected,
        )),
        WM_RBUTTONDOWN => mouse_buttons.publish(MouseButtonEvent::new(
            Button::Right,
            Direction::Press,
            injected,
        )),
        WM_MBUTTONDOWN => mouse_buttons.publish(MouseButtonEvent::new(
            Button::Middle,
            Direction::Press,
            injected,
        )),
        WM_XBUTTONDOWN => match get_xbutton_wparam(mouse_struct.mouseData) {
            XBUTTON1 => mouse_buttons.publish(MouseButtonEvent::new(
                Button::Back,
                Direction::Press,
                injected,
            )),
            XBUTTON2 => mouse_buttons.publish(MouseButtonEvent::new(
                Button::Forward,
                Direction::Press,
                injected,
            )),
            _ => {}
        },
        WM_LBUTTONUP => mouse_buttons.publish(MouseButtonEvent::new(
            Button::Left,
            Direction::Release,
            injected,
        )),
        WM_RBUTTONUP => mouse_buttons.publish(MouseButtonEvent::new(
            Button::Right,
            Direction::Release,
            injected,
        )),
        WM_MBUTTONUP => mouse_buttons.publish(MouseButtonEvent::new(
            Button::Middle,
            Direction::Release,
            injected,
        )),
        WM_XBUTTONUP => match get_xbutton_wparam(mouse_struct.mouseData) {
            XBUTTON1 => mouse_buttons.publish(MouseButtonEvent::new(
                Button::Back,
                Direction::Release,
                injected,
            )),
            XBUTTON2 => mouse_buttons.publish(MouseButtonEvent::new(
                Button::Forward,
                Direction::Release,
                injected,
            )),
            _ => {}
        },
        WM_MOUSEMOVE => {
            mouse_move.publish(MouseMoveEvent::new(
                point(mouse_struct.pt.x, mouse_struct.pt.y),
                injected,
            ));
        }
        WM_MOUSEWHEEL => {
            // TODO
        }
        WM_MOUSEHWHEEL => {
            // TODO
        }
        _ => {}
    };

    unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
}
