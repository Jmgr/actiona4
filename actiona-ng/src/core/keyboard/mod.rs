use std::sync::{Arc, Mutex};

use convert_case::{Case, Casing};
use enigo::{Direction, Enigo, InputError, NewConError};
use macros::ExposeEnum;
use platform::KeyboardImplTrait;
use rquickjs::{JsLifetime, class::Trace};
use strum::Display;
use thiserror::Error;
use tracing::instrument;

pub(crate) mod platform;

pub mod js;

pub use enigo::Coordinate;
#[cfg(windows)]
use platform::win::KeyboardImpl;
#[cfg(unix)]
use platform::x11::KeyboardImpl;

use crate::runtime::Runtime;

#[derive(Debug, Error)]
pub enum KeyboardError {
    #[error("Connecting to the X11 server failed: {0}")]
    ConnectError(String),

    #[error("Connection to the X11 server failed: {0}")]
    ConnectionError(String),

    #[error("X11 reply error: {0}")]
    ReplyError(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),

    #[error("Enigo new connection error: {0}")]
    EnigoNewConnError(#[from] NewConError),

    #[error("Enigo input error: {0}")]
    EnigoInputError(#[from] InputError),

    #[error("{0}")]
    ParameterError(String),
}

pub type Result<T> = std::result::Result<T, KeyboardError>;

#[derive(Clone, Copy, Debug, Display, Eq, ExposeEnum, Hash, JsLifetime, PartialEq, Trace)]
#[rquickjs::class(rename = "Key")]
pub enum JsKey {
    Alt,
    Backspace,
    Cancel,
    CapsLock,
    Clear,
    Control,
    Delete,
    DownArrow,
    End,
    Escape,
    Execute,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Hangul,
    Hanja,
    Help,
    Home,
    Insert,
    Kanji,
    LControl,
    LeftArrow,
    LMenu,
    LShift,
    MediaNextTrack,
    MediaPlayPause,
    MediaPrevTrack,
    MediaStop,
    Meta,
    ModeChange,
    Numlock,
    Option,
    PageDown,
    PageUp,
    Pause,
    PrintScr,
    RControl,
    Return,
    RightArrow,
    RShift,
    Select,
    Shift,
    Space,
    Tab,
    UpArrow,
    VolumeDown,
    VolumeMute,
    VolumeUp,
    Windows,
}

impl JsKey {
    const fn into_enigo(self) -> enigo::Key {
        use JsKey::*;

        match self {
            Alt => enigo::Key::Alt,
            Backspace => enigo::Key::Backspace,
            Cancel => enigo::Key::Cancel,
            CapsLock => enigo::Key::CapsLock,
            Clear => enigo::Key::Clear,
            Control => enigo::Key::Control,
            Delete => enigo::Key::Delete,
            DownArrow => enigo::Key::DownArrow,
            End => enigo::Key::End,
            Escape => enigo::Key::Escape,
            Execute => enigo::Key::Execute,
            F1 => enigo::Key::F1,
            F2 => enigo::Key::F2,
            F3 => enigo::Key::F3,
            F4 => enigo::Key::F4,
            F5 => enigo::Key::F5,
            F6 => enigo::Key::F6,
            F7 => enigo::Key::F7,
            F8 => enigo::Key::F8,
            F9 => enigo::Key::F9,
            F10 => enigo::Key::F10,
            F11 => enigo::Key::F11,
            F12 => enigo::Key::F12,
            F13 => enigo::Key::F13,
            F14 => enigo::Key::F14,
            F15 => enigo::Key::F15,
            F16 => enigo::Key::F16,
            F17 => enigo::Key::F17,
            F18 => enigo::Key::F18,
            F19 => enigo::Key::F19,
            F20 => enigo::Key::F20,
            F21 => enigo::Key::F21,
            F22 => enigo::Key::F22,
            F23 => enigo::Key::F23,
            F24 => enigo::Key::F24,
            Hangul => enigo::Key::Hangul,
            Hanja => enigo::Key::Hanja,
            Help => enigo::Key::Help,
            Home => enigo::Key::Home,
            Insert => enigo::Key::Insert,
            Kanji => enigo::Key::Kanji,
            LControl => enigo::Key::LControl,
            LeftArrow => enigo::Key::LeftArrow,
            LMenu => enigo::Key::LMenu,
            LShift => enigo::Key::LShift,
            MediaNextTrack => enigo::Key::MediaNextTrack,
            MediaPlayPause => enigo::Key::MediaPlayPause,
            MediaPrevTrack => enigo::Key::MediaPrevTrack,
            MediaStop => enigo::Key::MediaStop,
            Meta => enigo::Key::Meta,
            ModeChange => enigo::Key::ModeChange,
            Numlock => enigo::Key::Numlock,
            Option => enigo::Key::Option,
            PageDown => enigo::Key::PageDown,
            PageUp => enigo::Key::PageUp,
            Pause => enigo::Key::Pause,
            PrintScr => enigo::Key::PrintScr,
            RControl => enigo::Key::RControl,
            Return => enigo::Key::Return,
            RightArrow => enigo::Key::RightArrow,
            RShift => enigo::Key::RShift,
            Select => enigo::Key::Select,
            Shift => enigo::Key::Shift,
            Space => enigo::Key::Space,
            Tab => enigo::Key::Tab,
            UpArrow => enigo::Key::UpArrow,
            VolumeDown => enigo::Key::VolumeDown,
            VolumeMute => enigo::Key::VolumeMute,
            VolumeUp => enigo::Key::VolumeUp,
            Windows => enigo::Key::Meta,
        }
    }
}

#[derive(Debug)]
pub struct Keyboard {
    enigo: Arc<Mutex<Enigo>>,
    implementation: KeyboardImpl,
}

impl Keyboard {
    #[instrument]
    pub fn new(runtime: Arc<Runtime>) -> Result<Self> {
        Ok(Self {
            enigo: runtime.enigo(),
            implementation: KeyboardImpl::new(runtime)?,
        })
    }

    #[instrument(skip(self), err, ret)]
    pub fn text(&self, text: &str) -> Result<()> {
        use enigo::Keyboard;

        self.enigo.lock().unwrap().text(text)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn key(&self, key: JsKey, direction: Direction) -> Result<()> {
        use enigo::Keyboard;

        self.enigo
            .lock()
            .unwrap()
            .key(key.into_enigo(), direction)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn raw(&self, keycode: u16, direction: Direction) -> Result<()> {
        use enigo::Keyboard;

        self.enigo.lock().unwrap().raw(keycode, direction)?;

        Ok(())
    }

    pub fn is_key_pressed(&self, key: JsKey) -> Result<bool> {
        self.implementation.is_key_pressed(key)
    }
}

#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use tracing_test::traced_test;

    use crate::runtime::Runtime;

    #[test]
    #[traced_test]
    fn test_keyboard() {
        Runtime::test(async |_runtime| {
            //let _keyboard = Keyboard::new(runtime).unwrap();

            //keyboard.text("hello").unwrap();

            sleep(Duration::from_secs(1))
        });
    }
}
