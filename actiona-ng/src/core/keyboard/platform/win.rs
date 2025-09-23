use std::sync::Arc;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_BACK, VK_CANCEL, VK_CAPITAL, VK_CLEAR, VK_CONTROL, VK_DELETE, VK_DOWN,
    VK_END, VK_ESCAPE, VK_EXECUTE, VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9,
    VK_F10, VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16, VK_F17, VK_F18, VK_F19, VK_F20, VK_F21,
    VK_F22, VK_F23, VK_F24, VK_HANGUL, VK_HANJA, VK_HELP, VK_HOME, VK_INSERT, VK_KANJI,
    VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE,
    VK_MEDIA_PREV_TRACK, VK_MEDIA_STOP, VK_MENU, VK_MODECHANGE, VK_NEXT, VK_NUMLOCK, VK_PAUSE,
    VK_PRIOR, VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RSHIFT, VK_SELECT, VK_SHIFT, VK_SNAPSHOT,
    VK_SPACE, VK_TAB, VK_UP, VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP,
};

use super::{Key, KeyboardImplTrait, Result};
use crate::runtime::Runtime;

#[derive(Clone, Debug, Default)]
pub struct KeyboardImpl;

impl Key {
    fn into_vkey(self) -> i32 {
        match self {
            Key::Alt => VK_MENU,
            Key::Backspace => VK_BACK,
            Key::Cancel => VK_CANCEL,
            Key::CapsLock => VK_CAPITAL,
            Key::Clear => VK_CLEAR,
            Key::Control => VK_CONTROL,
            Key::Delete => VK_DELETE,
            Key::DownArrow => VK_DOWN,
            Key::End => VK_END,
            Key::Escape => VK_ESCAPE,
            Key::Execute => VK_EXECUTE,
            Key::F1 => VK_F1,
            Key::F2 => VK_F2,
            Key::F3 => VK_F3,
            Key::F4 => VK_F4,
            Key::F5 => VK_F5,
            Key::F6 => VK_F6,
            Key::F7 => VK_F7,
            Key::F8 => VK_F8,
            Key::F9 => VK_F9,
            Key::F10 => VK_F10,
            Key::F11 => VK_F11,
            Key::F12 => VK_F12,
            Key::F13 => VK_F13,
            Key::F14 => VK_F14,
            Key::F15 => VK_F15,
            Key::F16 => VK_F16,
            Key::F17 => VK_F17,
            Key::F18 => VK_F18,
            Key::F19 => VK_F19,
            Key::F20 => VK_F20,
            Key::F21 => VK_F21,
            Key::F22 => VK_F22,
            Key::F23 => VK_F23,
            Key::F24 => VK_F24,
            Key::Hangul => VK_HANGUL,
            Key::Hanja => VK_HANJA,
            Key::Help => VK_HELP,
            Key::Home => VK_HOME,
            Key::Insert => VK_INSERT,
            Key::Kanji => VK_KANJI,
            Key::LControl => VK_LCONTROL,
            Key::LeftArrow => VK_LEFT,
            Key::LMenu => VK_LMENU,
            Key::LShift => VK_LSHIFT,
            Key::MediaNextTrack => VK_MEDIA_NEXT_TRACK,
            Key::MediaPlayPause => VK_MEDIA_PLAY_PAUSE,
            Key::MediaPrevTrack => VK_MEDIA_PREV_TRACK,
            Key::MediaStop => VK_MEDIA_STOP,
            Key::Meta => VK_LWIN,
            Key::ModeChange => VK_MODECHANGE,
            Key::Numlock => VK_NUMLOCK,
            Key::Option => VK_MENU,
            Key::PageDown => VK_NEXT,
            Key::PageUp => VK_PRIOR,
            Key::Pause => VK_PAUSE,
            Key::PrintScr => VK_SNAPSHOT,
            Key::RControl => VK_RCONTROL,
            Key::Return => VK_RETURN,
            Key::RightArrow => VK_RIGHT,
            Key::RShift => VK_RSHIFT,
            Key::Select => VK_SELECT,
            Key::Shift => VK_SHIFT,
            Key::Space => VK_SPACE,
            Key::Tab => VK_TAB,
            Key::UpArrow => VK_UP,
            Key::VolumeDown => VK_VOLUME_DOWN,
            Key::VolumeMute => VK_VOLUME_MUTE,
            Key::VolumeUp => VK_VOLUME_UP,
            Key::Windows => VK_LWIN,
        }
        .0
        .into()
    }
}

impl KeyboardImpl {
    pub fn new(_runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self)
    }
}

#[allow(unsafe_code)]
impl KeyboardImplTrait for KeyboardImpl {
    fn is_key_pressed(&self, key: Key) -> Result<bool> {
        Ok(unsafe { GetAsyncKeyState(key.into_vkey()) as u16 & 0x8000u16 != 0 })
    }
}
