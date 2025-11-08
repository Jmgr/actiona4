#![allow(clippy::as_conversions)]

use std::{
    collections::HashSet,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicUsize, Ordering},
    },
};

use derive_more::Display;
use derive_more::{Constructor, Deref};
use enigo::Key;
use eyre::Result;
use once_cell::sync::OnceCell;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::warn;
use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{
            GetKeyNameTextW, GetKeyState, GetKeyboardLayout, HKL, ToUnicodeEx, VIRTUAL_KEY,
            VK__none_, VK_0, VK_1, VK_2, VK_3, VK_4, VK_5, VK_6, VK_7, VK_8, VK_9, VK_A,
            VK_ABNT_C1, VK_ABNT_C2, VK_ACCEPT, VK_ADD, VK_APPS, VK_ATTN, VK_B, VK_BACK,
            VK_BROWSER_BACK, VK_BROWSER_FAVORITES, VK_BROWSER_FORWARD, VK_BROWSER_HOME,
            VK_BROWSER_REFRESH, VK_BROWSER_SEARCH, VK_BROWSER_STOP, VK_C, VK_CANCEL, VK_CAPITAL,
            VK_CLEAR, VK_CONTROL, VK_CONVERT, VK_CRSEL, VK_D, VK_DECIMAL, VK_DELETE, VK_DIVIDE,
            VK_DOWN, VK_E, VK_END, VK_EREOF, VK_ESCAPE, VK_EXECUTE, VK_EXSEL, VK_F, VK_F1, VK_F2,
            VK_F3, VK_F4, VK_F5, VK_F6, VK_F7, VK_F8, VK_F9, VK_F10, VK_F11, VK_F12, VK_F13,
            VK_F14, VK_F15, VK_F16, VK_F17, VK_F18, VK_F19, VK_F20, VK_F21, VK_F22, VK_F23, VK_F24,
            VK_FINAL, VK_G, VK_H, VK_HELP, VK_HOME, VK_I, VK_ICO_00, VK_ICO_CLEAR, VK_ICO_HELP,
            VK_IME_OFF, VK_IME_ON, VK_INSERT, VK_J, VK_K, VK_L, VK_LAUNCH_APP1, VK_LAUNCH_APP2,
            VK_LAUNCH_MAIL, VK_LAUNCH_MEDIA_SELECT, VK_LBUTTON, VK_LCONTROL, VK_LEFT, VK_LMENU,
            VK_LSHIFT, VK_LWIN, VK_M, VK_MBUTTON, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE,
            VK_MEDIA_PREV_TRACK, VK_MEDIA_STOP, VK_MENU, VK_MODECHANGE, VK_MULTIPLY, VK_N,
            VK_NAVIGATION_ACCEPT, VK_NAVIGATION_CANCEL, VK_NAVIGATION_DOWN, VK_NAVIGATION_LEFT,
            VK_NAVIGATION_MENU, VK_NAVIGATION_RIGHT, VK_NAVIGATION_UP, VK_NAVIGATION_VIEW, VK_NEXT,
            VK_NONAME, VK_NONCONVERT, VK_NUMLOCK, VK_NUMPAD0, VK_NUMPAD1, VK_NUMPAD2, VK_NUMPAD3,
            VK_NUMPAD4, VK_NUMPAD5, VK_NUMPAD6, VK_NUMPAD7, VK_NUMPAD8, VK_NUMPAD9, VK_O, VK_OEM_1,
            VK_OEM_2, VK_OEM_3, VK_OEM_4, VK_OEM_5, VK_OEM_6, VK_OEM_7, VK_OEM_8, VK_OEM_102,
            VK_OEM_COMMA, VK_OEM_MINUS, VK_OEM_PERIOD, VK_OEM_PLUS, VK_P, VK_PA1, VK_PACKET,
            VK_PAUSE, VK_PLAY, VK_PRINT, VK_PRIOR, VK_PROCESSKEY, VK_Q, VK_R, VK_RBUTTON,
            VK_RCONTROL, VK_RETURN, VK_RIGHT, VK_RMENU, VK_RSHIFT, VK_RWIN, VK_S, VK_SCROLL,
            VK_SELECT, VK_SEPARATOR, VK_SHIFT, VK_SLEEP, VK_SNAPSHOT, VK_SPACE, VK_SUBTRACT, VK_T,
            VK_TAB, VK_U, VK_UP, VK_V, VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP, VK_W, VK_X,
            VK_XBUTTON1, VK_XBUTTON2, VK_Y, VK_Z, VK_ZOOM,
        },
        WindowsAndMessaging::{
            CallNextHookEx, GetForegroundWindow, GetWindowThreadProcessId, HC_ACTION, HOOKPROC,
            KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, LLKHF_UP, WH_KEYBOARD_LL,
            WINDOWS_HOOK_ID, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
};

use crate::{
    runtime::{
        events::{AllSignals, Guard, KeyboardKeyEvent, KeyboardTextEvent, Topic, TopicWrapper},
        platform::win::{
            SafeMessagePump,
            events::input::{HookSpec, LowLevelHookRunner, MSG_START, MSG_STOP},
        },
    },
    types::input::Direction,
};

static KEYBOARD_INPUT_DISPATCHER: OnceCell<Weak<KeyboardInputDispatcher>> = OnceCell::new();

#[derive(Default)]
pub struct KeyboardHook {}

impl HookSpec for KeyboardHook {
    const ID: WINDOWS_HOOK_ID = WH_KEYBOARD_LL;

    fn proc() -> HOOKPROC {
        Some(low_level_keyboard_proc)
    }
}

#[derive(Debug, Clone, Copy, Hash, Constructor, PartialEq, Eq, Display)]
#[display("(scan code: {scan_code}, vk code: {vk_code}, extended: {extended})")]
struct KeyId {
    scan_code: u32,
    vk_code: u32,
    extended: bool,
}

#[derive(Debug)]
pub struct KeyboardInputDispatcher {
    keys: Arc<TopicWrapper<KeyboardKeysTopic>>,
    text: Arc<TopicWrapper<KeyboardTextTopic>>,
    subscribers: Arc<AtomicUsize>,
    message_pump: SafeMessagePump,
    pressed_keys: Mutex<HashSet<KeyId>>,
}

impl KeyboardInputDispatcher {
    pub async fn new(
        cancellation_token: CancellationToken,
        task_tracker: TaskTracker,
    ) -> Result<Arc<Self>> {
        let message_pump = SafeMessagePump::new::<LowLevelHookRunner<KeyboardHook>>(
            "input_dispatcher",
            cancellation_token.clone(),
            task_tracker.clone(),
        )
        .await?;

        Ok(Arc::new_cyclic(|me| {
            if KEYBOARD_INPUT_DISPATCHER.set(me.clone()).is_err() {
                panic!("InputDispatcher should only be instantiated once");
            }

            Self {
                keys: Arc::new(TopicWrapper::new(
                    KeyboardKeysTopic {
                        dispatcher: me.clone(),
                    },
                    cancellation_token.clone(),
                    task_tracker.clone(),
                )),
                text: Arc::new(TopicWrapper::new(
                    KeyboardTextTopic {
                        dispatcher: me.clone(),
                    },
                    cancellation_token.clone(),
                    task_tracker.clone(),
                )),
                subscribers: Arc::new(AtomicUsize::new(0)),
                message_pump,
                pressed_keys: Mutex::new(HashSet::default()),
            }
        }))
    }

    #[must_use]
    pub fn subscribe_keyboard_keys(&self) -> Guard<KeyboardKeysTopic> {
        self.keys.subscribe()
    }

    #[must_use]
    pub fn subscribe_keyboard_text(&self) -> Guard<KeyboardTextTopic> {
        self.text.subscribe()
    }

    fn check_is_repeat(&self, key_id: &KeyId, is_pressed: bool) -> bool {
        let mut pressed_keys = self.pressed_keys.lock().unwrap();
        if is_pressed {
            if !pressed_keys.insert(*key_id) {
                return true;
            }
        } else if !pressed_keys.remove(key_id) {
            warn!("releasing a non-pressed key: {key_id}");
        }

        false
    }

    fn event_received(&self, keyboard_struct: &KBDLLHOOKSTRUCT, message: u32) {
        if !matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP) {
            return;
        }

        let is_pressed = keyboard_struct.flags & LLKHF_UP != LLKHF_UP;
        let is_extended = keyboard_struct.flags & LLKHF_EXTENDED == LLKHF_EXTENDED;
        let key_id = KeyId::new(
            keyboard_struct.scanCode,
            keyboard_struct.vkCode,
            is_extended,
        );
        let is_repeat = self.check_is_repeat(&key_id, is_pressed);

        let keystate = get_keystate();
        let is_injected = keyboard_struct.flags & LLKHF_INJECTED == LLKHF_INJECTED;
        let key = vk_to_enigo_key(
            keyboard_struct.vkCode,
            keyboard_struct.scanCode,
            is_extended,
            &keystate,
        );

        if is_pressed
            && let Some(character) =
                to_unicode(keyboard_struct.vkCode, keyboard_struct.scanCode, &keystate)
        {
            self.text
                .publish(KeyboardTextEvent::new(character, is_injected, is_repeat));
        }

        let name = key_name_from_llhook(keyboard_struct.scanCode, is_extended).unwrap_or_default();

        self.keys.publish(KeyboardKeyEvent::new(
            key,
            keyboard_struct.scanCode,
            if is_pressed {
                Direction::Press
            } else {
                Direction::Release
            },
            is_injected,
            name,
            is_repeat,
        ))
    }

    async fn on_start(&self) {
        if self.subscribers.fetch_add(1, Ordering::Relaxed) == 0 {
            self.message_pump.send_message(MSG_START);
        }
    }

    async fn on_stop(&self) {
        if self.subscribers.fetch_sub(1, Ordering::Relaxed) == 1 {
            self.message_pump.send_message(MSG_STOP);
        }
    }
}

#[derive(Debug, Default)]
pub struct KeyboardKeysTopic {
    dispatcher: Weak<KeyboardInputDispatcher>,
}

impl Topic for KeyboardKeysTopic {
    type T = KeyboardKeyEvent;
    type Signal = AllSignals<Self::T>;

    async fn on_start(&self) -> Result<()> {
        if let Some(dispatcher) = self.dispatcher.upgrade() {
            dispatcher.on_start().await;
        }
        Ok(())
    }

    async fn on_stop(&self) -> Result<()> {
        if let Some(dispatcher) = self.dispatcher.upgrade() {
            dispatcher.on_stop().await;
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct KeyboardTextTopic {
    dispatcher: Weak<KeyboardInputDispatcher>,
}

impl Topic for KeyboardTextTopic {
    type T = KeyboardTextEvent;
    type Signal = AllSignals<Self::T>;

    async fn on_start(&self) -> Result<()> {
        if let Some(dispatcher) = self.dispatcher.upgrade() {
            dispatcher.on_start().await;
        }
        Ok(())
    }

    async fn on_stop(&self) -> Result<()> {
        if let Some(dispatcher) = self.dispatcher.upgrade() {
            dispatcher.on_stop().await;
        }
        Ok(())
    }
}

unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code != HC_ACTION as i32 {
        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
    }

    let Some(dispatcher) = KEYBOARD_INPUT_DISPATCHER
        .get()
        .and_then(|dispatcher| dispatcher.upgrade())
    else {
        return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
    };

    let keyboard_struct = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };

    dispatcher.event_received(&keyboard_struct, w_param.0 as u32);

    unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
}

fn key_name_from_llhook(scan_code: u32, is_extended: bool) -> Option<String> {
    // Build an lParam like WM_KEYDOWN:
    // bits 16..23 = scan code
    // bit 24      = extended (E0 prefix)
    let mut lparam: i32 = (scan_code as i32) << 16;
    if is_extended {
        lparam |= 1 << 24;
    }

    let mut buf = [0u16; 64];
    let len = unsafe { GetKeyNameTextW(lparam, &mut buf) };
    if len > 0 {
        Some(String::from_utf16_lossy(&buf[..len as usize]))
    } else {
        None
    }
}

fn vk_to_enigo_key(vk_code: u32, scan_code: u32, is_extended: bool, keystate: &Keystate) -> Key {
    match VIRTUAL_KEY(vk_code as u16) {
        VK_RETURN if is_extended => Key::NumpadEnter,
        VK_RETURN if !is_extended => Key::Return,

        VK_0 => Key::Num0,
        VK_1 => Key::Num1,
        VK_2 => Key::Num2,
        VK_3 => Key::Num3,
        VK_4 => Key::Num4,
        VK_5 => Key::Num5,
        VK_6 => Key::Num6,
        VK_7 => Key::Num7,
        VK_8 => Key::Num8,
        VK_9 => Key::Num9,

        VK_A => Key::A,
        VK_B => Key::B,
        VK_C => Key::C,
        VK_D => Key::D,
        VK_E => Key::E,
        VK_F => Key::F,
        VK_G => Key::G,
        VK_H => Key::H,
        VK_I => Key::I,
        VK_J => Key::J,
        VK_K => Key::K,
        VK_L => Key::L,
        VK_M => Key::M,
        VK_N => Key::N,
        VK_O => Key::O,
        VK_P => Key::P,
        VK_Q => Key::Q,
        VK_R => Key::R,
        VK_S => Key::S,
        VK_T => Key::T,
        VK_U => Key::U,
        VK_V => Key::V,
        VK_W => Key::W,
        VK_X => Key::X,
        VK_Y => Key::Y,
        VK_Z => Key::Z,

        VK_ABNT_C1 => Key::AbntC1,
        VK_ABNT_C2 => Key::AbntC2,
        VK_ACCEPT => Key::Accept,
        VK_ADD => Key::Add,
        VK_MENU => Key::Alt,
        VK_APPS => Key::Apps,
        VK_ATTN => Key::Attn,
        VK_BACK => Key::Backspace,
        VK_BROWSER_BACK => Key::BrowserBack,
        VK_BROWSER_FAVORITES => Key::BrowserFavorites,
        VK_BROWSER_FORWARD => Key::BrowserForward,
        VK_BROWSER_HOME => Key::BrowserHome,
        VK_BROWSER_REFRESH => Key::BrowserRefresh,
        VK_BROWSER_SEARCH => Key::BrowserSearch,
        VK_BROWSER_STOP => Key::BrowserStop,
        VK_CANCEL => Key::Cancel,
        VK_CAPITAL => Key::CapsLock,
        VK_CLEAR => Key::Clear,
        VK_CONTROL if is_extended => Key::RControl,
        VK_CONTROL if !is_extended => Key::LControl,
        VK_CONVERT => Key::Convert,
        VK_CRSEL => Key::Crsel,
        /*
        // TODO: we ignore this for now
        VK_DBE_ALPHANUMERIC => Key::DBEAlphanumeric,
        VK_DBE_CODEINPUT => Key::DBECodeinput,
        VK_DBE_DETERMINESTRING => Key::DBEDetermineString,
        VK_DBE_ENTERDLGCONVERSIONMODE => Key::DBEEnterDLGConversionMode,
        VK_DBE_ENTERIMECONFIGMODE => Key::DBEEnterIMEConfigMode,
        VK_DBE_ENTERWORDREGISTERMODE => Key::DBEEnterWordRegisterMode,
        VK_DBE_FLUSHSTRING => Key::DBEFlushString,
        VK_DBE_HIRAGANA => Key::DBEHiragana,
        VK_DBE_KATAKANA => Key::DBEKatakana,
        VK_DBE_NOCODEINPUT => Key::DBENoCodepoint,
        VK_DBE_NOROMAN => Key::DBENoRoman,
        VK_DBE_ROMAN => Key::DBERoman,
        VK_DBE_SBCSCHAR => Key::DBESBCSChar,
        VK_DBE_DBCSCHAR => Key::DBESChar,
        */
        VK_DECIMAL => Key::Decimal,
        VK_DELETE => Key::Delete,
        VK_DIVIDE => Key::Divide,
        VK_DOWN => Key::DownArrow,
        VK_END => Key::End,
        VK_EREOF => Key::Ereof,
        VK_ESCAPE => Key::Escape,
        VK_EXECUTE => Key::Execute,
        VK_EXSEL => Key::Exsel,

        VK_F1 => Key::F1,
        VK_F2 => Key::F2,
        VK_F3 => Key::F3,
        VK_F4 => Key::F4,
        VK_F5 => Key::F5,
        VK_F6 => Key::F6,
        VK_F7 => Key::F7,
        VK_F8 => Key::F8,
        VK_F9 => Key::F9,
        VK_F10 => Key::F10,
        VK_F11 => Key::F11,
        VK_F12 => Key::F12,
        VK_F13 => Key::F13,
        VK_F14 => Key::F14,
        VK_F15 => Key::F15,
        VK_F16 => Key::F16,
        VK_F17 => Key::F17,
        VK_F18 => Key::F18,
        VK_F19 => Key::F19,
        VK_F20 => Key::F20,
        VK_F21 => Key::F21,
        VK_F22 => Key::F22,
        VK_F23 => Key::F23,
        VK_F24 => Key::F24,

        VK_FINAL => Key::Final,
        /*
        // TODO: we ignore this for now
        VK_HANGEUL => Key::Hangeul,
        VK_HANGUL => Key::Hangul,
        VK_HANJA => Key::Hanja,
        */
        VK_HELP => Key::Help,
        VK_HOME => Key::Home,
        VK_ICO_00 => Key::Ico00,
        VK_ICO_CLEAR => Key::IcoClear,
        VK_ICO_HELP => Key::IcoHelp,
        VK_IME_OFF => Key::IMEOff,
        VK_IME_ON => Key::IMEOn,
        VK_INSERT => Key::Insert,
        /*
        // TODO: we ignore this for now
        VK_JUNJA => Key::Junja,
        VK_KANA => Key::Kana,
        VK_KANJI => Key::Kanji,
        */
        VK_LAUNCH_APP1 => Key::LaunchApp1,
        VK_LAUNCH_APP2 => Key::LaunchApp2,
        VK_LAUNCH_MAIL => Key::LaunchMail,
        VK_LAUNCH_MEDIA_SELECT => Key::LaunchMediaSelect,
        VK_LBUTTON => Key::LButton,
        VK_LCONTROL => Key::LControl,
        VK_LEFT => Key::LeftArrow,
        VK_LMENU => Key::LMenu,
        VK_LSHIFT => Key::LShift,
        VK_LWIN => Key::LWin,
        VK_MBUTTON => Key::MButton,
        VK_MEDIA_NEXT_TRACK => Key::MediaNextTrack,
        VK_MEDIA_PLAY_PAUSE => Key::MediaPlayPause,
        VK_MEDIA_PREV_TRACK => Key::MediaPrevTrack,
        VK_MEDIA_STOP => Key::MediaStop,
        VK_MODECHANGE => Key::ModeChange,
        VK_MULTIPLY => Key::Multiply,
        VK_NAVIGATION_ACCEPT => Key::NavigationAccept,
        VK_NAVIGATION_CANCEL => Key::NavigationCancel,
        VK_NAVIGATION_DOWN => Key::NavigationDown,
        VK_NAVIGATION_LEFT => Key::NavigationLeft,
        VK_NAVIGATION_MENU => Key::NavigationMenu,
        VK_NAVIGATION_RIGHT => Key::NavigationRight,
        VK_NAVIGATION_UP => Key::NavigationUp,
        VK_NAVIGATION_VIEW => Key::NavigationView,
        VK_NONAME => Key::NoName,
        VK_NONCONVERT => Key::NonConvert,
        #[allow(non_upper_case_globals)]
        VK__none_ => Key::None,
        VK_NUMLOCK => Key::Numlock,
        VK_NUMPAD0 => Key::Numpad0,
        VK_NUMPAD1 => Key::Numpad1,
        VK_NUMPAD2 => Key::Numpad2,
        VK_NUMPAD3 => Key::Numpad3,
        VK_NUMPAD4 => Key::Numpad4,
        VK_NUMPAD5 => Key::Numpad5,
        VK_NUMPAD6 => Key::Numpad6,
        VK_NUMPAD7 => Key::Numpad7,
        VK_NUMPAD8 => Key::Numpad8,
        VK_NUMPAD9 => Key::Numpad9,
        VK_OEM_1 => Key::OEM1,
        VK_OEM_102 => Key::OEM102,
        VK_OEM_2 => Key::OEM2,
        VK_OEM_3 => Key::OEM3,
        VK_OEM_4 => Key::OEM4,
        VK_OEM_5 => Key::OEM5,
        VK_OEM_6 => Key::OEM6,
        VK_OEM_7 => Key::OEM7,
        VK_OEM_8 => Key::OEM8,
        VK_OEM_COMMA => Key::OEMComma,
        VK_OEM_PERIOD => Key::OEMPeriod,
        VK_OEM_MINUS => Key::OEMMinus,
        VK_OEM_PLUS => Key::OEMPlus,
        /*
        // TODO: we ignore this for now
        VK_OEM_ATTN => Key::OEMAttn,
        VK_OEM_AUTO => Key::OEMAuto,
        VK_OEM_AX => Key::OEMAx,
        VK_OEM_BACKTAB => Key::OEMBacktab,
        VK_OEM_CLEAR => Key::OEMClear,
        VK_OEM_COPY => Key::OEMCopy,
        VK_OEM_CUSEL => Key::OEMCusel,
        VK_OEM_ENLW => Key::OEMEnlw,
        VK_OEM_FINISH => Key::OEMFinish,
        VK_OEM_FJ_JISHO => Key::OEMFJJisho,
        VK_OEM_FJ_LOYA => Key::OEMFJLoya,
        VK_OEM_FJ_MASSHOU => Key::OEMFJMasshou,
        VK_OEM_FJ_ROYA => Key::OEMFJRoya,
        VK_OEM_FJ_TOUROKU => Key::OEMFJTouroku,
        VK_OEM_JUMP => Key::OEMJump,
        VK_OEM_NEC_EQUAL => Key::OEMNECEqual,
        VK_OEM_PA1 => Key::OEMPA1,
        VK_OEM_PA2 => Key::OEMPA2,
        VK_OEM_PA3 => Key::OEMPA3,
        VK_OEM_RESET => Key::OEMReset,
        VK_OEM_WSCTRL => Key::OEMWsctrl,
        */
        VK_PA1 => Key::PA1,
        VK_PACKET => Key::Packet,
        VK_NEXT => Key::PageDown,
        VK_PRIOR => Key::PageUp,
        VK_PAUSE => Key::Pause,
        VK_PLAY => Key::Play,
        VK_SNAPSHOT | VK_PRINT => Key::PrintScr,
        VK_PROCESSKEY => Key::Processkey,
        VK_RBUTTON => Key::RButton,
        VK_RCONTROL => Key::RControl,
        VK_RIGHT => Key::RightArrow,
        VK_RMENU => Key::RMenu,
        VK_RSHIFT => Key::RShift,
        VK_RWIN => Key::RWin,
        VK_SCROLL => Key::Scroll,
        VK_SELECT => Key::Select,
        VK_SEPARATOR => Key::Separator,
        VK_SHIFT => Key::Shift,
        VK_SLEEP => Key::Sleep,
        VK_SPACE => Key::Space,
        VK_SUBTRACT => Key::Subtract,
        VK_TAB => Key::Tab,
        VK_UP => Key::UpArrow,
        VK_VOLUME_DOWN => Key::VolumeDown,
        VK_VOLUME_MUTE => Key::VolumeMute,
        VK_VOLUME_UP => Key::VolumeUp,
        VK_XBUTTON1 => Key::XButton1,
        VK_XBUTTON2 => Key::XButton2,
        VK_ZOOM => Key::Zoom,
        _ => to_unicode(vk_code, scan_code, keystate)
            .map_or_else(|| Key::Other(vk_code), Key::Unicode),
    }
}

#[derive(Debug, Deref)]
struct Keystate([u8; 256]);

fn get_keystate() -> Keystate {
    let mut keystate = [0u8; 256];

    for &mod_vk in [
        VK_SHIFT,
        VK_LSHIFT,
        VK_RSHIFT,
        VK_CONTROL,
        VK_LCONTROL,
        VK_RCONTROL,
        VK_MENU,
        VK_LMENU,
        VK_RMENU,
        VK_LWIN,
        VK_RWIN,
        VK_CAPITAL,
    ]
    .iter()
    {
        let s = unsafe { GetKeyState(mod_vk.0 as i32) as u16 };
        if (s & 0x8000) != 0 {
            keystate[mod_vk.0 as usize] |= 0x80;
        } // key is down
        if (s & 1) != 0 {
            keystate[mod_vk.0 as usize] |= 0x01;
        } // toggle (CapsLock)
    }

    keystate[VK_SHIFT.0 as usize] = keystate[VK_LSHIFT.0 as usize] | keystate[VK_RSHIFT.0 as usize];
    keystate[VK_CONTROL.0 as usize] =
        keystate[VK_LCONTROL.0 as usize] | keystate[VK_RCONTROL.0 as usize];
    keystate[VK_MENU.0 as usize] = keystate[VK_LMENU.0 as usize] | keystate[VK_RMENU.0 as usize];

    Keystate(keystate)
}

fn to_unicode(vk_code: u32, scan_code: u32, keystate: &Keystate) -> Option<char> {
    let mut buf = [0; 8];

    // On Windows 10 1607 or more recent, don't change the global key state
    const TOUNICODEEX_NO_STATE_CHANGE: u32 = 1 << 2;

    // Only call once; returns:
    //  >0 : number of UTF-16 units written
    //  -1 : dead key set (no char committed yet)
    //   0 : no translation
    let res = unsafe {
        ToUnicodeEx(
            vk_code,
            scan_code,
            keystate,
            &mut buf,
            TOUNICODEEX_NO_STATE_CHANGE,
            Some(get_keyboard_layout()),
        )
    };

    if res > 0 {
        String::from_utf16(&buf[..(res as usize)])
            .ok()?
            .chars()
            .next()
    } else {
        None
    }
}

fn get_keyboard_layout() -> HKL {
    unsafe { GetKeyboardLayout(GetWindowThreadProcessId(GetForegroundWindow(), None)) }
}
