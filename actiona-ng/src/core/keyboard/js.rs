use std::{collections::HashSet, str::FromStr, sync::Arc};

use derive_more::Display;
use enigo::Key;
use macros::{FromJsObject, FromSerde, IntoSerde};
use rquickjs::{
    Class, Ctx, Exception, FromJs, IntoJs, JsLifetime, Object, Promise, Result, Value,
    class::{JsClass, Readable, Trace, Tracer},
    function::Constructor,
    prelude::Opt,
};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString};
use tracing::instrument;

use crate::{
    IntoJsResult,
    core::js::{abort_controller::JsAbortSignal, classes::SingletonClass, task::task_with_token},
    runtime::Runtime,
};

impl<'js> Trace<'js> for super::Keyboard {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

/// Direction for key press/release actions.
///
/// ```ts
/// // Press and hold a key
/// await keyboard.key(Key.Shift, Direction.Press);
/// // Release it
/// await keyboard.key(Key.Shift, Direction.Release);
///
/// // Press and release in one action
/// await keyboard.key(Key.Return, Direction.Click);
/// ```
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    Eq,
    FromSerde,
    Hash,
    IntoSerde,
    PartialEq,
    Serialize,
)]
pub enum JsDirection {
    // TODO: same as mouse?
    Press,
    Release,
    Click,
}

impl From<JsDirection> for enigo::Direction {
    fn from(value: JsDirection) -> Self {
        use JsDirection::*;

        match value {
            Press => Self::Press,
            Release => Self::Release,
            Click => Self::Click,
        }
    }
}

/// Controls keyboard input: typing text, pressing keys, and waiting for key combinations.
///
/// ```ts
/// // Type text
/// await keyboard.text("Hello, world!");
/// ```
///
/// ```ts
/// // Press a key combination (Ctrl+C)
/// await keyboard.key(Key.Control, Direction.Press);
/// await keyboard.key("c", Direction.Click);
/// await keyboard.key(Key.Control, Direction.Release);
/// ```
///
/// ```ts
/// // Wait for a key combination
/// await keyboard.waitForKeys([Key.Control, Key.Alt, "q"]);
/// ```
/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Keyboard")]
pub struct JsKeyboard {
    inner: Arc<super::Keyboard>,
}

impl<'js> Trace<'js> for JsKeyboard {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl SingletonClass<'_> for JsKeyboard {
    fn register_dependencies(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
        // Register the Key class first, then add enum variants as static properties.
        // Both JsKey and JsStandardKey use the name "Key", so we must define the class
        // first and then set enum properties on its constructor object.
        Class::<JsKey>::define(&ctx.globals())?;

        let key_obj: Object = ctx.globals().get("Key")?;
        for v in <JsStandardKey as strum::IntoEnumIterator>::iter() {
            let name = serde_plain::to_string(&v).unwrap();
            key_obj.set(&name, name.clone())?;
        }

        Ok(())
    }
}

impl JsKeyboard {
    /// @skip
    #[instrument(skip_all)]
    pub fn new(runtime: Arc<Runtime>) -> super::Result<Self> {
        Ok(Self {
            inner: Arc::new(super::Keyboard::new(runtime)?),
        })
    }
}

/// Options for waiting for key combinations.
///
/// ```ts
/// // Wait for exactly Ctrl+S and no other keys
/// await keyboard.waitForKeys([Key.Control, "s"], { exclusive: true });
/// ```
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsWaitForKeysOptions {
    /// Wait for exactly these keys and no other
    /// @default `false`
    pub exclusive: bool,

    /// Abort signal to cancel the wait.
    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsKeyboard {
    /// Types the given text string using simulated key events.
    pub async fn text(&self, ctx: Ctx<'_>, text: String) -> Result<()> {
        self.inner.text(&text).into_js_result(&ctx)?;

        Ok(())
    }

    /// Presses, releases, or clicks a key.
    ///
    /// Accepts a `Key` constant, a single character string, or a raw keycode number.
    /// @param key: Key | string | number
    /// @param direction: Direction
    pub async fn key(&self, ctx: Ctx<'_>, key: JsKey, direction: JsDirection) -> Result<()> {
        let key = key.try_into().map_err(|_| {
            Exception::throw_message(
                &ctx,
                &format!("key {key} is not supported on this platform"),
            )
        })?;

        self.inner.key(key, direction.into()).into_js_result(&ctx)?;

        Ok(())
    }

    /// Sends a raw keycode event. Use this for keys not covered by the `Key` enum.
    pub async fn raw(&self, ctx: Ctx<'_>, keycode: u16, direction: JsDirection) -> Result<()> {
        self.inner
            .raw(keycode, direction.into())
            .into_js_result(&ctx)?;

        Ok(())
    }

    /// Returns whether a key is currently pressed.
    /// @param key: Key | string | number
    pub async fn is_key_pressed(&self, ctx: Ctx<'_>, key: JsKey) -> Result<bool> {
        let key = key.try_into().map_err(|_| {
            Exception::throw_message(
                &ctx,
                &format!("key {key} is not supported on this platform"),
            )
        })?;

        self.inner.is_key_pressed(key).await.into_js_result(&ctx)
    }

    /// Waits until the specified keys are all pressed simultaneously.
    ///
    /// ```ts
    /// await keyboard.waitForKeys([Key.Control, "s"]);
    /// ```
    ///
    /// ```ts
    /// // Wait for exactly these keys and no others, with abort support
    /// const controller = new AbortController();
    /// await keyboard.waitForKeys([Key.Control, Key.Alt, Key.Delete], {
    ///   exclusive: true,
    ///   signal: controller.signal
    /// });
    /// ```
    /// @param keys: (Key | string | number)[]
    /// @returns Task<void>
    pub fn wait_for_keys<'js>(
        &self,
        ctx: Ctx<'js>,
        keys: Vec<JsKey>,
        options: Opt<JsWaitForKeysOptions>,
    ) -> Result<Promise<'js>> {
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let keys = keys
            .into_iter()
            .map(|key| {
                Key::try_from(key).map_err(|_| {
                    Exception::throw_message(&ctx, &format!("key {key} is not supported"))
                })
            })
            .collect::<Result<HashSet<_>>>()?;
        let local_keyboard = self.inner.clone();

        task_with_token(ctx, signal, async move |ctx, token| {
            local_keyboard
                .wait_for_keys(&keys, options.exclusive, token)
                .await
                .into_js_result(&ctx)
        })
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    FromSerde,
    Hash,
    IntoSerde,
    PartialEq,
    Serialize,
)]
/// Standard keyboard keys.
///
/// Use as constants on the `Key` class. You can also pass a single character string
/// or a raw keycode number wherever a `Key` is expected.
///
/// ```ts
/// await keyboard.key(Key.Return, Direction.Click);
/// await keyboard.key("a", Direction.Click);
/// ```
#[serde(rename = "Key")]
/// @rename Key
pub enum JsStandardKey {
    /// Top-row digit '0' key (not numpad)
    Num0,
    /// Top-row digit '1' key (not numpad)
    Num1,
    /// Top-row digit '2' key (not numpad)
    Num2,
    /// Top-row digit '3' key (not numpad)
    Num3,
    /// Top-row digit '4' key (not numpad)
    Num4,
    /// Top-row digit '5' key (not numpad)
    Num5,
    /// Top-row digit '6' key (not numpad)
    Num6,
    /// Top-row digit '7' key (not numpad)
    Num7,
    /// Top-row digit '8' key (not numpad)
    Num8,
    /// Top-row digit '9' key (not numpad)
    Num9,
    /// Letter key 'A'
    A,
    /// Letter key 'B'
    B,
    /// Letter key 'C'
    C,
    /// Letter key 'D'
    D,
    /// Letter key 'E'
    E,
    /// Letter key 'F'
    F,
    /// Letter key 'G'
    G,
    /// Letter key 'H'
    H,
    /// Letter key 'I'
    I,
    /// Letter key 'J'
    J,
    /// Letter key 'K'
    K,
    /// Letter key 'L'
    L,
    /// Letter key 'M'
    M,
    /// Letter key 'N'
    N,
    /// Letter key 'O'
    O,
    /// Letter key 'P'
    P,
    /// Letter key 'Q'
    Q,
    /// Letter key 'R'
    R,
    /// Letter key 'S'
    S,
    /// Letter key 'T'
    T,
    /// Letter key 'U'
    U,
    /// Letter key 'V'
    V,
    /// Letter key 'W'
    W,
    /// Letter key 'X'
    X,
    /// Letter key 'Y'
    Y,
    /// Letter key 'Z'
    Z,
    /// Brazilian ABNT keyboard key C1
    /// @platforms =windows
    AbntC1,
    /// Brazilian ABNT keyboard key C2
    /// @platforms =windows
    AbntC2,
    /// IME “Accept” / commit conversion
    /// @platforms =windows
    Accept,
    /// Numpad '+' (addition) key
    Add,
    /// Alt (Alternate) modifier key
    Alt,
    /// Application/Menu key
    /// @platforms =windows
    Apps,
    /// Attention key (legacy/rare)
    /// @platforms =windows
    Attention,
    /// Backspace / Delete-previous-character
    Backspace,
    /// Break key (X11/Linux)
    /// @platforms =linux
    Break,
    /// Begin key
    /// @platforms =linux
    Begin,
    /// Browser Back
    /// @platforms =windows
    BrowserBack,
    /// Browser Favorites
    /// @platforms =windows
    BrowserFavorites,
    /// Browser Forward
    /// @platforms =windows
    BrowserForward,
    /// Browser Home
    /// @platforms =windows
    BrowserHome,
    /// Browser Refresh
    /// @platforms =windows
    BrowserRefresh,
    /// Browser Search
    /// @platforms =windows
    BrowserSearch,
    /// Browser Stop
    /// @platforms =windows
    BrowserStop,
    /// Cancel key (legacy)
    Cancel,
    /// Caps Lock toggle
    CapsLock,
    /// Clear key
    Clear,
    /// Control (Ctrl) modifier key
    Control,
    /// IME Convert (start/confirm conversion)
    /// @platforms =windows
    Convert,
    /// Cursor Select (CRSel)
    /// @platforms =windows
    CursorSelect,
    /// IME: switch to alphanumeric
    /// @platforms =windows
    DBEAlphanumeric,
    /// IME: code input mode
    /// @platforms =windows
    DBECodeinput,
    /// IME: determine string
    /// @platforms =windows
    DBEDetermineString,
    /// IME: enter dialog conversion mode
    /// @platforms =windows
    DBEEnterDLGConversionMode,
    /// IME: open configuration
    /// @platforms =windows
    DBEEnterIMEConfigMode,
    /// IME: word register mode
    /// @platforms =windows
    DBEEnterWordRegisterMode,
    /// IME: flush/reset composition string
    /// @platforms =windows
    DBEFlushString,
    /// IME: Hiragana
    /// @platforms =windows
    DBEHiragana,
    /// IME: Katakana
    /// @platforms =windows
    DBEKatakana,
    /// IME: no code point
    /// @platforms =windows
    DBENoCodepoint,
    /// IME: no roman
    /// @platforms =windows
    DBENoRoman,
    /// IME: Roman
    /// @platforms =windows
    DBERoman,
    /// IME: SBCS character
    /// @platforms =windows
    DBESBCSChar,
    /// IME: SBCS/Special char
    /// @platforms =windows
    DBESChar,
    /// Numpad decimal point '.'
    Decimal,
    /// Delete / Forward delete
    Delete,
    /// Numpad divide '/'
    Divide,
    /// Arrow: Down
    DownArrow,
    /// End key
    End,
    /// Erase EOF
    /// @platforms =windows
    Ereof,
    /// Escape key
    Escape,
    /// Execute key
    Execute,
    /// Extend Selection (ExSel)
    /// @platforms =windows
    Exsel,
    /// Function key F1
    F1,
    /// Function key F2
    F2,
    /// Function key F3
    F3,
    /// Function key F4
    F4,
    /// Function key F5
    F5,
    /// Function key F6
    F6,
    /// Function key F7
    F7,
    /// Function key F8
    F8,
    /// Function key F9
    F9,
    /// Function key F10
    F10,
    /// Function key F11
    F11,
    /// Function key F12
    F12,
    /// Function key F13
    F13,
    /// Function key F14
    F14,
    /// Function key F15
    F15,
    /// Function key F16
    F16,
    /// Function key F17
    F17,
    /// Function key F18
    F18,
    /// Function key F19
    F19,
    /// Function key F20
    F20,
    /// Function key F21
    F21,
    /// Function key F22
    F22,
    /// Function key F23
    F23,
    /// Function key F24
    F24,
    /// Function key F25
    /// @platforms =linux
    F25,
    /// Function key F26
    /// @platforms =linux
    F26,
    /// Function key F27
    /// @platforms =linux
    F27,
    /// Function key F28
    /// @platforms =linux
    F28,
    /// Function key F29
    /// @platforms =linux
    F29,
    /// Function key F30
    /// @platforms =linux
    F30,
    /// Function key F31
    /// @platforms =linux
    F31,
    /// Function key F32
    /// @platforms =linux
    F32,
    /// Function key F33
    /// @platforms =linux
    F33,
    /// Function key F34
    /// @platforms =linux
    F34,
    /// Function key F35
    /// @platforms =linux
    F35,
    /// IME Final (end conversion)
    /// @platforms =windows
    Final,
    /// Find key
    /// @platforms =linux
    Find,
    /// Gamepad: A button
    /// @platforms =windows
    GamepadA,
    /// Gamepad: B button
    /// @platforms =windows
    GamepadB,
    /// Gamepad: D-Pad Down
    /// @platforms =windows
    GamepadDPadDown,
    /// Gamepad: D-Pad Left
    /// @platforms =windows
    GamepadDPadLeft,
    /// Gamepad: D-Pad Right
    /// @platforms =windows
    GamepadDPadRight,
    /// Gamepad: D-Pad Up
    /// @platforms =windows
    GamepadDPadUp,
    /// Gamepad: Left shoulder (L1)
    /// @platforms =windows
    GamepadLeftShoulder,
    /// Gamepad: Left thumbstick button (L3)
    /// @platforms =windows
    GamepadLeftThumbstickButton,
    /// Gamepad: Left thumbstick down
    /// @platforms =windows
    GamepadLeftThumbstickDown,
    /// Gamepad: Left thumbstick left
    /// @platforms =windows
    GamepadLeftThumbstickLeft,
    /// Gamepad: Left thumbstick right
    /// @platforms =windows
    GamepadLeftThumbstickRight,
    /// Gamepad: Left thumbstick up
    /// @platforms =windows
    GamepadLeftThumbstickUp,
    /// Gamepad: Left trigger (L2)
    /// @platforms =windows
    GamepadLeftTrigger,
    /// Gamepad: Menu / Start
    /// @platforms =windows
    GamepadMenu,
    /// Gamepad: Right shoulder (R1)
    /// @platforms =windows
    GamepadRightShoulder,
    /// Gamepad: Right thumbstick button (R3)
    /// @platforms =windows
    GamepadRightThumbstickButton,
    /// Gamepad: Right thumbstick down
    /// @platforms =windows
    GamepadRightThumbstickDown,
    /// Gamepad: Right thumbstick left
    /// @platforms =windows
    GamepadRightThumbstickLeft,
    /// Gamepad: Right thumbstick right
    /// @platforms =windows
    GamepadRightThumbstickRight,
    /// Gamepad: Right thumbstick up
    /// @platforms =windows
    GamepadRightThumbstickUp,
    /// Gamepad: Right trigger (R2)
    /// @platforms =windows
    GamepadRightTrigger,
    /// Gamepad: View / Back
    /// @platforms =windows
    GamepadView,
    /// Gamepad: X button
    /// @platforms =windows
    GamepadX,
    /// Gamepad: Y button
    /// @platforms =windows
    GamepadY,
    /// Hangeul toggle (Korean layout)
    /// @platforms =windows
    Hangeul,
    /// Hangul toggle (Korean layout)
    Hangul,
    /// Hanja toggle (Chinese characters on Korean layout)
    Hanja,
    /// Help key
    Help,
    /// Home key
    Home,
    /// ICO (legacy) key 00
    /// @platforms =windows
    Ico00,
    /// ICO (legacy) Clear
    /// @platforms =windows
    IcoClear,
    /// ICO (legacy) Help
    /// @platforms =windows
    IcoHelp,
    /// IME Off (disable IME)
    /// @platforms =windows
    IMEOff,
    /// IME On (enable IME)
    /// @platforms =windows
    IMEOn,
    /// Insert key
    Insert,
    /// IME: Junja mode
    /// @platforms =windows
    Junja,
    /// IME: Kana mode
    /// @platforms =windows
    Kana,
    /// Kanji toggle (Japanese layout)
    Kanji,
    /// Launch application 1
    /// @platforms =windows
    LaunchApp1,
    /// Launch application 2
    /// @platforms =windows
    LaunchApp2,
    /// Launch default mail client
    /// @platforms =windows
    LaunchMail,
    /// Launch media selector
    /// @platforms =windows
    LaunchMediaSelect,
    /// Left Control
    LeftControl,
    /// Arrow: Left
    LeftArrow,
    /// Line Feed key
    /// @platforms =linux
    Linefeed,
    /// Left Alt/Menu
    LeftAlt,
    /// Left Shift
    LeftShift,
    /// Left Windows / Super key
    /// @platforms =windows
    LeftWindows,
    /// Next media track
    MediaNextTrack,
    /// Play/Pause media
    MediaPlayPause,
    /// Previous media track
    MediaPrevTrack,
    /// Stop media
    MediaStop,
    /// Meta key (also known as "windows", "super", and "command")
    Meta,
    /// IME mode change
    ModeChange,
    /// Numpad multiply '*'
    Multiply,
    /// Navigation: Accept/OK (UWP)
    /// @platforms =windows
    NavigationAccept,
    /// Navigation: Cancel/Back (UWP)
    /// @platforms =windows
    NavigationCancel,
    /// Navigation: Down (UWP)
    /// @platforms =windows
    NavigationDown,
    /// Navigation: Left (UWP)
    /// @platforms =windows
    NavigationLeft,
    /// Navigation: Menu (UWP)
    /// @platforms =windows
    NavigationMenu,
    /// Navigation: Right (UWP)
    /// @platforms =windows
    NavigationRight,
    /// Navigation: Up (UWP)
    /// @platforms =windows
    NavigationUp,
    /// Navigation: View (UWP)
    /// @platforms =windows
    NavigationView,
    /// NoName key (reserved)
    /// @platforms =windows
    NoName,
    /// IME Non-Convert (cancel conversion)
    /// @platforms =windows
    NonConvert,
    /// Placeholder "no key"
    /// @platforms =windows
    None,
    /// Num Lock toggle
    Numlock,
    /// Numpad digit '0'
    Numpad0,
    /// Numpad digit '1'
    Numpad1,
    /// Numpad digit '2'
    Numpad2,
    /// Numpad digit '3'
    Numpad3,
    /// Numpad digit '4'
    Numpad4,
    /// Numpad digit '5'
    Numpad5,
    /// Numpad digit '6'
    Numpad6,
    /// Numpad digit '7'
    Numpad7,
    /// Numpad digit '8'
    Numpad8,
    /// Numpad digit '9'
    Numpad9,
    /// Numpad Enter
    NumpadEnter,
    /// OEM specific key 1
    /// @platforms =windows
    OEM1,
    /// OEM specific key 102 (angle bracket/pipe on some layouts)
    /// @platforms =windows
    OEM102,
    /// OEM specific key 2
    /// @platforms =windows
    OEM2,
    /// OEM specific key 3 (backtick/tilde on some layouts)
    /// @platforms =windows
    OEM3,
    /// OEM specific key 4 (left bracket on some layouts)
    /// @platforms =windows
    OEM4,
    /// OEM specific key 5 (right bracket on some layouts)
    /// @platforms =windows
    OEM5,
    /// OEM specific key 6 (semicolon on some layouts)
    /// @platforms =windows
    OEM6,
    /// OEM specific key 7 (quote on some layouts)
    /// @platforms =windows
    OEM7,
    /// OEM specific key 8
    /// @platforms =windows
    OEM8,
    /// OEM Attention
    /// @platforms =windows
    OEMAttn,
    /// OEM Auto
    /// @platforms =windows
    OEMAuto,
    /// OEM Ax
    /// @platforms =windows
    OEMAx,
    /// OEM Backtab (reverse Tab)
    /// @platforms =windows
    OEMBacktab,
    /// OEM Clear
    /// @platforms =windows
    OEMClear,
    /// OEM Comma ','
    /// @platforms =windows
    OEMComma,
    /// OEM Copy
    /// @platforms =windows
    OEMCopy,
    /// OEM Cusel
    /// @platforms =windows
    OEMCusel,
    /// OEM Enlw
    /// @platforms =windows
    OEMEnlw,
    /// OEM Finish
    /// @platforms =windows
    OEMFinish,
    /// OEM FJ Jisho (dictionary)
    /// @platforms =windows
    OEMFJJisho,
    /// OEM FJ Loya
    /// @platforms =windows
    OEMFJLoya,
    /// OEM FJ Masshou
    /// @platforms =windows
    OEMFJMasshou,
    /// OEM FJ Roya
    /// @platforms =windows
    OEMFJRoya,
    /// OEM FJ Touroku
    /// @platforms =windows
    OEMFJTouroku,
    /// OEM Jump
    /// @platforms =windows
    OEMJump,
    /// OEM Minus '-'
    /// @platforms =windows
    OEMMinus,
    /// OEM NEC Equal '='
    /// @platforms =windows
    OEMNECEqual,
    /// OEM PA1
    /// @platforms =windows
    OEMPA1,
    /// OEM PA2
    /// @platforms =windows
    OEMPA2,
    /// OEM PA3
    /// @platforms =windows
    OEMPA3,
    /// OEM Period '.'
    /// @platforms =windows
    OEMPeriod,
    /// OEM Plus '+'
    /// @platforms =windows
    OEMPlus,
    /// OEM Reset
    /// @platforms =windows
    OEMReset,
    /// OEM Wsctrl
    /// @platforms =windows
    OEMWsctrl,
    /// Same as Alt
    Option,
    /// PA1 key
    /// @platforms =windows
    PA1,
    /// Packet key (used to pass Unicode chars)
    /// @platforms =windows
    Packet,
    /// Page Down
    PageDown,
    /// Page Up
    PageUp,
    /// Pause key
    Pause,
    /// Media Play
    /// @platforms =windows
    Play,
    /// Screenshot
    PrintScreen,
    /// IME Process key
    /// @platforms =windows
    Processkey,
    /// Right Control
    RightControl,
    /// Redo
    /// @platforms =linux
    Redo,
    /// Enter / Return
    Return,
    /// Arrow: Right
    RightArrow,
    /// Right Alt/Menu
    /// @platforms =windows
    RightAlt,
    /// Right Shift
    RightShift,
    /// Right Windows / Super key
    /// @platforms =windows
    RightWindows,
    /// Scroll key (legacy)
    /// @platforms =windows
    Scroll,
    /// Scroll Lock
    /// @platforms =linux
    ScrollLock,
    /// Select key
    Select,
    /// Script switch
    /// @platforms =linux
    ScriptSwitch,
    /// Numpad separator (locale-dependent)
    /// @platforms =windows
    Separator,
    /// Shift modifier
    Shift,
    /// Shift Lock
    /// @platforms =linux
    ShiftLock,
    /// System Sleep
    /// @platforms =windows
    Sleep,
    /// Spacebar
    Space,
    /// Numpad '-' (subtract)
    Subtract,
    /// System Request (SysRq)
    /// @platforms =linux
    SysReq,
    /// Tab / focus next
    Tab,
    /// Undo
    /// @platforms =linux
    Undo,
    /// Arrow: Up
    UpArrow,
    /// Volume down
    VolumeDown,
    /// Volume mute
    VolumeMute,
    /// Volume up
    VolumeUp,
    /// Microphone mute
    /// @platforms =linux
    MicrophoneMute,
    /// Zoom key
    /// @platforms =windows
    Zoom,
}

/// @skip
#[derive(Clone, Copy, Debug, Display, Eq, Hash, JsLifetime, PartialEq)]
pub enum JsKey {
    Standard(JsStandardKey),
    Unicode(char),
    Other(u32),
}

impl<'js> Trace<'js> for JsKey {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> JsClass<'js> for JsKey {
    const NAME: &'static str = "Key";

    type Mutable = Readable;

    fn constructor(ctx: &Ctx<'js>) -> Result<Option<Constructor<'js>>> {
        Ok(Some(Constructor::new_class::<Self, _, _>(
            ctx.clone(),
            || Self::Other(0),
        )?))
    }
}

impl<'js> IntoJs<'js> for JsKey {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        Ok(match self {
            Self::Standard(standard_key) => standard_key.into_js(ctx)?,
            Self::Unicode(c) => c.to_string().into_js(ctx)?,
            Self::Other(n) => n.into_js(ctx)?,
        })
    }
}

impl<'js> FromJs<'js> for JsKey {
    fn from_js(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        Ok(
            if let Ok(standard_key) = JsStandardKey::from_js(ctx, value.clone()) {
                Self::Standard(standard_key)
            } else if let Some(string) = value.as_string() {
                let string = string.to_string()?;
                if let Ok(standard_key) = JsStandardKey::from_str(&string) {
                    return Ok(Self::Standard(standard_key));
                }
                if string.chars().count() != 1 {
                    return Err(Exception::throw_message(ctx, "invalid key name"));
                }
                let Some(c) = string.chars().next() else {
                    return Err(Exception::throw_message(ctx, "invalid key name"));
                };
                Self::Unicode(c)
            } else if let Some(n) = value.as_int() {
                let n = u32::try_from(n)
                    .map_err(|_| Exception::throw_message(ctx, "invalid key name"))?;
                Self::Other(n)
            } else {
                return Err(Exception::throw_message(ctx, "invalid key name"));
            },
        )
    }
}

pub enum KeyError {
    Unsupported,
}

impl TryFrom<JsKey> for enigo::Key {
    type Error = KeyError;

    fn try_from(value: JsKey) -> std::result::Result<Self, KeyError> {
        use JsKey::*;
        Ok(match value {
            Standard(js_standard_key) => js_standard_key.try_into()?,
            Unicode(c) => Self::Unicode(c),
            Other(v) => Self::Other(v),
        })
    }
}

impl TryFrom<JsStandardKey> for enigo::Key {
    type Error = KeyError;

    fn try_from(value: JsStandardKey) -> std::result::Result<Self, KeyError> {
        use JsStandardKey::*;
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
            LeftControl => Self::LControl,
            LeftArrow => Self::LeftArrow,
            LeftAlt => Self::LMenu,
            LeftShift => Self::LShift,
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
            PrintScreen => Self::PrintScr,
            RightControl => Self::RControl,
            Return => Self::Return,
            RightArrow => Self::RightArrow,
            RightShift => Self::RShift,
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
            Attention => Self::Attn,
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
            CursorSelect => Self::Crsel,
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
            LeftWindows => Self::LWin,
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
            RightAlt => Self::RMenu,
            #[cfg(target_os = "windows")]
            RightWindows => Self::RWin,
            #[cfg(target_os = "windows")]
            Scroll => Self::Scroll,
            #[cfg(target_os = "windows")]
            Play => Self::Play,
            #[cfg(target_os = "windows")]
            Processkey => Self::Processkey,
            #[cfg(target_os = "windows")]
            Separator => Self::Separator,
            #[cfg(target_os = "windows")]
            Sleep => Self::Sleep,
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
            MicrophoneMute => Self::MicMute,

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
            | Attention
            | BrowserBack
            | BrowserFavorites
            | BrowserForward
            | BrowserHome
            | BrowserRefresh
            | BrowserSearch
            | BrowserStop
            | Convert
            | CursorSelect
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
            | LeftWindows
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
            | RightAlt
            | RightWindows
            | Scroll
            | Play
            | Processkey
            | Separator
            | Sleep
            | Zoom => return Err(KeyError::Unsupported),

            #[cfg(target_os = "windows")]
            Break | Begin | Find | Linefeed | Redo | ScrollLock | ScriptSwitch | ShiftLock
            | SysReq | Undo | MicrophoneMute | F25 | F26 | F27 | F28 | F29 | F30 | F31 | F32
            | F33 | F34 | F35 => return Err(KeyError::Unsupported),
        })
    }
}

impl TryFrom<enigo::Key> for JsKey {
    type Error = KeyError;

    fn try_from(value: enigo::Key) -> std::result::Result<Self, KeyError> {
        use enigo::Key::*;
        Ok(match value {
            Unicode(c) => Self::Unicode(c),
            Other(v) => Self::Other(v),
            value => Self::Standard(value.try_into()?),
        })
    }
}

impl TryFrom<enigo::Key> for JsStandardKey {
    type Error = KeyError;

    fn try_from(value: enigo::Key) -> std::result::Result<Self, KeyError> {
        use enigo::Key::*;
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
            LControl => Self::LeftControl,
            LeftArrow => Self::LeftArrow,
            LMenu => Self::LeftAlt,
            LShift => Self::LeftShift,
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
            PrintScr => Self::PrintScreen,
            RControl => Self::RightControl,
            Return => Self::Return,
            RightArrow => Self::RightArrow,
            RShift => Self::RightShift,
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
            Attn => Self::Attention,
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
            Crsel => Self::CursorSelect,
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
            LWin => Self::LeftWindows,
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
            RMenu => Self::RightAlt,
            #[cfg(target_os = "windows")]
            RWin => Self::RightWindows,
            #[cfg(target_os = "windows")]
            Scroll => Self::Scroll,
            #[cfg(target_os = "windows")]
            Play => Self::Play,
            #[cfg(target_os = "windows")]
            Processkey => Self::Processkey,
            #[cfg(target_os = "windows")]
            Separator => Self::Separator,
            #[cfg(target_os = "windows")]
            Sleep => Self::Sleep,
            #[cfg(target_os = "windows")]
            Zoom => Self::Zoom,

            #[cfg(target_os = "linux")]
            Unicode('0') => Self::Num0,
            #[cfg(target_os = "linux")]
            Unicode('1') => Self::Num1,
            #[cfg(target_os = "linux")]
            Unicode('2') => Self::Num2,
            #[cfg(target_os = "linux")]
            Unicode('3') => Self::Num3,
            #[cfg(target_os = "linux")]
            Unicode('4') => Self::Num4,
            #[cfg(target_os = "linux")]
            Unicode('5') => Self::Num5,
            #[cfg(target_os = "linux")]
            Unicode('6') => Self::Num6,
            #[cfg(target_os = "linux")]
            Unicode('7') => Self::Num7,
            #[cfg(target_os = "linux")]
            Unicode('8') => Self::Num8,
            #[cfg(target_os = "linux")]
            Unicode('9') => Self::Num9,

            #[cfg(target_os = "linux")]
            Unicode('a') | Unicode('A') => Self::A,
            #[cfg(target_os = "linux")]
            Unicode('b') | Unicode('B') => Self::B,
            #[cfg(target_os = "linux")]
            Unicode('c') | Unicode('C') => Self::C,
            #[cfg(target_os = "linux")]
            Unicode('d') | Unicode('D') => Self::D,
            #[cfg(target_os = "linux")]
            Unicode('e') | Unicode('E') => Self::E,
            #[cfg(target_os = "linux")]
            Unicode('f') | Unicode('F') => Self::F,
            #[cfg(target_os = "linux")]
            Unicode('g') | Unicode('G') => Self::G,
            #[cfg(target_os = "linux")]
            Unicode('h') | Unicode('H') => Self::H,
            #[cfg(target_os = "linux")]
            Unicode('i') | Unicode('I') => Self::I,
            #[cfg(target_os = "linux")]
            Unicode('j') | Unicode('J') => Self::J,
            #[cfg(target_os = "linux")]
            Unicode('k') | Unicode('K') => Self::K,
            #[cfg(target_os = "linux")]
            Unicode('l') | Unicode('L') => Self::L,
            #[cfg(target_os = "linux")]
            Unicode('m') | Unicode('M') => Self::M,
            #[cfg(target_os = "linux")]
            Unicode('n') | Unicode('N') => Self::N,
            #[cfg(target_os = "linux")]
            Unicode('o') | Unicode('O') => Self::O,
            #[cfg(target_os = "linux")]
            Unicode('p') | Unicode('P') => Self::P,
            #[cfg(target_os = "linux")]
            Unicode('q') | Unicode('Q') => Self::Q,
            #[cfg(target_os = "linux")]
            Unicode('r') | Unicode('R') => Self::R,
            #[cfg(target_os = "linux")]
            Unicode('s') | Unicode('S') => Self::S,
            #[cfg(target_os = "linux")]
            Unicode('t') | Unicode('T') => Self::T,
            #[cfg(target_os = "linux")]
            Unicode('u') | Unicode('U') => Self::U,
            #[cfg(target_os = "linux")]
            Unicode('v') | Unicode('V') => Self::V,
            #[cfg(target_os = "linux")]
            Unicode('w') | Unicode('W') => Self::W,
            #[cfg(target_os = "linux")]
            Unicode('x') | Unicode('X') => Self::X,
            #[cfg(target_os = "linux")]
            Unicode('y') | Unicode('Y') => Self::Y,
            #[cfg(target_os = "linux")]
            Unicode('z') | Unicode('Z') => Self::Z,

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
            MicMute => Self::MicrophoneMute,

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

            #[allow(deprecated)]
            Command => Self::Meta,
            #[allow(deprecated)]
            Windows => Self::Meta,
            #[allow(deprecated)]
            Super => Self::Meta,
            #[allow(deprecated)]
            Print => Self::PrintScreen,
            #[cfg(target_os = "windows")]
            #[allow(deprecated)]
            Snapshot => Self::PrintScreen,

            #[cfg(target_os = "windows")]
            LButton | MButton | RButton | XButton1 | XButton2 => return Err(KeyError::Unsupported),

            Unicode(_) | Other(_) => return Err(KeyError::Unsupported),
        })
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::{
        core::keyboard::js::{JsKey, JsStandardKey},
        runtime::Runtime,
    };

    #[test]
    #[traced_test]
    #[ignore]
    fn test_keyboard_is_pressed() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval_async::<()>(
                    r#"
                    while(true) {
                        await sleep(1000);
                        console.printLn("hello", await keyboard.isKeyPressed(Key.A));
                    }
                "#,
                )
                .await
                .unwrap();
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_wait_for_key() {
        Runtime::test_with_script_engine(async |script_engine| {
            _ = script_engine
                .eval_async::<()>(
                    r#"
                    await keyboard.waitForKeys(["a", "z"]);
                    //console.printLn("key", key);
                    console.printLn("END");
                "#,
                )
                .await;
        });
    }

    #[test]
    #[traced_test]
    fn test_standard_key() {
        Runtime::test_with_script_engine(async |script_engine| {
            let key = script_engine
                .eval::<JsStandardKey>("Key.Space")
                .await
                .unwrap();
            assert_eq!(key, JsStandardKey::Space);

            script_engine
                .with(|ctx| ctx.globals().set("key", key))
                .await
                .unwrap();

            assert_eq!(
                script_engine.eval::<JsStandardKey>("key").await.unwrap(),
                key
            );
        });
    }

    #[test]
    #[traced_test]
    fn test_key() {
        Runtime::test_with_script_engine(async |script_engine| {
            let key = script_engine.eval::<JsKey>("Key.Space").await.unwrap();
            assert_eq!(key, JsKey::Standard(JsStandardKey::Space));

            let key = script_engine.eval::<JsKey>(r#""Space""#).await.unwrap();
            assert_eq!(key, JsKey::Standard(JsStandardKey::Space));

            assert!(
                script_engine
                    .eval::<JsKey>(r#""InvalidKey""#)
                    .await
                    .is_err()
            );

            let key = script_engine.eval::<JsKey>(r#""=""#).await.unwrap();
            assert_eq!(key, JsKey::Unicode('='));

            let key = script_engine.eval::<JsKey>("42").await.unwrap();
            assert_eq!(key, JsKey::Other(42));
        });
    }
}
