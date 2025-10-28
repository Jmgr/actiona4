use std::sync::Arc;

use super::{Key, KeyboardImplTrait, Result};
use crate::runtime::Runtime;

#[derive(Debug)]
pub struct KeyboardImpl {
    _runtime: Arc<Runtime>,
}

impl KeyboardImpl {
    pub const fn new(runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self { _runtime: runtime })
    }
}

impl KeyboardImplTrait for KeyboardImpl {
    fn is_key_pressed(&self, _key: Key) -> Result<bool> {
        // TODO
        Ok(false)
    }
}

impl Key {
    fn into_keysym(self) -> xkeysym::Keysym {
        let k = enigo::Key::Add;
        k.into()

        /*
        use xkeysym::Keysym;

        match self {
            Key::Unicode(c) => xkeysym::Keysym::from_char(c),
            Key::Add => Keysym::KP_Add,
            Key::Alt | Key::Option => Keysym::Alt_L,
            Key::Backspace => Keysym::BackSpace,
            Key::Begin => Keysym::Begin,
            Key::Break => Keysym::Break,
            Key::Cancel => Keysym::Cancel,
            Key::CapsLock => Keysym::Caps_Lock,
            Key::Clear => Keysym::Clear,
            Key::Control | Key::LControl => Keysym::Control_L,
            Key::Decimal => Keysym::KP_Decimal,
            Key::Delete => Keysym::Delete,
            Key::Divide => Keysym::KP_Divide,
            Key::DownArrow => Keysym::Down,
            Key::End => Keysym::End,
            Key::NumpadEnter => Keysym::KP_Enter,
            Key::Escape => Keysym::Escape,
            Key::Execute => Keysym::Execute,
            Key::F1 => Keysym::F1,
            Key::F2 => Keysym::F2,
            Key::F3 => Keysym::F3,
            Key::F4 => Keysym::F4,
            Key::F5 => Keysym::F5,
            Key::F6 => Keysym::F6,
            Key::F7 => Keysym::F7,
            Key::F8 => Keysym::F8,
            Key::F9 => Keysym::F9,
            Key::F10 => Keysym::F10,
            Key::F11 => Keysym::F11,
            Key::F12 => Keysym::F12,
            Key::F13 => Keysym::F13,
            Key::F14 => Keysym::F14,
            Key::F15 => Keysym::F15,
            Key::F16 => Keysym::F16,
            Key::F17 => Keysym::F17,
            Key::F18 => Keysym::F18,
            Key::F19 => Keysym::F19,
            Key::F20 => Keysym::F20,
            Key::F21 => Keysym::F21,
            Key::F22 => Keysym::F22,
            Key::F23 => Keysym::F23,
            Key::F24 => Keysym::F24,
            Key::F25 => Keysym::F25,
            Key::F26 => Keysym::F26,
            Key::F27 => Keysym::F27,
            Key::F28 => Keysym::F28,
            Key::F29 => Keysym::F29,
            Key::F30 => Keysym::F30,
            Key::F31 => Keysym::F31,
            Key::F32 => Keysym::F32,
            Key::F33 => Keysym::F33,
            Key::F34 => Keysym::F34,
            Key::F35 => Keysym::F35,
            Key::Find => Keysym::Find,
            Key::Hangul => Keysym::Hangul,
            Key::Hanja => Keysym::Hangul_Hanja,
            Key::Help => Keysym::Help,
            Key::Home => Keysym::Home,
            Key::Insert => Keysym::Insert,
            Key::Kanji => Keysym::Kanji,
            Key::LeftArrow => Keysym::Left,
            Key::Linefeed => Keysym::Linefeed,
            Key::LMenu => Keysym::Menu,
            Key::ModeChange => Keysym::Mode_switch,
            Key::Multiply => Keysym::KP_Multiply,
            Key::MediaNextTrack => Keysym::XF86_AudioNext,
            Key::MediaPlayPause => Keysym::XF86_AudioPlay,
            Key::MediaPrevTrack => Keysym::XF86_AudioPrev,
            Key::MediaStop => Keysym::XF86_AudioStop,
            Key::Numlock => Keysym::Num_Lock,
            Key::Numpad0 => Keysym::KP_0,
            Key::Numpad1 => Keysym::KP_1,
            Key::Numpad2 => Keysym::KP_2,
            Key::Numpad3 => Keysym::KP_3,
            Key::Numpad4 => Keysym::KP_4,
            Key::Numpad5 => Keysym::KP_5,
            Key::Numpad6 => Keysym::KP_6,
            Key::Numpad7 => Keysym::KP_7,
            Key::Numpad8 => Keysym::KP_8,
            Key::Numpad9 => Keysym::KP_9,
            Key::PageDown => Keysym::Page_Down,
            Key::PageUp => Keysym::Page_Up,
            Key::Pause => Keysym::Pause,
            Key::Print => Keysym::Print,
            Key::PrintScr => Keysym::Print,
            Key::RControl => Keysym::Control_R,
            Key::Redo => Keysym::Redo,
            Key::Return => Keysym::Return,
            Key::RightArrow => Keysym::Right,
            Key::RShift => Keysym::Shift_R,
            Key::ScrollLock => Keysym::Scroll_Lock,
            Key::Select => Keysym::Select,
            Key::ScriptSwitch => Keysym::script_switch,
            Key::Shift | Key::LShift => Keysym::Shift_L,
            Key::ShiftLock => Keysym::Shift_Lock,
            Key::Space => Keysym::space,
            Key::Subtract => Keysym::KP_Subtract,
            Key::SysReq => Keysym::Sys_Req,
            Key::Tab => Keysym::Tab,
            Key::Undo => Keysym::Undo,
            Key::UpArrow => Keysym::Up,
            Key::VolumeDown => Keysym::XF86_AudioLowerVolume,
            Key::VolumeUp => Keysym::XF86_AudioRaiseVolume,
            Key::VolumeMute => Keysym::XF86_AudioMute,
            Key::MicMute => Keysym::XF86_AudioMicMute,
            Key::Command | Key::Super | Key::Windows | Key::Meta => Keysym::Super_L,
            Key::Other(v) => Keysym::from(v),
        }
        .0
        .into()
          */
    }
}
