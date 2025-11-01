use std::sync::Arc;

use enigo::Key;
use eyre::{Result, eyre};
use tokio::select;
use tokio_util::sync::CancellationToken;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VIRTUAL_KEY, VK_BACK, VK_CANCEL, VK_CAPITAL, VK_CLEAR, VK_CONTROL, VK_DELETE,
    VK_DOWN, VK_END, VK_ESCAPE, VK_EXECUTE, VK_F1, VK_F2, VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8,
    VK_F9, VK_F10, VK_F11, VK_F12, VK_F13, VK_F14, VK_F15, VK_F16, VK_F17, VK_F18, VK_F19, VK_F20,
    VK_F21, VK_F22, VK_F23, VK_F24, VK_HANGUL, VK_HANJA, VK_HELP, VK_HOME, VK_INSERT, VK_KANJI,
    VK_LCONTROL, VK_LEFT, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE,
    VK_MEDIA_PREV_TRACK, VK_MEDIA_STOP, VK_MENU, VK_MODECHANGE, VK_NEXT, VK_NUMLOCK, VK_PAUSE,
    VK_PRIOR, VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RSHIFT, VK_SELECT, VK_SHIFT, VK_SNAPSHOT,
    VK_SPACE, VK_TAB, VK_UP, VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP,
};

use crate::{error::CommonError, runtime::Runtime};

#[derive(Debug)]
pub struct KeyboardImpl {
    runtime: Arc<Runtime>,
}

impl KeyboardImpl {
    pub fn new(runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self { runtime })
    }

    #[allow(unsafe_code)]
    pub fn is_key_pressed(&self, key: Key) -> Result<bool> {
        let key = VIRTUAL_KEY::try_from(key).map_err(|err| eyre!("invalid key: {err}"))?;

        Ok(unsafe { GetAsyncKeyState(key.0.into()) as u16 & 0x8000u16 != 0 })
    }

    pub async fn wait_for_key(
        &self,
        //conditions: ButtonConditions,
        cancellation_token: CancellationToken,
    ) -> Result<Key> {
        // MouseButtonEvent
        let guard = self
            .runtime
            .platform()
            .keyboard_input_dispatcher()
            .subscribe_keyboard_keys();
        let mut receiver = guard.subscribe();
        let runtime_cancellation_token = self.runtime.cancellation_token();
        loop {
            let event = select! {
                _ = runtime_cancellation_token.cancelled() => { break; }
                _ = cancellation_token.cancelled() => { break; }
                event = receiver.recv() => { event }
            };

            let Ok(event) = event else {
                break;
            };

            if event.key == Key::Escape {
                runtime_cancellation_token.cancel();
            }

            println!("{:?}", event);

            return Ok(event.key); // TODO

            /*
            let button_result = conditions
                .button
                .is_none_or(|button| button == event.button);
            let direction_result = conditions
                .direction
                .is_none_or(|direction| direction == event.direction);

            if button_result && direction_result {
                return Ok(event);
            }
            */
        }

        Err(CommonError::Cancelled.into())
    }
}
