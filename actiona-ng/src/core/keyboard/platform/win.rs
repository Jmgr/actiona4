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

impl KeyboardImpl {
    pub fn new(_runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self)
    }

    #[allow(unsafe_code)]
    fn is_key_pressed(&self, key: Key) -> Result<bool> {
        Ok(unsafe { GetAsyncKeyState(key.into_vkey()) as u16 & 0x8000u16 != 0 })
    }
}
