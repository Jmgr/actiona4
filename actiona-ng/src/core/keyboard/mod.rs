use std::sync::{Arc, Mutex};

use enigo::{Direction, Enigo, InputError, NewConError};
use macros::{FromSerde, IntoSerde};
use platform::KeyboardImplTrait;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};
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

#[derive(
    Clone,
    Copy,
    Debug,
    Display,
    Eq,
    Hash,
    PartialEq,
    Serialize,
    Deserialize,
    EnumIter,
    IntoSerde,
    FromSerde,
)]
pub enum Key {
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

impl From<Key> for enigo::Key {
    fn from(value: Key) -> Self {
        use Key::*;

        match value {
            Alt => Self::Alt,
            Backspace => Self::Backspace,
            Cancel => Self::Cancel,
            CapsLock => Self::CapsLock,
            Clear => Self::Clear,
            Control => Self::Control,
            Delete => Self::Delete,
            DownArrow => Self::DownArrow,
            End => Self::End,
            Escape => Self::Escape,
            Execute => Self::Execute,
            F1 => Self::F1,
            F2 => Self::F2,
            F3 => Self::F3,
            F4 => Self::F4,
            F5 => Self::F5,
            F6 => Self::F6,
            F7 => Self::F7,
            F8 => Self::F8,
            F9 => Self::F9,
            F10 => Self::F10,
            F11 => Self::F11,
            F12 => Self::F12,
            F13 => Self::F13,
            F14 => Self::F14,
            F15 => Self::F15,
            F16 => Self::F16,
            F17 => Self::F17,
            F18 => Self::F18,
            F19 => Self::F19,
            F20 => Self::F20,
            F21 => Self::F21,
            F22 => Self::F22,
            F23 => Self::F23,
            F24 => Self::F24,
            Hangul => Self::Hangul,
            Hanja => Self::Hanja,
            Help => Self::Help,
            Home => Self::Home,
            Insert => Self::Insert,
            Kanji => Self::Kanji,
            LControl => Self::LControl,
            LeftArrow => Self::LeftArrow,
            LMenu => Self::LMenu,
            LShift => Self::LShift,
            MediaNextTrack => Self::MediaNextTrack,
            MediaPlayPause => Self::MediaPlayPause,
            MediaPrevTrack => Self::MediaPrevTrack,
            MediaStop => Self::MediaStop,
            Meta => Self::Meta,
            ModeChange => Self::ModeChange,
            Numlock => Self::Numlock,
            Option => Self::Option,
            PageDown => Self::PageDown,
            PageUp => Self::PageUp,
            Pause => Self::Pause,
            PrintScr => Self::PrintScr,
            RControl => Self::RControl,
            Return => Self::Return,
            RightArrow => Self::RightArrow,
            RShift => Self::RShift,
            Select => Self::Select,
            Shift => Self::Shift,
            Space => Self::Space,
            Tab => Self::Tab,
            UpArrow => Self::UpArrow,
            VolumeDown => Self::VolumeDown,
            VolumeMute => Self::VolumeMute,
            VolumeUp => Self::VolumeUp,
            Windows => Self::Meta,
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
    pub fn key(&self, key: Key, direction: Direction) -> Result<()> {
        use enigo::Keyboard;

        self.enigo.lock().unwrap().key(key.into(), direction)?;

        Ok(())
    }

    #[instrument(skip(self), err, ret)]
    pub fn raw(&self, keycode: u16, direction: Direction) -> Result<()> {
        use enigo::Keyboard;

        self.enigo.lock().unwrap().raw(keycode, direction)?;

        Ok(())
    }

    pub fn is_key_pressed(&self, key: Key) -> Result<bool> {
        self.implementation.is_key_pressed(key)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::time::sleep;
    use tracing_test::traced_test;

    use crate::runtime::Runtime;

    #[test]
    #[traced_test]
    fn test_keyboard() {
        Runtime::test(async |_runtime| {
            //let _keyboard = Keyboard::new(runtime).unwrap();

            //keyboard.text("hello").unwrap();

            sleep(Duration::from_secs(1)).await
        });
    }
}
