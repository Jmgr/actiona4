use std::sync::Arc;

use rquickjs::{
    Ctx, Exception, JsLifetime, Result,
    class::{Trace, Tracer},
};

use crate::{IntoJsResult, core::js::classes::SingletonClass, runtime::Runtime};

impl<T> IntoJsResult<T> for super::Result<T> {
    fn into_js_result(self, ctx: &Ctx<'_>) -> Result<T> {
        self.map_err(|err| Exception::throw_message(ctx, &err.to_string()))
    }
}

impl<'js> Trace<'js> for super::Keyboard {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

/// @singleton
#[derive(Debug, JsLifetime, Trace)]
#[rquickjs::class(rename = "Keyboard")]
pub struct JsKeyboard {
    inner: super::Keyboard,
}

impl SingletonClass<'_> for JsKeyboard {}

impl JsKeyboard {
    /// @skip
    pub fn new(runtime: Arc<Runtime>) -> super::Result<Self> {
        Ok(Self {
            inner: super::Keyboard::new(runtime)?,
        })
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsKeyboard {
    // TODO
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum JsKey {
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    /// @platforms =windows
    AbntC1,
    /// @platforms =windows
    AbntC2,
    /// @platforms =windows
    Accept,
    Add,
    Alt,
    /// @platforms =windows
    Apps,
    /// @platforms =windows
    Attn,
    Backspace,
    /// @platforms =linux
    Break,
    /// @platforms =linux
    Begin,
    /// @platforms =windows
    BrowserBack,
    /// @platforms =windows
    BrowserFavorites,
    /// @platforms =windows
    BrowserForward,
    /// @platforms =windows
    BrowserHome,
    /// @platforms =windows
    BrowserRefresh,
    /// @platforms =windows
    BrowserSearch,
    /// @platforms =windows
    BrowserStop,
    Cancel,
    CapsLock,
    Clear,
    Control,
    /// @platforms =windows
    Convert,
    /// @platforms =windows
    Crsel,
    /// @platforms =windows
    DBEAlphanumeric,
    /// @platforms =windows
    DBECodeinput,
    /// @platforms =windows
    DBEDetermineString,
    /// @platforms =windows
    DBEEnterDLGConversionMode,
    /// @platforms =windows
    DBEEnterIMEConfigMode,
    /// @platforms =windows
    DBEEnterWordRegisterMode,
    /// @platforms =windows
    DBEFlushString,
    /// @platforms =windows
    DBEHiragana,
    /// @platforms =windows
    DBEKatakana,
    /// @platforms =windows
    DBENoCodepoint,
    /// @platforms =windows
    DBENoRoman,
    /// @platforms =windows
    DBERoman,
    /// @platforms =windows
    DBESBCSChar,
    /// @platforms =windows
    DBESChar,
    Decimal,
    Delete,
    Divide,
    DownArrow,
    End,
    /// @platforms =windows
    Ereof,
    Escape,
    Execute,
    /// @platforms =windows
    Exsel,
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
    /// @platforms =linux
    F25,
    /// @platforms =linux
    F26,
    /// @platforms =linux
    F27,
    /// @platforms =linux
    F28,
    /// @platforms =linux
    F29,
    /// @platforms =linux
    F30,
    /// @platforms =linux
    F31,
    /// @platforms =linux
    F32,
    /// @platforms =linux
    F33,
    /// @platforms =linux
    F34,
    /// @platforms =linux
    F35,
    /// @platforms =windows
    Final,
    /// @platforms =linux
    Find,
    /// @platforms =windows
    GamepadA,
    /// @platforms =windows
    GamepadB,
    /// @platforms =windows
    GamepadDPadDown,
    /// @platforms =windows
    GamepadDPadLeft,
    /// @platforms =windows
    GamepadDPadRight,
    /// @platforms =windows
    GamepadDPadUp,
    /// @platforms =windows
    GamepadLeftShoulder,
    /// @platforms =windows
    GamepadLeftThumbstickButton,
    /// @platforms =windows
    GamepadLeftThumbstickDown,
    /// @platforms =windows
    GamepadLeftThumbstickLeft,
    /// @platforms =windows
    GamepadLeftThumbstickRight,
    /// @platforms =windows
    GamepadLeftThumbstickUp,
    /// @platforms =windows
    GamepadLeftTrigger,
    /// @platforms =windows
    GamepadMenu,
    /// @platforms =windows
    GamepadRightShoulder,
    /// @platforms =windows
    GamepadRightThumbstickButton,
    /// @platforms =windows
    GamepadRightThumbstickDown,
    /// @platforms =windows
    GamepadRightThumbstickLeft,
    /// @platforms =windows
    GamepadRightThumbstickRight,
    /// @platforms =windows
    GamepadRightThumbstickUp,
    /// @platforms =windows
    GamepadRightTrigger,
    /// @platforms =windows
    GamepadView,
    /// @platforms =windows
    GamepadX,
    /// @platforms =windows
    GamepadY,
    /// @platforms =windows
    Hangeul,
    Hangul,
    Hanja,
    Help,
    Home,
    /// @platforms =windows
    Ico00,
    /// @platforms =windows
    IcoClear,
    /// @platforms =windows
    IcoHelp,
    /// @platforms =windows
    IMEOff,
    /// @platforms =windows
    IMEOn,
    Insert,
    /// @platforms =windows
    Junja,
    /// @platforms =windows
    Kana,
    Kanji,
    /// @platforms =windows
    LaunchApp1,
    /// @platforms =windows
    LaunchApp2,
    /// @platforms =windows
    LaunchMail,
    /// @platforms =windows
    LaunchMediaSelect,
    /// @platforms =windows
    LButton,
    LControl,
    LeftArrow,
    /// @platforms =linux
    Linefeed,
    LMenu,
    LShift,
    /// @platforms =windows
    LWin,
    /// @platforms =windows
    MButton,
    MediaNextTrack,
    MediaPlayPause,
    MediaPrevTrack,
    MediaStop,
    /// meta key (also known as "windows", "super", and "command")
    Meta,
    ModeChange,
    Multiply,
    /// @platforms =windows
    NavigationAccept,
    /// @platforms =windows
    NavigationCancel,
    /// @platforms =windows
    NavigationDown,
    /// @platforms =windows
    NavigationLeft,
    /// @platforms =windows
    NavigationMenu,
    /// @platforms =windows
    NavigationRight,
    /// @platforms =windows
    NavigationUp,
    /// @platforms =windows
    NavigationView,
    /// @platforms =windows
    NoName,
    /// @platforms =windows
    NonConvert,
    /// @platforms =windows
    None,
    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadEnter,
    /// @platforms =windows
    OEM1,
    /// @platforms =windows
    OEM102,
    /// @platforms =windows
    OEM2,
    /// @platforms =windows
    OEM3,
    /// @platforms =windows
    OEM4,
    /// @platforms =windows
    OEM5,
    /// @platforms =windows
    OEM6,
    /// @platforms =windows
    OEM7,
    /// @platforms =windows
    OEM8,
    /// @platforms =windows
    OEMAttn,
    /// @platforms =windows
    OEMAuto,
    /// @platforms =windows
    OEMAx,
    /// @platforms =windows
    OEMBacktab,
    /// @platforms =windows
    OEMClear,
    /// @platforms =windows
    OEMComma,
    /// @platforms =windows
    OEMCopy,
    /// @platforms =windows
    OEMCusel,
    /// @platforms =windows
    OEMEnlw,
    /// @platforms =windows
    OEMFinish,
    /// @platforms =windows
    OEMFJJisho,
    /// @platforms =windows
    OEMFJLoya,
    /// @platforms =windows
    OEMFJMasshou,
    /// @platforms =windows
    OEMFJRoya,
    /// @platforms =windows
    OEMFJTouroku,
    /// @platforms =windows
    OEMJump,
    /// @platforms =windows
    OEMMinus,
    /// @platforms =windows
    OEMNECEqual,
    /// @platforms =windows
    OEMPA1,
    /// @platforms =windows
    OEMPA2,
    /// @platforms =windows
    OEMPA3,
    /// @platforms =windows
    OEMPeriod,
    /// @platforms =windows
    OEMPlus,
    /// @platforms =windows
    OEMReset,
    /// @platforms =windows
    OEMWsctrl,
    /// Same as Alt
    Option,
    /// @platforms =windows
    PA1,
    /// @platforms =windows
    Packet,
    PageDown,
    PageUp,
    Pause,
    /// @platforms =windows
    Play,
    /// Screenshot
    PrintScr,
    /// @platforms =windows
    Processkey,
    /// @platforms =windows
    RButton,
    RControl,
    /// @platforms =linux
    Redo,
    Return,
    RightArrow,
    /// @platforms =windows
    RMenu,
    RShift,
    /// @platforms =windows
    RWin,
    /// @platforms =windows
    Scroll,
    /// @platforms =linux
    ScrollLock,
    Select,
    /// @platforms =linux
    ScriptSwitch,
    /// @platforms =windows
    Separator,
    Shift,
    /// @platforms =linux
    ShiftLock,
    /// @platforms =windows
    Sleep,
    Space,
    Subtract,
    /// @platforms =linux
    SysReq,
    Tab,
    /// @platforms =linux
    Undo,
    UpArrow,
    VolumeDown,
    VolumeMute,
    VolumeUp,
    /// @platforms =linux
    MicMute,
    /// @platforms =windows
    XButton1,
    /// @platforms =windows
    XButton2,
    /// @platforms =windows
    Zoom,
}

pub enum KeyError {
    Unsupported,
}

impl TryFrom<JsKey> for enigo::Key {
    type Error = KeyError;

    fn try_from(value: JsKey) -> std::result::Result<Self, KeyError> {
        use JsKey::*;
        Ok(match value {
            Add => Self::Add,
            Alt => Self::Alt,
            Backspace => Self::Backspace,
            Cancel => Self::Cancel,
            CapsLock => Self::CapsLock,
            Clear => Self::Clear,
            Control => Self::Control,
            Decimal => Self::Decimal,
            Delete => Self::Delete,
            Divide => Self::Divide,
            DownArrow => Self::DownArrow,
            End => Self::End,
            Escape => Self::Escape,
            Execute => Self::Execute,
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
            Multiply => Self::Multiply,
            Numlock => Self::Numlock,
            Numpad0 => Self::Numpad0,
            Numpad1 => Self::Numpad1,
            Numpad2 => Self::Numpad2,
            Numpad3 => Self::Numpad3,
            Numpad4 => Self::Numpad4,
            Numpad5 => Self::Numpad5,
            Numpad6 => Self::Numpad6,
            Numpad7 => Self::Numpad7,
            Numpad8 => Self::Numpad8,
            Numpad9 => Self::Numpad9,
            NumpadEnter => Self::NumpadEnter,
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
            Subtract => Self::Subtract,
            Tab => Self::Tab,
            UpArrow => Self::UpArrow,
            VolumeDown => Self::VolumeDown,
            VolumeMute => Self::VolumeMute,
            VolumeUp => Self::VolumeUp,

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

            #[cfg(target_os = "windows")]
            Num0 => Self::Num0,
            #[cfg(target_os = "windows")]
            Num1 => Self::Num1,
            #[cfg(target_os = "windows")]
            Num2 => Self::Num2,
            #[cfg(target_os = "windows")]
            Num3 => Self::Num3,
            #[cfg(target_os = "windows")]
            Num4 => Self::Num4,
            #[cfg(target_os = "windows")]
            Num5 => Self::Num5,
            #[cfg(target_os = "windows")]
            Num6 => Self::Num6,
            #[cfg(target_os = "windows")]
            Num7 => Self::Num7,
            #[cfg(target_os = "windows")]
            Num8 => Self::Num8,
            #[cfg(target_os = "windows")]
            Num9 => Self::Num9,

            #[cfg(target_os = "windows")]
            A => Self::A,
            #[cfg(target_os = "windows")]
            B => Self::B,
            #[cfg(target_os = "windows")]
            C => Self::C,
            #[cfg(target_os = "windows")]
            D => Self::D,
            #[cfg(target_os = "windows")]
            E => Self::E,
            #[cfg(target_os = "windows")]
            F => Self::F,
            #[cfg(target_os = "windows")]
            G => Self::G,
            #[cfg(target_os = "windows")]
            H => Self::H,
            #[cfg(target_os = "windows")]
            I => Self::I,
            #[cfg(target_os = "windows")]
            J => Self::J,
            #[cfg(target_os = "windows")]
            K => Self::K,
            #[cfg(target_os = "windows")]
            L => Self::L,
            #[cfg(target_os = "windows")]
            M => Self::M,
            #[cfg(target_os = "windows")]
            N => Self::N,
            #[cfg(target_os = "windows")]
            O => Self::O,
            #[cfg(target_os = "windows")]
            P => Self::P,
            #[cfg(target_os = "windows")]
            Q => Self::Q,
            #[cfg(target_os = "windows")]
            R => Self::R,
            #[cfg(target_os = "windows")]
            S => Self::S,
            #[cfg(target_os = "windows")]
            T => Self::T,
            #[cfg(target_os = "windows")]
            U => Self::U,
            #[cfg(target_os = "windows")]
            V => Self::V,
            #[cfg(target_os = "windows")]
            W => Self::W,
            #[cfg(target_os = "windows")]
            X => Self::X,
            #[cfg(target_os = "windows")]
            Y => Self::Y,
            #[cfg(target_os = "windows")]
            Z => Self::Z,

            #[cfg(target_os = "windows")]
            AbntC1 => Self::AbntC1,
            #[cfg(target_os = "windows")]
            AbntC2 => Self::AbntC2,
            #[cfg(target_os = "windows")]
            Accept => Self::Accept,
            #[cfg(target_os = "windows")]
            Apps => Self::Apps,
            #[cfg(target_os = "windows")]
            Attn => Self::Attn,
            #[cfg(target_os = "windows")]
            BrowserBack => Self::BrowserBack,
            #[cfg(target_os = "windows")]
            BrowserFavorites => Self::BrowserFavorites,
            #[cfg(target_os = "windows")]
            BrowserForward => Self::BrowserForward,
            #[cfg(target_os = "windows")]
            BrowserHome => Self::BrowserHome,
            #[cfg(target_os = "windows")]
            BrowserRefresh => Self::BrowserRefresh,
            #[cfg(target_os = "windows")]
            BrowserSearch => Self::BrowserSearch,
            #[cfg(target_os = "windows")]
            BrowserStop => Self::BrowserStop,
            #[cfg(target_os = "windows")]
            Convert => Self::Convert,
            #[cfg(target_os = "windows")]
            Crsel => Self::Crsel,
            #[cfg(target_os = "windows")]
            DBEAlphanumeric => Self::DBEAlphanumeric,
            #[cfg(target_os = "windows")]
            DBECodeinput => Self::DBECodeinput,
            #[cfg(target_os = "windows")]
            DBEDetermineString => Self::DBEDetermineString,
            #[cfg(target_os = "windows")]
            DBEEnterDLGConversionMode => Self::DBEEnterDLGConversionMode,
            #[cfg(target_os = "windows")]
            DBEEnterIMEConfigMode => Self::DBEEnterIMEConfigMode,
            #[cfg(target_os = "windows")]
            DBEEnterWordRegisterMode => Self::DBEEnterWordRegisterMode,
            #[cfg(target_os = "windows")]
            DBEFlushString => Self::DBEFlushString,
            #[cfg(target_os = "windows")]
            DBEHiragana => Self::DBEHiragana,
            #[cfg(target_os = "windows")]
            DBEKatakana => Self::DBEKatakana,
            #[cfg(target_os = "windows")]
            DBENoCodepoint => Self::DBENoCodepoint,
            #[cfg(target_os = "windows")]
            DBENoRoman => Self::DBENoRoman,
            #[cfg(target_os = "windows")]
            DBERoman => Self::DBERoman,
            #[cfg(target_os = "windows")]
            DBESBCSChar => Self::DBESBCSChar,
            #[cfg(target_os = "windows")]
            DBESChar => Self::DBESChar,
            #[cfg(target_os = "windows")]
            Ereof => Self::Ereof,
            #[cfg(target_os = "windows")]
            Exsel => Self::Exsel,
            #[cfg(target_os = "windows")]
            Final => Self::Final,
            #[cfg(target_os = "windows")]
            GamepadA => Self::GamepadA,
            #[cfg(target_os = "windows")]
            GamepadB => Self::GamepadB,
            #[cfg(target_os = "windows")]
            GamepadDPadDown => Self::GamepadDPadDown,
            #[cfg(target_os = "windows")]
            GamepadDPadLeft => Self::GamepadDPadLeft,
            #[cfg(target_os = "windows")]
            GamepadDPadRight => Self::GamepadDPadRight,
            #[cfg(target_os = "windows")]
            GamepadDPadUp => Self::GamepadDPadUp,
            #[cfg(target_os = "windows")]
            GamepadLeftShoulder => Self::GamepadLeftShoulder,
            #[cfg(target_os = "windows")]
            GamepadLeftThumbstickButton => Self::GamepadLeftThumbstickButton,
            #[cfg(target_os = "windows")]
            GamepadLeftThumbstickDown => Self::GamepadLeftThumbstickDown,
            #[cfg(target_os = "windows")]
            GamepadLeftThumbstickLeft => Self::GamepadLeftThumbstickLeft,
            #[cfg(target_os = "windows")]
            GamepadLeftThumbstickRight => Self::GamepadLeftThumbstickRight,
            #[cfg(target_os = "windows")]
            GamepadLeftThumbstickUp => Self::GamepadLeftThumbstickUp,
            #[cfg(target_os = "windows")]
            GamepadLeftTrigger => Self::GamepadLeftTrigger,
            #[cfg(target_os = "windows")]
            GamepadMenu => Self::GamepadMenu,
            #[cfg(target_os = "windows")]
            GamepadRightShoulder => Self::GamepadRightShoulder,
            #[cfg(target_os = "windows")]
            GamepadRightThumbstickButton => Self::GamepadRightThumbstickButton,
            #[cfg(target_os = "windows")]
            GamepadRightThumbstickDown => Self::GamepadRightThumbstickDown,
            #[cfg(target_os = "windows")]
            GamepadRightThumbstickLeft => Self::GamepadRightThumbstickLeft,
            #[cfg(target_os = "windows")]
            GamepadRightThumbstickRight => Self::GamepadRightThumbstickRight,
            #[cfg(target_os = "windows")]
            GamepadRightThumbstickUp => Self::GamepadRightThumbstickUp,
            #[cfg(target_os = "windows")]
            GamepadRightTrigger => Self::GamepadRightTrigger,
            #[cfg(target_os = "windows")]
            GamepadView => Self::GamepadView,
            #[cfg(target_os = "windows")]
            GamepadX => Self::GamepadX,
            #[cfg(target_os = "windows")]
            GamepadY => Self::GamepadY,
            #[cfg(target_os = "windows")]
            Ico00 => Self::Ico00,
            #[cfg(target_os = "windows")]
            IcoClear => Self::IcoClear,
            #[cfg(target_os = "windows")]
            IcoHelp => Self::IcoHelp,
            #[cfg(target_os = "windows")]
            Hangeul => Self::Hangeul,
            #[cfg(target_os = "windows")]
            IMEOff => Self::IMEOff,
            #[cfg(target_os = "windows")]
            IMEOn => Self::IMEOn,
            #[cfg(target_os = "windows")]
            Junja => Self::Junja,
            #[cfg(target_os = "windows")]
            Kana => Self::Kana,
            #[cfg(target_os = "windows")]
            LaunchApp1 => Self::LaunchApp1,
            #[cfg(target_os = "windows")]
            LaunchApp2 => Self::LaunchApp2,
            #[cfg(target_os = "windows")]
            LaunchMail => Self::LaunchMail,
            #[cfg(target_os = "windows")]
            LaunchMediaSelect => Self::LaunchMediaSelect,
            #[cfg(target_os = "windows")]
            LButton => Self::LButton,
            #[cfg(target_os = "windows")]
            LWin => Self::LWin,
            #[cfg(target_os = "windows")]
            MButton => Self::MButton,
            #[cfg(target_os = "windows")]
            NavigationAccept => Self::NavigationAccept,
            #[cfg(target_os = "windows")]
            NavigationCancel => Self::NavigationCancel,
            #[cfg(target_os = "windows")]
            NavigationDown => Self::NavigationDown,
            #[cfg(target_os = "windows")]
            NavigationLeft => Self::NavigationLeft,
            #[cfg(target_os = "windows")]
            NavigationMenu => Self::NavigationMenu,
            #[cfg(target_os = "windows")]
            NavigationRight => Self::NavigationRight,
            #[cfg(target_os = "windows")]
            NavigationUp => Self::NavigationUp,
            #[cfg(target_os = "windows")]
            NavigationView => Self::NavigationView,
            #[cfg(target_os = "windows")]
            NoName => Self::NoName,
            #[cfg(target_os = "windows")]
            NonConvert => Self::NonConvert,
            #[cfg(target_os = "windows")]
            None => Self::None,
            #[cfg(target_os = "windows")]
            OEM1 => Self::OEM1,
            #[cfg(target_os = "windows")]
            OEM102 => Self::OEM102,
            #[cfg(target_os = "windows")]
            OEM2 => Self::OEM2,
            #[cfg(target_os = "windows")]
            OEM3 => Self::OEM3,
            #[cfg(target_os = "windows")]
            OEM4 => Self::OEM4,
            #[cfg(target_os = "windows")]
            OEM5 => Self::OEM5,
            #[cfg(target_os = "windows")]
            OEM6 => Self::OEM6,
            #[cfg(target_os = "windows")]
            OEM7 => Self::OEM7,
            #[cfg(target_os = "windows")]
            OEM8 => Self::OEM8,
            #[cfg(target_os = "windows")]
            OEMAttn => Self::OEMAttn,
            #[cfg(target_os = "windows")]
            OEMAuto => Self::OEMAuto,
            #[cfg(target_os = "windows")]
            OEMAx => Self::OEMAx,
            #[cfg(target_os = "windows")]
            OEMBacktab => Self::OEMBacktab,
            #[cfg(target_os = "windows")]
            OEMClear => Self::OEMClear,
            #[cfg(target_os = "windows")]
            OEMComma => Self::OEMComma,
            #[cfg(target_os = "windows")]
            OEMCopy => Self::OEMCopy,
            #[cfg(target_os = "windows")]
            OEMCusel => Self::OEMCusel,
            #[cfg(target_os = "windows")]
            OEMEnlw => Self::OEMEnlw,
            #[cfg(target_os = "windows")]
            OEMFinish => Self::OEMFinish,
            #[cfg(target_os = "windows")]
            OEMFJJisho => Self::OEMFJJisho,
            #[cfg(target_os = "windows")]
            OEMFJLoya => Self::OEMFJLoya,
            #[cfg(target_os = "windows")]
            OEMFJMasshou => Self::OEMFJMasshou,
            #[cfg(target_os = "windows")]
            OEMFJRoya => Self::OEMFJRoya,
            #[cfg(target_os = "windows")]
            OEMFJTouroku => Self::OEMFJTouroku,
            #[cfg(target_os = "windows")]
            OEMJump => Self::OEMJump,
            #[cfg(target_os = "windows")]
            OEMMinus => Self::OEMMinus,
            #[cfg(target_os = "windows")]
            OEMNECEqual => Self::OEMNECEqual,
            #[cfg(target_os = "windows")]
            OEMPA1 => Self::OEMPA1,
            #[cfg(target_os = "windows")]
            OEMPA2 => Self::OEMPA2,
            #[cfg(target_os = "windows")]
            OEMPA3 => Self::OEMPA3,
            #[cfg(target_os = "windows")]
            OEMPeriod => Self::OEMPeriod,
            #[cfg(target_os = "windows")]
            OEMPlus => Self::OEMPlus,
            #[cfg(target_os = "windows")]
            OEMReset => Self::OEMReset,
            #[cfg(target_os = "windows")]
            OEMWsctrl => Self::OEMWsctrl,
            #[cfg(target_os = "windows")]
            PA1 => Self::PA1,
            #[cfg(target_os = "windows")]
            Packet => Self::Packet,
            #[cfg(target_os = "windows")]
            RMenu => Self::RMenu,
            #[cfg(target_os = "windows")]
            RWin => Self::RWin,
            #[cfg(target_os = "windows")]
            Scroll => Self::Scroll,
            #[cfg(target_os = "windows")]
            Play => Self::Play,
            #[cfg(target_os = "windows")]
            Processkey => Self::Processkey,
            #[cfg(target_os = "windows")]
            RButton => Self::RButton,
            #[cfg(target_os = "windows")]
            Separator => Self::Separator,
            #[cfg(target_os = "windows")]
            Sleep => Self::Sleep,
            #[cfg(target_os = "windows")]
            XButton1 => Self::XButton1,
            #[cfg(target_os = "windows")]
            XButton2 => Self::XButton2,
            #[cfg(target_os = "windows")]
            Zoom => Self::Zoom,

            #[cfg(target_os = "linux")]
            Num0 => Self::Unicode('0'),
            #[cfg(target_os = "linux")]
            Num1 => Self::Unicode('1'),
            #[cfg(target_os = "linux")]
            Num2 => Self::Unicode('2'),
            #[cfg(target_os = "linux")]
            Num3 => Self::Unicode('3'),
            #[cfg(target_os = "linux")]
            Num4 => Self::Unicode('4'),
            #[cfg(target_os = "linux")]
            Num5 => Self::Unicode('5'),
            #[cfg(target_os = "linux")]
            Num6 => Self::Unicode('6'),
            #[cfg(target_os = "linux")]
            Num7 => Self::Unicode('7'),
            #[cfg(target_os = "linux")]
            Num8 => Self::Unicode('8'),
            #[cfg(target_os = "linux")]
            Num9 => Self::Unicode('9'),

            #[cfg(target_os = "linux")]
            A => Self::Unicode('A'),
            #[cfg(target_os = "linux")]
            B => Self::Unicode('B'),
            #[cfg(target_os = "linux")]
            C => Self::Unicode('C'),
            #[cfg(target_os = "linux")]
            D => Self::Unicode('D'),
            #[cfg(target_os = "linux")]
            E => Self::Unicode('E'),
            #[cfg(target_os = "linux")]
            F => Self::Unicode('F'),
            #[cfg(target_os = "linux")]
            G => Self::Unicode('G'),
            #[cfg(target_os = "linux")]
            H => Self::Unicode('H'),
            #[cfg(target_os = "linux")]
            I => Self::Unicode('I'),
            #[cfg(target_os = "linux")]
            J => Self::Unicode('J'),
            #[cfg(target_os = "linux")]
            K => Self::Unicode('K'),
            #[cfg(target_os = "linux")]
            L => Self::Unicode('L'),
            #[cfg(target_os = "linux")]
            M => Self::Unicode('M'),
            #[cfg(target_os = "linux")]
            N => Self::Unicode('N'),
            #[cfg(target_os = "linux")]
            O => Self::Unicode('O'),
            #[cfg(target_os = "linux")]
            P => Self::Unicode('P'),
            #[cfg(target_os = "linux")]
            Q => Self::Unicode('Q'),
            #[cfg(target_os = "linux")]
            R => Self::Unicode('R'),
            #[cfg(target_os = "linux")]
            S => Self::Unicode('S'),
            #[cfg(target_os = "linux")]
            T => Self::Unicode('T'),
            #[cfg(target_os = "linux")]
            U => Self::Unicode('U'),
            #[cfg(target_os = "linux")]
            V => Self::Unicode('V'),
            #[cfg(target_os = "linux")]
            W => Self::Unicode('W'),
            #[cfg(target_os = "linux")]
            X => Self::Unicode('X'),
            #[cfg(target_os = "linux")]
            Y => Self::Unicode('Y'),
            #[cfg(target_os = "linux")]
            Z => Self::Unicode('Z'),

            #[cfg(target_os = "linux")]
            Break => Self::Break,
            #[cfg(target_os = "linux")]
            Begin => Self::Begin,
            #[cfg(target_os = "linux")]
            Find => Self::Find,
            #[cfg(target_os = "linux")]
            Linefeed => Self::Linefeed,
            #[cfg(target_os = "linux")]
            Redo => Self::Redo,
            #[cfg(target_os = "linux")]
            ScrollLock => Self::ScrollLock,
            #[cfg(target_os = "linux")]
            ScriptSwitch => Self::ScriptSwitch,
            #[cfg(target_os = "linux")]
            ShiftLock => Self::ShiftLock,
            #[cfg(target_os = "linux")]
            SysReq => Self::SysReq,
            #[cfg(target_os = "linux")]
            Undo => Self::Undo,
            #[cfg(target_os = "linux")]
            MicMute => Self::MicMute,

            #[cfg(target_os = "linux")]
            F25 => Self::F25,
            #[cfg(target_os = "linux")]
            F26 => Self::F26,
            #[cfg(target_os = "linux")]
            F27 => Self::F27,
            #[cfg(target_os = "linux")]
            F28 => Self::F28,
            #[cfg(target_os = "linux")]
            F29 => Self::F29,
            #[cfg(target_os = "linux")]
            F30 => Self::F30,
            #[cfg(target_os = "linux")]
            F31 => Self::F31,
            #[cfg(target_os = "linux")]
            F32 => Self::F32,
            #[cfg(target_os = "linux")]
            F33 => Self::F33,
            #[cfg(target_os = "linux")]
            F34 => Self::F34,
            #[cfg(target_os = "linux")]
            F35 => Self::F35,

            #[cfg(target_os = "linux")]
            AbntC1
            | AbntC2
            | Accept
            | Apps
            | Attn
            | BrowserBack
            | BrowserFavorites
            | BrowserForward
            | BrowserHome
            | BrowserRefresh
            | BrowserSearch
            | BrowserStop
            | Convert
            | Crsel
            | DBEAlphanumeric
            | DBECodeinput
            | DBEDetermineString
            | DBEEnterDLGConversionMode
            | DBEEnterIMEConfigMode
            | DBEEnterWordRegisterMode
            | DBEFlushString
            | DBEHiragana
            | DBEKatakana
            | DBENoCodepoint
            | DBENoRoman
            | DBERoman
            | DBESBCSChar
            | DBESChar
            | Ereof
            | Exsel
            | Final
            | GamepadA
            | GamepadB
            | GamepadDPadDown
            | GamepadDPadLeft
            | GamepadDPadRight
            | GamepadDPadUp
            | GamepadLeftShoulder
            | GamepadLeftThumbstickButton
            | GamepadLeftThumbstickDown
            | GamepadLeftThumbstickLeft
            | GamepadLeftThumbstickRight
            | GamepadLeftThumbstickUp
            | GamepadLeftTrigger
            | GamepadMenu
            | GamepadRightShoulder
            | GamepadRightThumbstickButton
            | GamepadRightThumbstickDown
            | GamepadRightThumbstickLeft
            | GamepadRightThumbstickRight
            | GamepadRightThumbstickUp
            | GamepadRightTrigger
            | GamepadView
            | GamepadX
            | GamepadY
            | Ico00
            | IcoClear
            | IcoHelp
            | Hangeul
            | IMEOff
            | IMEOn
            | Junja
            | Kana
            | LaunchApp1
            | LaunchApp2
            | LaunchMail
            | LaunchMediaSelect
            | LButton
            | LWin
            | MButton
            | NavigationAccept
            | NavigationCancel
            | NavigationDown
            | NavigationLeft
            | NavigationMenu
            | NavigationRight
            | NavigationUp
            | NavigationView
            | NoName
            | NonConvert
            | None
            | OEM1
            | OEM102
            | OEM2
            | OEM3
            | OEM4
            | OEM5
            | OEM6
            | OEM7
            | OEM8
            | OEMAttn
            | OEMAuto
            | OEMAx
            | OEMBacktab
            | OEMClear
            | OEMComma
            | OEMCopy
            | OEMCusel
            | OEMEnlw
            | OEMFinish
            | OEMFJJisho
            | OEMFJLoya
            | OEMFJMasshou
            | OEMFJRoya
            | OEMFJTouroku
            | OEMJump
            | OEMMinus
            | OEMNECEqual
            | OEMPA1
            | OEMPA2
            | OEMPA3
            | OEMPeriod
            | OEMPlus
            | OEMReset
            | OEMWsctrl
            | PA1
            | Packet
            | RMenu
            | RWin
            | Scroll
            | Play
            | Processkey
            | RButton
            | Separator
            | Sleep
            | XButton1
            | XButton2
            | Zoom => return Err(KeyError::Unsupported),

            #[cfg(target_os = "windows")]
            Break | Begin | Find | Linefeed | Redo | ScrollLock | ScriptSwitch | ShiftLock
            | SysReq | Undo | MicMute | F25 | F26 | F27 | F28 | F29 | F30 | F31 | F32 | F33
            | F34 | F35 => return Err(KeyError::Unsupported),
        })
    }
}
