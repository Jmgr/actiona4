use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use color_eyre::Result;
use derive_more::Constructor;
use enigo::Key;
use parking_lot::Mutex;
use x11rb::protocol::xinput::{Device, EventMask, XIEventMask};
use x11rb_async::{connection::Connection, protocol::xinput::xi_select_events};
use xkeysym::Keysym;

use crate::{
    api::{mouse::Axis, point::Point},
    platform::x11::X11Connection,
    runtime::events::{
        AllSignals, KeyboardKeyEvent, KeyboardTextEvent, LatestOnlySignals, MouseButtonEvent,
        MouseScrollEvent, Topic,
    },
};

#[derive(Clone, Debug)]
pub struct InputMask {
    x11_connection: Arc<X11Connection>,
    inner: Arc<Mutex<XIEventMask>>,
}

impl InputMask {
    pub async fn new(x11_connection: Arc<X11Connection>) -> Result<Self> {
        let result = Self {
            x11_connection,
            inner: Arc::new(Mutex::new(XIEventMask::default())),
        };

        Ok(result)
    }

    pub async fn set(&self, mask: XIEventMask) -> Result<()> {
        {
            let mut inner = self.inner.lock();
            *inner |= mask;
        }

        self.apply().await
    }

    pub async fn remove(&self, mask: XIEventMask) -> Result<()> {
        {
            let mut inner = self.inner.lock();
            *inner = inner.remove(mask);
        }

        self.apply().await
    }

    #[must_use]
    fn to_vec(&self) -> Vec<XIEventMask> {
        let inner = self.inner.lock();
        vec![*inner]
    }

    async fn apply(&self) -> Result<()> {
        let connection = self.x11_connection.async_connection();
        let mask = self.to_vec();
        let masks = &[EventMask {
            deviceid: Device::ALL_MASTER.into(),
            mask,
        }];
        let root_window = self.x11_connection.screen().root;

        xi_select_events(connection, root_window, masks).await?;

        connection.flush().await?;

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct ActivationCounter(Arc<AtomicUsize>);

impl ActivationCounter {
    fn increment(&self) -> usize {
        self.0.fetch_add(1, Ordering::Relaxed)
    }

    fn decrement(&self) -> usize {
        self.0.fetch_sub(1, Ordering::Relaxed)
    }
}

fn mouse_buttons_events() -> XIEventMask {
    XIEventMask::RAW_BUTTON_PRESS | XIEventMask::RAW_BUTTON_RELEASE
}

async fn mouse_buttons_start(input_mask: &InputMask) -> Result<()> {
    input_mask.set(mouse_buttons_events()).await
}

async fn mouse_buttons_stop(input_mask: &InputMask) -> Result<()> {
    input_mask.remove(mouse_buttons_events()).await
}

#[derive(Constructor, Debug)]
pub struct MouseButtonsTopic {
    input_mask: InputMask,
    /// Shared with `MouseScrollTopic` — both use the same `RAW_BUTTON_PRESS` mask.
    activation_counter: ActivationCounter,
}

impl Topic for MouseButtonsTopic {
    type T = MouseButtonEvent;
    type Signal = AllSignals<Self::T>;

    async fn on_start(&self) -> Result<()> {
        if self.activation_counter.increment() == 0
            && let Err(error) = mouse_buttons_start(&self.input_mask).await
        {
            _ = self.activation_counter.decrement();
            return Err(error);
        }
        Ok(())
    }

    async fn on_stop(&self) -> Result<()> {
        if self.activation_counter.decrement() == 1
            && let Err(error) = mouse_buttons_stop(&self.input_mask).await
        {
            _ = self.activation_counter.increment();
            return Err(error);
        }
        Ok(())
    }
}

/// On X11 scroll events arrive as raw button presses (buttons 4–7), so this topic shares the
/// same `XIEventMask` and `ActivationCounter` as `MouseButtonsTopic`.
#[derive(Constructor, Debug)]
pub struct MouseScrollTopic {
    input_mask: InputMask,
    activation_counter: ActivationCounter,
}

impl Topic for MouseScrollTopic {
    type T = MouseScrollEvent;
    type Signal = AllSignals<Self::T>;

    async fn on_start(&self) -> Result<()> {
        if self.activation_counter.increment() == 0
            && let Err(error) = mouse_buttons_start(&self.input_mask).await
        {
            _ = self.activation_counter.decrement();
            return Err(error);
        }
        Ok(())
    }

    async fn on_stop(&self) -> Result<()> {
        if self.activation_counter.decrement() == 1
            && let Err(error) = mouse_buttons_stop(&self.input_mask).await
        {
            _ = self.activation_counter.increment();
            return Err(error);
        }
        Ok(())
    }
}

/// Convert an X11 raw button detail to a `MouseScrollEvent`, if it is a scroll button (4–7).
#[must_use]
pub const fn scroll_event_from_x11_button(detail: u32, is_injected: bool) -> Option<MouseScrollEvent> {
    match detail {
        4 => Some(MouseScrollEvent::new(Axis::Vertical, -1, is_injected)),
        5 => Some(MouseScrollEvent::new(Axis::Vertical, 1, is_injected)),
        6 => Some(MouseScrollEvent::new(Axis::Horizontal, -1, is_injected)),
        7 => Some(MouseScrollEvent::new(Axis::Horizontal, 1, is_injected)),
        _ => None,
    }
}

#[derive(Constructor, Debug)]
pub struct MouseMoveTopic {
    input_mask: InputMask,
}

const fn mouse_motion_events() -> XIEventMask {
    XIEventMask::MOTION
}

impl Topic for MouseMoveTopic {
    type T = Point;
    type Signal = LatestOnlySignals<Self::T>;

    async fn on_start(&self) -> Result<()> {
        self.input_mask.set(mouse_motion_events()).await
    }

    async fn on_stop(&self) -> Result<()> {
        self.input_mask.remove(mouse_motion_events()).await
    }
}

#[derive(Constructor, Debug)]
pub struct KeyboardKeysTopic {
    input_mask: InputMask,
    activation_counter: ActivationCounter, // TODO: use dispatcher, like Windows
}

fn keyboard_keys_events() -> XIEventMask {
    XIEventMask::RAW_KEY_PRESS | XIEventMask::RAW_KEY_RELEASE
}

async fn keyboard_start(input_mask: &InputMask) -> Result<()> {
    input_mask.set(keyboard_keys_events()).await
}

async fn keyboard_stop(input_mask: &InputMask) -> Result<()> {
    input_mask.remove(keyboard_keys_events()).await
}

impl Topic for KeyboardKeysTopic {
    type T = KeyboardKeyEvent;
    type Signal = AllSignals<Self::T>;

    async fn on_start(&self) -> Result<()> {
        if self.activation_counter.increment() == 0
            && let Err(error) = keyboard_start(&self.input_mask).await
        {
            _ = self.activation_counter.decrement();
            return Err(error);
        }
        Ok(())
    }

    async fn on_stop(&self) -> Result<()> {
        if self.activation_counter.decrement() == 1
            && let Err(error) = keyboard_stop(&self.input_mask).await
        {
            _ = self.activation_counter.increment();
            return Err(error);
        }
        Ok(())
    }
}

#[derive(Constructor, Debug)]
pub struct KeyboardTextTopic {
    input_mask: InputMask,
    activation_counter: ActivationCounter,
}

impl Topic for KeyboardTextTopic {
    type T = KeyboardTextEvent;
    type Signal = AllSignals<Self::T>;

    async fn on_start(&self) -> Result<()> {
        if self.activation_counter.increment() == 0
            && let Err(error) = keyboard_start(&self.input_mask).await
        {
            _ = self.activation_counter.decrement();
            return Err(error);
        }
        Ok(())
    }

    async fn on_stop(&self) -> Result<()> {
        if self.activation_counter.decrement() == 1
            && let Err(error) = keyboard_stop(&self.input_mask).await
        {
            _ = self.activation_counter.increment();
            return Err(error);
        }
        Ok(())
    }
}

pub fn keysym_to_key(keysym: Keysym) -> Key {
    match keysym {
        Keysym::KP_Add => Key::Add,
        Keysym::Alt_L | Keysym::Alt_R => Key::Alt,
        Keysym::BackSpace => Key::Backspace,
        Keysym::Begin => Key::Begin,
        Keysym::Break => Key::Break,
        Keysym::Cancel => Key::Cancel,
        Keysym::Caps_Lock => Key::CapsLock,
        Keysym::Clear => Key::Clear,
        Keysym::Control_L => Key::LControl,
        Keysym::KP_Decimal => Key::Decimal,
        Keysym::Delete => Key::Delete,
        Keysym::KP_Divide => Key::Divide,
        Keysym::Down => Key::DownArrow,
        Keysym::End => Key::End,
        Keysym::KP_Enter => Key::NumpadEnter,
        Keysym::Escape => Key::Escape,
        Keysym::Execute => Key::Execute,
        Keysym::F1 => Key::F1,
        Keysym::F2 => Key::F2,
        Keysym::F3 => Key::F3,
        Keysym::F4 => Key::F4,
        Keysym::F5 => Key::F5,
        Keysym::F6 => Key::F6,
        Keysym::F7 => Key::F7,
        Keysym::F8 => Key::F8,
        Keysym::F9 => Key::F9,
        Keysym::F10 => Key::F10,
        Keysym::F11 => Key::F11,
        Keysym::F12 => Key::F12,
        Keysym::F13 => Key::F13,
        Keysym::F14 => Key::F14,
        Keysym::F15 => Key::F15,
        Keysym::F16 => Key::F16,
        Keysym::F17 => Key::F17,
        Keysym::F18 => Key::F18,
        Keysym::F19 => Key::F19,
        Keysym::F20 => Key::F20,
        Keysym::F21 => Key::F21,
        Keysym::F22 => Key::F22,
        Keysym::F23 => Key::F23,
        Keysym::F24 => Key::F24,
        Keysym::F25 => Key::F25,
        Keysym::F26 => Key::F26,
        Keysym::F27 => Key::F27,
        Keysym::F28 => Key::F28,
        Keysym::F29 => Key::F29,
        Keysym::F30 => Key::F30,
        Keysym::F31 => Key::F31,
        Keysym::F32 => Key::F32,
        Keysym::F33 => Key::F33,
        Keysym::F34 => Key::F34,
        Keysym::F35 => Key::F35,
        Keysym::Find => Key::Find,
        Keysym::Hangul => Key::Hangul,
        Keysym::Hangul_Hanja => Key::Hanja,
        Keysym::Help => Key::Help,
        Keysym::Home => Key::Home,
        Keysym::Insert => Key::Insert,
        Keysym::Kanji => Key::Kanji,
        Keysym::Left => Key::LeftArrow,
        Keysym::Linefeed => Key::Linefeed,
        Keysym::Menu => Key::LMenu,
        Keysym::Mode_switch => Key::ModeChange,
        Keysym::KP_Multiply => Key::Multiply,
        Keysym::XF86_AudioNext => Key::MediaNextTrack,
        Keysym::XF86_AudioPlay => Key::MediaPlayPause,
        Keysym::XF86_AudioPrev => Key::MediaPrevTrack,
        Keysym::XF86_AudioStop => Key::MediaStop,
        Keysym::Num_Lock => Key::Numlock,
        Keysym::KP_0 => Key::Numpad0,
        Keysym::KP_1 => Key::Numpad1,
        Keysym::KP_2 => Key::Numpad2,
        Keysym::KP_3 => Key::Numpad3,
        Keysym::KP_4 => Key::Numpad4,
        Keysym::KP_5 => Key::Numpad5,
        Keysym::KP_6 => Key::Numpad6,
        Keysym::KP_7 => Key::Numpad7,
        Keysym::KP_8 => Key::Numpad8,
        Keysym::KP_9 => Key::Numpad9,
        Keysym::Page_Down => Key::PageDown,
        Keysym::Page_Up => Key::PageUp,
        Keysym::Pause => Key::Pause,
        Keysym::Print => Key::PrintScr,
        Keysym::Control_R => Key::RControl,
        Keysym::Redo => Key::Redo,
        Keysym::Return => Key::Return,
        Keysym::Right => Key::RightArrow,
        Keysym::Shift_R => Key::RShift,
        Keysym::Scroll_Lock => Key::ScrollLock,
        Keysym::Select => Key::Select,
        Keysym::Shift_L => Key::LShift,
        Keysym::Shift_Lock => Key::ShiftLock,
        Keysym::space => Key::Space,
        Keysym::KP_Subtract => Key::Subtract,
        Keysym::Sys_Req => Key::SysReq,
        Keysym::Tab => Key::Tab,
        Keysym::Undo => Key::Undo,
        Keysym::Up => Key::UpArrow,
        Keysym::XF86_AudioLowerVolume => Key::VolumeDown,
        Keysym::XF86_AudioRaiseVolume => Key::VolumeUp,
        Keysym::XF86_AudioMute => Key::VolumeMute,
        Keysym::XF86_AudioMicMute => Key::MicMute,
        Keysym::Super_L | Keysym::Super_R => Key::Meta,
        other => other
            .key_char()
            .map_or_else(|| Key::Other(other.into()), Key::Unicode),
    }
}
