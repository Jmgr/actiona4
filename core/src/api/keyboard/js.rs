use std::{collections::HashSet, str::FromStr, sync::Arc};

use derive_more::Display;
use enigo::Key;
use macros::{FromJsObject, FromSerde, IntoSerde};
use rquickjs::{
    Class, Coerced, Ctx, Exception, FromJs, IntoJs, JsLifetime, Object, Promise, Result, Value,
    atom::PredefinedAtom,
    class::{JsClass, Readable, Trace, Tracer},
    function::Constructor,
    prelude::Opt,
};
use serde::{Deserialize, Serialize};
use strum::{EnumIter, EnumString};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, instrument};

use super::{
    key_triggers::{KeyTriggers, OnKeysOptions},
    text_replacements::{OnTextOptions, Replacement, TextReplacements},
};
use crate::{
    IntoJsResult,
    api::{
        image::js::JsImage,
        js::{
            abort_controller::JsAbortSignal,
            classes::{SingletonClass, register_host_class, registration_target},
            event_handle::{HandleId, JsEventHandle},
            task::task_with_token,
        },
    },
    runtime::{Runtime, WithUserData},
    types::display::display_with_type,
};

impl<'js> Trace<'js> for super::Keyboard {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

/// Controls keyboard input: typing text, pressing keys, waiting for key combinations,
/// and registering text or key event listeners.
///
/// ```ts
/// // Type text
/// keyboard.writeText("Hello, world!");
/// ```
///
/// ```ts
/// // Press a key combination (Ctrl+C)
/// keyboard.pressKey(Key.Control);
/// keyboard.tapKey("c");
/// keyboard.releaseKey(Key.Control);
/// ```
///
/// ```ts
/// // Wait for a key combination
/// await keyboard.waitForKeys([Key.Control, Key.Alt, "q"]);
/// ```
///
/// ```ts
/// // Replace typed text
/// const h = keyboard.onText("btw", "by the way");
/// h.cancel(); // unregister
/// ```
///
/// ```ts
/// // Run a callback when a key combo is pressed
/// const h = keyboard.onKeys([Key.Control, Key.Alt, "t"], () => console.println("triggered!"));
/// ```
/// @singleton
#[derive(Debug, JsLifetime)]
#[rquickjs::class(rename = "Keyboard")]
pub struct JsKeyboard {
    inner: super::Keyboard,
    text_replacements: TextReplacements,
    key_triggers: KeyTriggers,
}

impl<'js> Trace<'js> for JsKeyboard {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> Trace<'js> for TextReplacements {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> Trace<'js> for KeyTriggers {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl SingletonClass<'_> for JsKeyboard {
    fn register_dependencies(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
        register_host_class::<JsEventHandle>(ctx)?;

        // Register the Key class first, then add enum variants as static properties.
        // Both JsKey and JsStandardKey use the name "Key", so we must define the class
        // first and then set enum properties on its constructor object.
        let target = registration_target(ctx);
        Class::<JsKey>::define(&target)?;

        let key_obj: Object = target.get("Key")?;
        for v in <JsStandardKey as strum::IntoEnumIterator>::iter() {
            let name = serde_plain::to_string(&v).map_err(|err| {
                Exception::throw_message(
                    ctx,
                    &format!("Failed to serialize JsStandardKey variant name: {err}"),
                )
            })?;
            key_obj.set(&name, name.clone())?;
        }

        Ok(())
    }
}

impl JsKeyboard {
    /// @skip
    #[instrument(skip_all)]
    pub fn new(
        runtime: Arc<Runtime>,
        task_tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> super::Result<Self> {
        let text_replacements = TextReplacements::new(
            runtime.clone(),
            task_tracker.clone(),
            cancellation_token.clone(),
        );
        let key_triggers = KeyTriggers::new(runtime.clone(), task_tracker, cancellation_token);
        Ok(Self {
            inner: super::Keyboard::new(runtime)?,
            text_replacements,
            key_triggers,
        })
    }

    fn parse_key(ctx: &Ctx<'_>, key: JsKey) -> Result<Key> {
        key.try_into().map_err(|_| {
            Exception::throw_message(ctx, &format!("key {key} is not supported on this platform"))
        })
    }
}

/// Options for `onText`.
/// @options
#[derive(Clone, Debug, FromJsObject)]
pub struct JsOnTextOptions {
    /// Erase the typed text before inserting the replacement.
    /// Set to `false` to trigger an action without replacing the typed text.
    /// @default `true`
    pub erase: bool,

    /// When replacing with text, use the clipboard (Ctrl+V) instead of simulated keystrokes.
    /// Replacing with an image always uses the clipboard.
    /// @default `false`
    pub use_clipboard_for_text: bool,

    /// Save and restore the clipboard contents around a clipboard-based replacement.
    /// @default `true`
    pub save_restore_clipboard: bool,

    /// Abort signal to automatically cancel this listener when signalled.
    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,
}

impl Default for JsOnTextOptions {
    fn default() -> Self {
        Self {
            erase: true,
            use_clipboard_for_text: false,
            save_restore_clipboard: true,
            signal: None,
        }
    }
}

impl From<JsOnTextOptions> for OnTextOptions {
    fn from(options: JsOnTextOptions) -> Self {
        Self {
            erase: options.erase,
            use_clipboard_for_text: options.use_clipboard_for_text,
            save_restore_clipboard: options.save_restore_clipboard,
        }
    }
}

/// Options for key-based methods: `onKey`, `onKeys`, and `waitForKeys`.
///
/// ```ts
/// // Wait for exactly Ctrl+S and no other keys
/// await keyboard.waitForKeys([Key.Control, "s"], { exclusive: true });
/// ```
/// @options
#[derive(Clone, Debug, Default, FromJsObject)]
pub struct JsKeysOptions {
    /// Require exactly these keys and no others to be pressed simultaneously.
    /// @default `false`
    pub exclusive: bool,

    /// Abort signal to cancel the operation.
    /// @default `undefined`
    pub signal: Option<JsAbortSignal>,
}

impl From<JsKeysOptions> for OnKeysOptions {
    fn from(options: JsKeysOptions) -> Self {
        Self {
            exclusive: options.exclusive,
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsKeyboard {
    /// Types the given text string using simulated key events.
    pub fn write_text(&self, ctx: Ctx<'_>, text: String) -> Result<()> {
        self.inner.text(&text).into_js_result(&ctx)?;

        Ok(())
    }

    /// Presses and holds a key until `releaseKey` is called.
    ///
    /// Accepts a `Key` constant, a single character string, or a raw keycode number.
    /// @param key: Key | string | number
    pub fn press_key(&self, ctx: Ctx<'_>, key: JsKey) -> Result<()> {
        let key = Self::parse_key(&ctx, key)?;
        self.inner.press_key(key).into_js_result(&ctx)?;

        Ok(())
    }

    /// Releases a key previously held with `pressKey`.
    ///
    /// Accepts a `Key` constant, a single character string, or a raw keycode number.
    /// @param key: Key | string | number
    pub fn release_key(&self, ctx: Ctx<'_>, key: JsKey) -> Result<()> {
        let key = Self::parse_key(&ctx, key)?;
        self.inner.release_key(key).into_js_result(&ctx)?;

        Ok(())
    }

    /// Presses and releases a key in one action.
    ///
    /// Accepts a `Key` constant, a single character string, or a raw keycode number.
    /// @param key: Key | string | number
    pub fn tap_key(&self, ctx: Ctx<'_>, key: JsKey) -> Result<()> {
        let key = Self::parse_key(&ctx, key)?;
        self.inner.tap_key(key).into_js_result(&ctx)?;

        Ok(())
    }

    /// Presses and holds a raw keycode until `releaseRaw` is called.
    ///
    /// Use this for keys not covered by the `Key` enum.
    pub fn press_raw(&self, ctx: Ctx<'_>, keycode: u16) -> Result<()> {
        self.inner.press_raw(keycode).into_js_result(&ctx)?;

        Ok(())
    }

    /// Releases a raw keycode previously held with `pressRaw`.
    pub fn release_raw(&self, ctx: Ctx<'_>, keycode: u16) -> Result<()> {
        self.inner.release_raw(keycode).into_js_result(&ctx)?;

        Ok(())
    }

    /// Presses and releases a raw keycode in one action.
    ///
    /// Use this for keys not covered by the `Key` enum.
    pub fn tap_raw(&self, ctx: Ctx<'_>, keycode: u16) -> Result<()> {
        self.inner.tap_raw(keycode).into_js_result(&ctx)?;

        Ok(())
    }

    /// Returns whether a key is currently pressed.
    /// @param key: Key | string | number
    pub fn is_key_pressed(&self, ctx: Ctx<'_>, key: JsKey) -> Result<bool> {
        let key = Self::parse_key(&ctx, key)?;

        self.inner.is_key_pressed(key).into_js_result(&ctx)
    }

    /// Returns the list of keys that are currently pressed.
    pub fn get_pressed_keys(&self, ctx: Ctx<'_>) -> Result<Vec<JsKey>> {
        let keys = self.inner.get_pressed_keys().into_js_result(&ctx)?;

        Ok(keys
            .into_iter()
            .filter_map(|enigo_key| {
                let key = JsKey::try_from(enigo_key);

                match key {
                    Ok(key) => Some(key),
                    Err(err) => {
                        debug!("no JsKey for {:?}: {}", enigo_key, err);

                        None
                    }
                }
            })
            .collect())
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
        options: Opt<JsKeysOptions>,
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

    /// Registers a listener that fires when the specified text is typed.
    ///
    /// By default the typed text is erased and replaced with `handler`. Pass
    /// `{ erase: false }` to trigger an action without replacing the text.
    ///
    /// `handler` can be a string, an `Image`, or a callback returning either.
    /// A callback that returns nothing (void) fires without inserting anything.
    ///
    /// ```ts
    /// // Simple text replacement
    /// const h = keyboard.onText("btw", "by the way");
    ///
    /// // Dynamic replacement via callback
    /// const h = keyboard.onText("time", () => new Date().toLocaleTimeString());
    ///
    /// // Trigger only — don't erase the typed text
    /// const h = keyboard.onText("hello", () => console.println("hello typed!"), { erase: false });
    ///
    /// h.cancel(); // unregister
    /// ```
    ///
    /// @param text: string
    /// @param handler: string | Image | (() => string | Image | void | Promise<string | Image | void>)
    /// @param options?: OnTextOptions
    /// @returns EventHandle
    pub fn on_text<'js>(
        &self,
        ctx: Ctx<'js>,
        text: String,
        handler: Value<'js>,
        options: Opt<JsOnTextOptions>,
    ) -> Result<JsEventHandle> {
        if text.is_empty() {
            return Err(Exception::throw_type(&ctx, "text must not be empty"));
        }

        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let id = HandleId::default();

        let replacement = if let Some(func) = handler.as_function() {
            let user_data = ctx.user_data();
            let function_key = user_data.callbacks().register(&ctx, func.clone());
            Replacement::JsCallback((user_data.script_engine().context(), function_key))
        } else if let Ok(image) = handler.get::<JsImage>() {
            Replacement::Image(image.into_inner())
        } else {
            Replacement::Text(handler.get::<Coerced<String>>()?.0)
        };

        self.text_replacements
            .add(id, &text, replacement, options.into());

        let handle = JsEventHandle::new(id, Arc::new(self.text_replacements.clone()));

        Self::cancel_handle_on_signal(&ctx, signal, handle.clone());

        Ok(handle)
    }

    /// Registers a listener that fires when a single key is pressed.
    ///
    /// ```ts
    /// const h = keyboard.onKey(Key.F5, () => console.println("F5 pressed!"));
    /// h.cancel();
    /// ```
    ///
    /// @param key: Key | string | number
    /// @param callback: () => void | Promise<void>
    /// @param options?: KeysOptions
    /// @returns EventHandle
    pub fn on_key<'js>(
        &self,
        ctx: Ctx<'js>,
        key: JsKey,
        callback: Value<'js>,
        options: Opt<JsKeysOptions>,
    ) -> Result<JsEventHandle> {
        let enigo_key = Key::try_from(key)
            .map_err(|_| Exception::throw_message(&ctx, &format!("key {key} is not supported")))?;
        self.register_key_trigger(ctx, vec![enigo_key], callback, options)
    }

    /// Registers a listener that fires when all specified keys are pressed simultaneously.
    ///
    /// ```ts
    /// const h = keyboard.onKeys([Key.Control, Key.Alt, "t"], () => {
    ///   console.println("Ctrl+Alt+T pressed!");
    /// });
    ///
    /// // Require exactly these keys and no others
    /// const h2 = keyboard.onKeys([Key.Control, "s"], () => save(), { exclusive: true });
    ///
    /// h.cancel();
    /// ```
    ///
    /// @param keys: (Key | string | number)[]
    /// @param callback: () => void | Promise<void>
    /// @param options?: KeysOptions
    /// @returns EventHandle
    pub fn on_keys<'js>(
        &self,
        ctx: Ctx<'js>,
        keys: Vec<JsKey>,
        callback: Value<'js>,
        options: Opt<JsKeysOptions>,
    ) -> Result<JsEventHandle> {
        let enigo_keys = keys
            .into_iter()
            .map(|key| {
                Key::try_from(key).map_err(|_| {
                    Exception::throw_message(&ctx, &format!("key {key} is not supported"))
                })
            })
            .collect::<Result<Vec<_>>>()?;
        self.register_key_trigger(ctx, enigo_keys, callback, options)
    }

    /// Unregisters all event handles registered on this keyboard instance.
    ///
    /// ```ts
    /// keyboard.onText("btw", "by the way");
    /// keyboard.onKeys([Key.Control, "s"], () => save());
    /// keyboard.clearEventHandles(); // removes both
    /// ```
    pub fn clear_event_handles(&self) {
        self.text_replacements.clear();
        self.key_triggers.clear();
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Keyboard", &self.inner)
    }
}

impl JsKeyboard {
    fn register_key_trigger<'js>(
        &self,
        ctx: Ctx<'js>,
        keys: Vec<Key>,
        callback: Value<'js>,
        options: Opt<JsKeysOptions>,
    ) -> Result<JsEventHandle> {
        if keys.is_empty() {
            return Err(Exception::throw_type(&ctx, "keys must not be empty"));
        }

        let Some(func) = callback.as_function() else {
            return Err(Exception::throw_type(&ctx, "callback must be a function"));
        };
        let options = options.0.unwrap_or_default();
        let signal = options.signal.clone();
        let id = HandleId::default();
        let user_data = ctx.user_data();
        let function_key = user_data.callbacks().register(&ctx, func.clone());
        let context = user_data.script_engine().context();
        self.key_triggers
            .add(id, keys, context, function_key, options.into());

        let handle = JsEventHandle::new(id, Arc::new(self.key_triggers.clone()));
        Self::cancel_handle_on_signal(&ctx, signal, handle.clone());

        Ok(handle)
    }

    fn cancel_handle_on_signal(
        ctx: &Ctx<'_>,
        signal: Option<JsAbortSignal>,
        handle: JsEventHandle,
    ) {
        let Some(signal) = signal else {
            return;
        };

        let token = signal.into_token();
        ctx.user_data().task_tracker().spawn(async move {
            token.cancelled().await;
            handle.cancel();
        });
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
/// keyboard.tapKey(Key.Return);
/// keyboard.tapKey("a");
/// ```
#[serde(rename = "Key")]
/// @rename Key
pub enum JsStandardKey {
    /// Top-row digit '0' key (not numpad)
    /// `Key.Num0`
    Num0,
    /// Top-row digit '1' key (not numpad)
    /// `Key.Num1`
    Num1,
    /// Top-row digit '2' key (not numpad)
    /// `Key.Num2`
    Num2,
    /// Top-row digit '3' key (not numpad)
    /// `Key.Num3`
    Num3,
    /// Top-row digit '4' key (not numpad)
    /// `Key.Num4`
    Num4,
    /// Top-row digit '5' key (not numpad)
    /// `Key.Num5`
    Num5,
    /// Top-row digit '6' key (not numpad)
    /// `Key.Num6`
    Num6,
    /// Top-row digit '7' key (not numpad)
    /// `Key.Num7`
    Num7,
    /// Top-row digit '8' key (not numpad)
    /// `Key.Num8`
    Num8,
    /// Top-row digit '9' key (not numpad)
    /// `Key.Num9`
    Num9,
    /// Letter key 'A'
    /// `Key.A`
    A,
    /// Letter key 'B'
    /// `Key.B`
    B,
    /// Letter key 'C'
    /// `Key.C`
    C,
    /// Letter key 'D'
    /// `Key.D`
    D,
    /// Letter key 'E'
    /// `Key.E`
    E,
    /// Letter key 'F'
    /// `Key.F`
    F,
    /// Letter key 'G'
    /// `Key.G`
    G,
    /// Letter key 'H'
    /// `Key.H`
    H,
    /// Letter key 'I'
    /// `Key.I`
    I,
    /// Letter key 'J'
    /// `Key.J`
    J,
    /// Letter key 'K'
    /// `Key.K`
    K,
    /// Letter key 'L'
    /// `Key.L`
    L,
    /// Letter key 'M'
    /// `Key.M`
    M,
    /// Letter key 'N'
    /// `Key.N`
    N,
    /// Letter key 'O'
    /// `Key.O`
    O,
    /// Letter key 'P'
    /// `Key.P`
    P,
    /// Letter key 'Q'
    /// `Key.Q`
    Q,
    /// Letter key 'R'
    /// `Key.R`
    R,
    /// Letter key 'S'
    /// `Key.S`
    S,
    /// Letter key 'T'
    /// `Key.T`
    T,
    /// Letter key 'U'
    /// `Key.U`
    U,
    /// Letter key 'V'
    /// `Key.V`
    V,
    /// Letter key 'W'
    /// `Key.W`
    W,
    /// Letter key 'X'
    /// `Key.X`
    X,
    /// Letter key 'Y'
    /// `Key.Y`
    Y,
    /// Letter key 'Z'
    /// `Key.Z`
    Z,
    /// Brazilian ABNT keyboard key C1
    /// @platforms =windows
    /// `Key.AbntC1`
    AbntC1,
    /// Brazilian ABNT keyboard key C2
    /// @platforms =windows
    /// `Key.AbntC2`
    AbntC2,
    /// IME “Accept” / commit conversion
    /// @platforms =windows
    /// `Key.Accept`
    Accept,
    /// Numpad '+' (addition) key
    /// `Key.Add`
    Add,
    /// Alt (Alternate) modifier key
    /// `Key.Alt`
    Alt,
    /// Application/Menu key
    /// @platforms =windows
    /// `Key.Apps`
    Apps,
    /// Attention key (legacy/rare)
    /// @platforms =windows
    /// `Key.Attention`
    Attention,
    /// Backspace / Delete-previous-character
    /// `Key.Backspace`
    Backspace,
    /// Break key (X11/Linux)
    /// @platforms =linux
    /// `Key.Break`
    Break,
    /// Begin key
    /// @platforms =linux
    /// `Key.Begin`
    Begin,
    /// Browser Back
    /// @platforms =windows
    /// `Key.BrowserBack`
    BrowserBack,
    /// Browser Favorites
    /// @platforms =windows
    /// `Key.BrowserFavorites`
    BrowserFavorites,
    /// Browser Forward
    /// @platforms =windows
    /// `Key.BrowserForward`
    BrowserForward,
    /// Browser Home
    /// @platforms =windows
    /// `Key.BrowserHome`
    BrowserHome,
    /// Browser Refresh
    /// @platforms =windows
    /// `Key.BrowserRefresh`
    BrowserRefresh,
    /// Browser Search
    /// @platforms =windows
    /// `Key.BrowserSearch`
    BrowserSearch,
    /// Browser Stop
    /// @platforms =windows
    /// `Key.BrowserStop`
    BrowserStop,
    /// Cancel key (legacy)
    /// `Key.Cancel`
    Cancel,
    /// Caps Lock toggle
    /// `Key.CapsLock`
    CapsLock,
    /// Clear key
    /// `Key.Clear`
    Clear,
    /// Control (Ctrl) modifier key
    /// `Key.Control`
    Control,
    /// IME Convert (start/confirm conversion)
    /// @platforms =windows
    /// `Key.Convert`
    Convert,
    /// Cursor Select (CRSel)
    /// @platforms =windows
    /// `Key.CursorSelect`
    CursorSelect,
    /// IME: switch to alphanumeric
    /// @platforms =windows
    /// `Key.DBEAlphanumeric`
    DBEAlphanumeric,
    /// IME: code input mode
    /// @platforms =windows
    /// `Key.DBECodeinput`
    DBECodeinput,
    /// IME: determine string
    /// @platforms =windows
    /// `Key.DBEDetermineString`
    DBEDetermineString,
    /// IME: enter dialog conversion mode
    /// @platforms =windows
    /// `Key.DBEEnterDLGConversionMode`
    DBEEnterDLGConversionMode,
    /// IME: open configuration
    /// @platforms =windows
    /// `Key.DBEEnterIMEConfigMode`
    DBEEnterIMEConfigMode,
    /// IME: word register mode
    /// @platforms =windows
    /// `Key.DBEEnterWordRegisterMode`
    DBEEnterWordRegisterMode,
    /// IME: flush/reset composition string
    /// @platforms =windows
    /// `Key.DBEFlushString`
    DBEFlushString,
    /// IME: Hiragana
    /// @platforms =windows
    /// `Key.DBEHiragana`
    DBEHiragana,
    /// IME: Katakana
    /// @platforms =windows
    /// `Key.DBEKatakana`
    DBEKatakana,
    /// IME: no code point
    /// @platforms =windows
    /// `Key.DBENoCodepoint`
    DBENoCodepoint,
    /// IME: no roman
    /// @platforms =windows
    /// `Key.DBENoRoman`
    DBENoRoman,
    /// IME: Roman
    /// @platforms =windows
    /// `Key.DBERoman`
    DBERoman,
    /// IME: SBCS character
    /// @platforms =windows
    /// `Key.DBESBCSChar`
    DBESBCSChar,
    /// IME: SBCS/Special char
    /// @platforms =windows
    /// `Key.DBESChar`
    DBESChar,
    /// Numpad decimal point '.'
    /// `Key.Decimal`
    Decimal,
    /// Delete / Forward delete
    /// `Key.Delete`
    Delete,
    /// Numpad divide '/'
    /// `Key.Divide`
    Divide,
    /// Arrow: Down
    /// `Key.DownArrow`
    DownArrow,
    /// End key
    /// `Key.End`
    End,
    /// Erase EOF
    /// @platforms =windows
    /// `Key.Ereof`
    Ereof,
    /// Escape key
    /// `Key.Escape`
    Escape,
    /// Execute key
    /// `Key.Execute`
    Execute,
    /// Extend Selection (ExSel)
    /// @platforms =windows
    /// `Key.Exsel`
    Exsel,
    /// Function key F1
    /// `Key.F1`
    F1,
    /// Function key F2
    /// `Key.F2`
    F2,
    /// Function key F3
    /// `Key.F3`
    F3,
    /// Function key F4
    /// `Key.F4`
    F4,
    /// Function key F5
    /// `Key.F5`
    F5,
    /// Function key F6
    /// `Key.F6`
    F6,
    /// Function key F7
    /// `Key.F7`
    F7,
    /// Function key F8
    /// `Key.F8`
    F8,
    /// Function key F9
    /// `Key.F9`
    F9,
    /// Function key F10
    /// `Key.F10`
    F10,
    /// Function key F11
    /// `Key.F11`
    F11,
    /// Function key F12
    /// `Key.F12`
    F12,
    /// Function key F13
    /// `Key.F13`
    F13,
    /// Function key F14
    /// `Key.F14`
    F14,
    /// Function key F15
    /// `Key.F15`
    F15,
    /// Function key F16
    /// `Key.F16`
    F16,
    /// Function key F17
    /// `Key.F17`
    F17,
    /// Function key F18
    /// `Key.F18`
    F18,
    /// Function key F19
    /// `Key.F19`
    F19,
    /// Function key F20
    /// `Key.F20`
    F20,
    /// Function key F21
    /// `Key.F21`
    F21,
    /// Function key F22
    /// `Key.F22`
    F22,
    /// Function key F23
    /// `Key.F23`
    F23,
    /// Function key F24
    /// `Key.F24`
    F24,
    /// Function key F25
    /// @platforms =linux
    /// `Key.F25`
    F25,
    /// Function key F26
    /// @platforms =linux
    /// `Key.F26`
    F26,
    /// Function key F27
    /// @platforms =linux
    /// `Key.F27`
    F27,
    /// Function key F28
    /// @platforms =linux
    /// `Key.F28`
    F28,
    /// Function key F29
    /// @platforms =linux
    /// `Key.F29`
    F29,
    /// Function key F30
    /// @platforms =linux
    /// `Key.F30`
    F30,
    /// Function key F31
    /// @platforms =linux
    /// `Key.F31`
    F31,
    /// Function key F32
    /// @platforms =linux
    /// `Key.F32`
    F32,
    /// Function key F33
    /// @platforms =linux
    /// `Key.F33`
    F33,
    /// Function key F34
    /// @platforms =linux
    /// `Key.F34`
    F34,
    /// Function key F35
    /// @platforms =linux
    /// `Key.F35`
    F35,
    /// IME Final (end conversion)
    /// @platforms =windows
    /// `Key.Final`
    Final,
    /// Find key
    /// @platforms =linux
    /// `Key.Find`
    Find,
    /// Gamepad: A button
    /// @platforms =windows
    /// `Key.GamepadA`
    GamepadA,
    /// Gamepad: B button
    /// @platforms =windows
    /// `Key.GamepadB`
    GamepadB,
    /// Gamepad: D-Pad Down
    /// @platforms =windows
    /// `Key.GamepadDPadDown`
    GamepadDPadDown,
    /// Gamepad: D-Pad Left
    /// @platforms =windows
    /// `Key.GamepadDPadLeft`
    GamepadDPadLeft,
    /// Gamepad: D-Pad Right
    /// @platforms =windows
    /// `Key.GamepadDPadRight`
    GamepadDPadRight,
    /// Gamepad: D-Pad Up
    /// @platforms =windows
    /// `Key.GamepadDPadUp`
    GamepadDPadUp,
    /// Gamepad: Left shoulder (L1)
    /// @platforms =windows
    /// `Key.GamepadLeftShoulder`
    GamepadLeftShoulder,
    /// Gamepad: Left thumbstick button (L3)
    /// @platforms =windows
    /// `Key.GamepadLeftThumbstickButton`
    GamepadLeftThumbstickButton,
    /// Gamepad: Left thumbstick down
    /// @platforms =windows
    /// `Key.GamepadLeftThumbstickDown`
    GamepadLeftThumbstickDown,
    /// Gamepad: Left thumbstick left
    /// @platforms =windows
    /// `Key.GamepadLeftThumbstickLeft`
    GamepadLeftThumbstickLeft,
    /// Gamepad: Left thumbstick right
    /// @platforms =windows
    /// `Key.GamepadLeftThumbstickRight`
    GamepadLeftThumbstickRight,
    /// Gamepad: Left thumbstick up
    /// @platforms =windows
    /// `Key.GamepadLeftThumbstickUp`
    GamepadLeftThumbstickUp,
    /// Gamepad: Left trigger (L2)
    /// @platforms =windows
    /// `Key.GamepadLeftTrigger`
    GamepadLeftTrigger,
    /// Gamepad: Menu / Start
    /// @platforms =windows
    /// `Key.GamepadMenu`
    GamepadMenu,
    /// Gamepad: Right shoulder (R1)
    /// @platforms =windows
    /// `Key.GamepadRightShoulder`
    GamepadRightShoulder,
    /// Gamepad: Right thumbstick button (R3)
    /// @platforms =windows
    /// `Key.GamepadRightThumbstickButton`
    GamepadRightThumbstickButton,
    /// Gamepad: Right thumbstick down
    /// @platforms =windows
    /// `Key.GamepadRightThumbstickDown`
    GamepadRightThumbstickDown,
    /// Gamepad: Right thumbstick left
    /// @platforms =windows
    /// `Key.GamepadRightThumbstickLeft`
    GamepadRightThumbstickLeft,
    /// Gamepad: Right thumbstick right
    /// @platforms =windows
    /// `Key.GamepadRightThumbstickRight`
    GamepadRightThumbstickRight,
    /// Gamepad: Right thumbstick up
    /// @platforms =windows
    /// `Key.GamepadRightThumbstickUp`
    GamepadRightThumbstickUp,
    /// Gamepad: Right trigger (R2)
    /// @platforms =windows
    /// `Key.GamepadRightTrigger`
    GamepadRightTrigger,
    /// Gamepad: View / Back
    /// @platforms =windows
    /// `Key.GamepadView`
    GamepadView,
    /// Gamepad: X button
    /// @platforms =windows
    /// `Key.GamepadX`
    GamepadX,
    /// Gamepad: Y button
    /// @platforms =windows
    /// `Key.GamepadY`
    GamepadY,
    /// Hangeul toggle (Korean layout)
    /// @platforms =windows
    /// `Key.Hangeul`
    Hangeul,
    /// Hangul toggle (Korean layout)
    /// `Key.Hangul`
    Hangul,
    /// Hanja toggle (Chinese characters on Korean layout)
    /// `Key.Hanja`
    Hanja,
    /// Help key
    /// `Key.Help`
    Help,
    /// Home key
    /// `Key.Home`
    Home,
    /// ICO (legacy) key 00
    /// @platforms =windows
    /// `Key.Ico00`
    Ico00,
    /// ICO (legacy) Clear
    /// @platforms =windows
    /// `Key.IcoClear`
    IcoClear,
    /// ICO (legacy) Help
    /// @platforms =windows
    /// `Key.IcoHelp`
    IcoHelp,
    /// IME Off (disable IME)
    /// @platforms =windows
    /// `Key.IMEOff`
    IMEOff,
    /// IME On (enable IME)
    /// @platforms =windows
    /// `Key.IMEOn`
    IMEOn,
    /// Insert key
    /// `Key.Insert`
    Insert,
    /// IME: Junja mode
    /// @platforms =windows
    /// `Key.Junja`
    Junja,
    /// IME: Kana mode
    /// @platforms =windows
    /// `Key.Kana`
    Kana,
    /// Kanji toggle (Japanese layout)
    /// `Key.Kanji`
    Kanji,
    /// Launch application 1
    /// @platforms =windows
    /// `Key.LaunchApp1`
    LaunchApp1,
    /// Launch application 2
    /// @platforms =windows
    /// `Key.LaunchApp2`
    LaunchApp2,
    /// Launch default mail client
    /// @platforms =windows
    /// `Key.LaunchMail`
    LaunchMail,
    /// Launch media selector
    /// @platforms =windows
    /// `Key.LaunchMediaSelect`
    LaunchMediaSelect,
    /// Left Control
    /// `Key.LeftControl`
    LeftControl,
    /// Arrow: Left
    /// `Key.LeftArrow`
    LeftArrow,
    /// Line Feed key
    /// @platforms =linux
    /// `Key.Linefeed`
    Linefeed,
    /// Left Alt/Menu
    /// `Key.LeftAlt`
    LeftAlt,
    /// Left Shift
    /// `Key.LeftShift`
    LeftShift,
    /// Left Windows / Super key
    /// @platforms =windows
    /// `Key.LeftWindows`
    LeftWindows,
    /// Next media track
    /// `Key.MediaNextTrack`
    MediaNextTrack,
    /// Play/Pause media
    /// `Key.MediaPlayPause`
    MediaPlayPause,
    /// Previous media track
    /// `Key.MediaPrevTrack`
    MediaPrevTrack,
    /// Stop media
    /// `Key.MediaStop`
    MediaStop,
    /// Meta key (also known as "windows", "super", and "command")
    /// `Key.Meta`
    Meta,
    /// IME mode change
    /// `Key.ModeChange`
    ModeChange,
    /// Numpad multiply '*'
    /// `Key.Multiply`
    Multiply,
    /// Navigation: Accept/OK (UWP)
    /// @platforms =windows
    /// `Key.NavigationAccept`
    NavigationAccept,
    /// Navigation: Cancel/Back (UWP)
    /// @platforms =windows
    /// `Key.NavigationCancel`
    NavigationCancel,
    /// Navigation: Down (UWP)
    /// @platforms =windows
    /// `Key.NavigationDown`
    NavigationDown,
    /// Navigation: Left (UWP)
    /// @platforms =windows
    /// `Key.NavigationLeft`
    NavigationLeft,
    /// Navigation: Menu (UWP)
    /// @platforms =windows
    /// `Key.NavigationMenu`
    NavigationMenu,
    /// Navigation: Right (UWP)
    /// @platforms =windows
    /// `Key.NavigationRight`
    NavigationRight,
    /// Navigation: Up (UWP)
    /// @platforms =windows
    /// `Key.NavigationUp`
    NavigationUp,
    /// Navigation: View (UWP)
    /// @platforms =windows
    /// `Key.NavigationView`
    NavigationView,
    /// NoName key (reserved)
    /// @platforms =windows
    /// `Key.NoName`
    NoName,
    /// IME Non-Convert (cancel conversion)
    /// @platforms =windows
    /// `Key.NonConvert`
    NonConvert,
    /// Placeholder "no key"
    /// @platforms =windows
    /// `Key.None`
    None,
    /// Num Lock toggle
    /// `Key.Numlock`
    Numlock,
    /// Numpad digit '0'
    /// `Key.Numpad0`
    Numpad0,
    /// Numpad digit '1'
    /// `Key.Numpad1`
    Numpad1,
    /// Numpad digit '2'
    /// `Key.Numpad2`
    Numpad2,
    /// Numpad digit '3'
    /// `Key.Numpad3`
    Numpad3,
    /// Numpad digit '4'
    /// `Key.Numpad4`
    Numpad4,
    /// Numpad digit '5'
    /// `Key.Numpad5`
    Numpad5,
    /// Numpad digit '6'
    /// `Key.Numpad6`
    Numpad6,
    /// Numpad digit '7'
    /// `Key.Numpad7`
    Numpad7,
    /// Numpad digit '8'
    /// `Key.Numpad8`
    Numpad8,
    /// Numpad digit '9'
    /// `Key.Numpad9`
    Numpad9,
    /// Numpad Enter
    /// `Key.NumpadEnter`
    NumpadEnter,
    /// OEM specific key 1
    /// @platforms =windows
    /// `Key.OEM1`
    OEM1,
    /// OEM specific key 102 (angle bracket/pipe on some layouts)
    /// @platforms =windows
    /// `Key.OEM102`
    OEM102,
    /// OEM specific key 2
    /// @platforms =windows
    /// `Key.OEM2`
    OEM2,
    /// OEM specific key 3 (backtick/tilde on some layouts)
    /// @platforms =windows
    /// `Key.OEM3`
    OEM3,
    /// OEM specific key 4 (left bracket on some layouts)
    /// @platforms =windows
    /// `Key.OEM4`
    OEM4,
    /// OEM specific key 5 (right bracket on some layouts)
    /// @platforms =windows
    /// `Key.OEM5`
    OEM5,
    /// OEM specific key 6 (semicolon on some layouts)
    /// @platforms =windows
    /// `Key.OEM6`
    OEM6,
    /// OEM specific key 7 (quote on some layouts)
    /// @platforms =windows
    /// `Key.OEM7`
    OEM7,
    /// OEM specific key 8
    /// @platforms =windows
    /// `Key.OEM8`
    OEM8,
    /// OEM Attention
    /// @platforms =windows
    /// `Key.OEMAttn`
    OEMAttn,
    /// OEM Auto
    /// @platforms =windows
    /// `Key.OEMAuto`
    OEMAuto,
    /// OEM Ax
    /// @platforms =windows
    /// `Key.OEMAx`
    OEMAx,
    /// OEM Backtab (reverse Tab)
    /// @platforms =windows
    /// `Key.OEMBacktab`
    OEMBacktab,
    /// OEM Clear
    /// @platforms =windows
    /// `Key.OEMClear`
    OEMClear,
    /// OEM Comma ','
    /// @platforms =windows
    /// `Key.OEMComma`
    OEMComma,
    /// OEM Copy
    /// @platforms =windows
    /// `Key.OEMCopy`
    OEMCopy,
    /// OEM Cusel
    /// @platforms =windows
    /// `Key.OEMCusel`
    OEMCusel,
    /// OEM Enlw
    /// @platforms =windows
    /// `Key.OEMEnlw`
    OEMEnlw,
    /// OEM Finish
    /// @platforms =windows
    /// `Key.OEMFinish`
    OEMFinish,
    /// OEM FJ Jisho (dictionary)
    /// @platforms =windows
    /// `Key.OEMFJJisho`
    OEMFJJisho,
    /// OEM FJ Loya
    /// @platforms =windows
    /// `Key.OEMFJLoya`
    OEMFJLoya,
    /// OEM FJ Masshou
    /// @platforms =windows
    /// `Key.OEMFJMasshou`
    OEMFJMasshou,
    /// OEM FJ Roya
    /// @platforms =windows
    /// `Key.OEMFJRoya`
    OEMFJRoya,
    /// OEM FJ Touroku
    /// @platforms =windows
    /// `Key.OEMFJTouroku`
    OEMFJTouroku,
    /// OEM Jump
    /// @platforms =windows
    /// `Key.OEMJump`
    OEMJump,
    /// OEM Minus '-'
    /// @platforms =windows
    /// `Key.OEMMinus`
    OEMMinus,
    /// OEM NEC Equal '='
    /// @platforms =windows
    /// `Key.OEMNECEqual`
    OEMNECEqual,
    /// OEM PA1
    /// @platforms =windows
    /// `Key.OEMPA1`
    OEMPA1,
    /// OEM PA2
    /// @platforms =windows
    /// `Key.OEMPA2`
    OEMPA2,
    /// OEM PA3
    /// @platforms =windows
    /// `Key.OEMPA3`
    OEMPA3,
    /// OEM Period '.'
    /// @platforms =windows
    /// `Key.OEMPeriod`
    OEMPeriod,
    /// OEM Plus '+'
    /// @platforms =windows
    /// `Key.OEMPlus`
    OEMPlus,
    /// OEM Reset
    /// @platforms =windows
    /// `Key.OEMReset`
    OEMReset,
    /// OEM Wsctrl
    /// @platforms =windows
    /// `Key.OEMWsctrl`
    OEMWsctrl,
    /// Same as Alt
    /// `Key.Option`
    Option,
    /// PA1 key
    /// @platforms =windows
    /// `Key.PA1`
    PA1,
    /// Packet key (used to pass Unicode chars)
    /// @platforms =windows
    /// `Key.Packet`
    Packet,
    /// Page Down
    /// `Key.PageDown`
    PageDown,
    /// Page Up
    /// `Key.PageUp`
    PageUp,
    /// Pause key
    /// `Key.Pause`
    Pause,
    /// Media Play
    /// @platforms =windows
    /// `Key.Play`
    Play,
    /// Screenshot
    /// `Key.PrintScreen`
    PrintScreen,
    /// IME Process key
    /// @platforms =windows
    /// `Key.Processkey`
    Processkey,
    /// Right Control
    /// `Key.RightControl`
    RightControl,
    /// Redo
    /// @platforms =linux
    /// `Key.Redo`
    Redo,
    /// Enter / Return
    /// `Key.Return`
    Return,
    /// Arrow: Right
    /// `Key.RightArrow`
    RightArrow,
    /// Right Alt/Menu
    /// @platforms =windows
    /// `Key.RightAlt`
    RightAlt,
    /// Right Shift
    /// `Key.RightShift`
    RightShift,
    /// Right Windows / Super key
    /// @platforms =windows
    /// `Key.RightWindows`
    RightWindows,
    /// Scroll key (legacy)
    /// @platforms =windows
    /// `Key.Scroll`
    Scroll,
    /// Scroll Lock
    /// @platforms =linux
    /// `Key.ScrollLock`
    ScrollLock,
    /// Select key
    /// `Key.Select`
    Select,
    /// Script switch
    /// @platforms =linux
    /// `Key.ScriptSwitch`
    ScriptSwitch,
    /// Numpad separator (locale-dependent)
    /// @platforms =windows
    /// `Key.Separator`
    Separator,
    /// Shift modifier
    /// `Key.Shift`
    Shift,
    /// Shift Lock
    /// @platforms =linux
    /// `Key.ShiftLock`
    ShiftLock,
    /// System Sleep
    /// @platforms =windows
    /// `Key.Sleep`
    Sleep,
    /// Spacebar
    /// `Key.Space`
    Space,
    /// Numpad '-' (subtract)
    /// `Key.Subtract`
    Subtract,
    /// System Request (SysRq)
    /// @platforms =linux
    /// `Key.SysReq`
    SysReq,
    /// Tab / focus next
    /// `Key.Tab`
    Tab,
    /// Undo
    /// @platforms =linux
    /// `Key.Undo`
    Undo,
    /// Arrow: Up
    /// `Key.UpArrow`
    UpArrow,
    /// Volume down
    /// `Key.VolumeDown`
    VolumeDown,
    /// Volume mute
    /// `Key.VolumeMute`
    VolumeMute,
    /// Volume up
    /// `Key.VolumeUp`
    VolumeUp,
    /// Microphone mute
    /// @platforms =linux
    /// `Key.MicrophoneMute`
    MicrophoneMute,
    /// Zoom key
    /// @platforms =windows
    /// `Key.Zoom`
    Zoom,
}

/// @skip
#[derive(Clone, Copy, Debug, Display, Eq, Hash, JsLifetime, PartialEq)]
pub enum JsKey {
    /// `Key.Standard`
    Standard(JsStandardKey),
    /// `Key.Unicode`
    Unicode(char),
    /// `Key.Other`
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

#[derive(Debug, Display)]
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
            A => Self::Unicode('a'),
            #[cfg(target_os = "linux")]
            B => Self::Unicode('b'),
            #[cfg(target_os = "linux")]
            C => Self::Unicode('c'),
            #[cfg(target_os = "linux")]
            D => Self::Unicode('d'),
            #[cfg(target_os = "linux")]
            E => Self::Unicode('e'),
            #[cfg(target_os = "linux")]
            F => Self::Unicode('f'),
            #[cfg(target_os = "linux")]
            G => Self::Unicode('g'),
            #[cfg(target_os = "linux")]
            H => Self::Unicode('h'),
            #[cfg(target_os = "linux")]
            I => Self::Unicode('i'),
            #[cfg(target_os = "linux")]
            J => Self::Unicode('j'),
            #[cfg(target_os = "linux")]
            K => Self::Unicode('k'),
            #[cfg(target_os = "linux")]
            L => Self::Unicode('l'),
            #[cfg(target_os = "linux")]
            M => Self::Unicode('m'),
            #[cfg(target_os = "linux")]
            N => Self::Unicode('n'),
            #[cfg(target_os = "linux")]
            O => Self::Unicode('o'),
            #[cfg(target_os = "linux")]
            P => Self::Unicode('p'),
            #[cfg(target_os = "linux")]
            Q => Self::Unicode('q'),
            #[cfg(target_os = "linux")]
            R => Self::Unicode('r'),
            #[cfg(target_os = "linux")]
            S => Self::Unicode('s'),
            #[cfg(target_os = "linux")]
            T => Self::Unicode('t'),
            #[cfg(target_os = "linux")]
            U => Self::Unicode('u'),
            #[cfg(target_os = "linux")]
            V => Self::Unicode('v'),
            #[cfg(target_os = "linux")]
            W => Self::Unicode('w'),
            #[cfg(target_os = "linux")]
            X => Self::Unicode('x'),
            #[cfg(target_os = "linux")]
            Y => Self::Unicode('y'),
            #[cfg(target_os = "linux")]
            Z => Self::Unicode('z'),

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
        api::keyboard::js::{JsKey, JsStandardKey},
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
                        await sleep("1s");
                        console.println("hello", keyboard.isKeyPressed(Key.A));
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
                    //console.println("key", key);
                    console.println("END");
                "#,
                )
                .await;
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_on_text() {
        Runtime::test_with_script_engine(async |script_engine| {
            _ = script_engine
                .eval_async::<()>(
                    r#"
                    console.println("Registering keyboard.onText handlers.");
                    console.println("Type `btw` to replace it, and `hello` to trigger a callback without erase.");
                    console.println("Press Escape to end this manual test.");

                    const replacementHandle = keyboard.onText("btw", "by the way");
                    const callbackHandle = keyboard.onText(
                        "hello",
                        () => console.println("onText callback fired"),
                        { erase: false },
                    );

                    await keyboard.waitForKeys([Key.Escape]);
                    console.println("STOPPING");
                    replacementHandle.cancel();
                    callbackHandle.cancel();
                    console.println("END");
                "#,
                )
                .await;
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_on_key() {
        Runtime::test_with_script_engine(async |script_engine| {
            _ = script_engine
                .eval_async::<()>(
                    r#"
                    console.println("Registering keyboard.onKey for F8.");
                    console.println("Press Escape to end this manual test.");
                    const handle = keyboard.onKey(Key.F8, async () => {await sleep(250); console.println("F8 pressed");});

                    await keyboard.waitForKeys([Key.Escape]);
                    console.println("STOPPING");
                    handle.cancel();
                    console.println("END");
                "#,
                )
                .await;
        });
    }

    #[test]
    #[traced_test]
    #[ignore]
    fn test_on_keys_and_clear_event_handles() {
        Runtime::test_with_script_engine(async |script_engine| {
            _ = script_engine
                .eval_async::<()>(
                    r#"
                    console.println("Registering keyboard.onKeys handlers.");
                    console.println("Try Ctrl+Alt+T and Ctrl+S (exclusive).");
                    console.println("Press Escape to end this manual test.");

                    keyboard.onKeys([Key.Control, Key.Alt, "t"], () => {
                        console.println("Ctrl+Alt+T fired");
                    });
                    keyboard.onKeys(
                        [Key.Control, "s"],
                        () => console.println("Ctrl+S exclusive fired"),
                        { exclusive: true },
                    );

                    await keyboard.waitForKeys([Key.Escape]);
                    console.println("STOPPING");
                    keyboard.clearEventHandles();
                    console.println("END");
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
