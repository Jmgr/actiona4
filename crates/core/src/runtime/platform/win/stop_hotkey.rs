#![allow(unsafe_code)]

use std::{
    process,
    sync::atomic::{AtomicBool, Ordering},
};

use color_eyre::{Result, eyre::eyre};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use windows::{
    Win32::{
        Foundation::{ERROR_HOTKEY_ALREADY_REGISTERED, HWND, LPARAM, WPARAM},
        UI::{
            Input::KeyboardAndMouse::{
                HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN,
                RegisterHotKey, VIRTUAL_KEY,
            },
            WindowsAndMessaging::{
                ChangeWindowMessageFilterEx, HWND_BROADCAST, KillTimer, MSGFLT_ALLOW, PostMessageW,
                RegisterWindowMessageW, SetTimer,
            },
        },
    },
    core::{Error, HSTRING},
};

use crate::runtime::hotkey::Hotkey;

const HOTKEY_ID: i32 = 1;
const RETRY_TIMER_ID: usize = 1;
const RETRY_INTERVAL_MS: u32 = 250;

/// Global hotkey that stops every running instance sharing it.
///
/// Only one process can register a given key combination, so the first instance owns it and
/// the others retry on a timer until it is released. When the owner receives `WM_HOTKEY`, it
/// broadcasts a registered window message named after the hotkey; every instance (owner
/// included) stops when its message window receives it.
///
/// `RegisterHotKey` only reports the registered combination: no other key press is seen.
///
/// All methods taking a window must be called on the thread that owns it. Destroying that
/// window releases the hotkey.
#[derive(Debug)]
pub struct StopHotkey {
    hotkey: Hotkey,
    modifiers: HOT_KEY_MODIFIERS,
    virtual_key: u32,
    broadcast_message: u32,
    cancellation_token: CancellationToken,
    reported_busy: AtomicBool,
}

impl StopHotkey {
    pub fn new(hotkey: Hotkey, cancellation_token: CancellationToken) -> Result<Self> {
        let virtual_key = VIRTUAL_KEY::try_from(hotkey.key)
            .map_err(|err| eyre!("stop hotkey {hotkey}: {err}"))?;

        let mut modifiers = MOD_NOREPEAT;
        for (enabled, modifier) in [
            (hotkey.ctrl, MOD_CONTROL),
            (hotkey.alt, MOD_ALT),
            (hotkey.shift, MOD_SHIFT),
            (hotkey.meta, MOD_WIN),
        ] {
            if enabled {
                modifiers |= modifier;
            }
        }

        // SAFETY: the HSTRING outlives this synchronous call.
        let broadcast_message =
            unsafe { RegisterWindowMessageW(&HSTRING::from(format!("ActionaRunStop {hotkey}"))) };
        if broadcast_message == 0 {
            return Err(Error::from_thread().into());
        }

        Ok(Self {
            hotkey,
            modifiers,
            virtual_key: u32::from(virtual_key.0),
            broadcast_message,
            cancellation_token,
            reported_busy: AtomicBool::new(false),
        })
    }

    /// Lets the stop broadcast through UIPI when this instance is elevated, then registers the
    /// hotkey.
    pub fn start(&self, window: HWND) {
        // SAFETY: `window` is a live window owned by the calling thread.
        if let Err(err) = unsafe {
            ChangeWindowMessageFilterEx(window, self.broadcast_message, MSGFLT_ALLOW, None)
        } {
            warn!("failed to allow the stop hotkey broadcast message: {err}");
        }

        if !self.try_register(window) {
            // SAFETY: `window` is a live window owned by the calling thread; the timer is
            // delivered to its window procedure as WM_TIMER.
            unsafe {
                SetTimer(Some(window), RETRY_TIMER_ID, RETRY_INTERVAL_MS, None);
            }
        }
    }

    pub fn on_retry_timer(&self, window: HWND, timer_id: usize) {
        if timer_id == RETRY_TIMER_ID && self.try_register(window) {
            // SAFETY: the timer was created on `window` by this thread.
            unsafe {
                _ = KillTimer(Some(window), RETRY_TIMER_ID);
            }
        }
    }

    pub fn on_hotkey(&self, hotkey_id: usize) {
        if hotkey_id != usize::try_from(HOTKEY_ID).unwrap_or_default() {
            return;
        }

        info!("stop hotkey pressed");

        // SAFETY: posting a registered message to all top-level windows passes no pointers.
        if let Err(err) = unsafe {
            PostMessageW(
                Some(HWND_BROADCAST),
                self.broadcast_message,
                WPARAM(usize::try_from(process::id()).unwrap_or_default()),
                LPARAM(0),
            )
        } {
            warn!("failed to stop the other instances: {err}");
        }

        self.cancellation_token.cancel();
    }

    pub fn on_message(&self, message: u32) -> bool {
        if message != self.broadcast_message {
            return false;
        }

        info!("stop requested by the stop hotkey");
        self.cancellation_token.cancel();
        true
    }

    /// Returns whether the hotkey is registered. Failing because another process holds it is not
    /// an error: the caller retries later.
    fn try_register(&self, window: HWND) -> bool {
        // SAFETY: `window` is a live window owned by the calling thread.
        match unsafe { RegisterHotKey(Some(window), HOTKEY_ID, self.modifiers, self.virtual_key) } {
            Ok(()) => {
                info!("stop hotkey {} registered", self.hotkey);
                true
            }
            Err(err) => {
                let is_busy = err.code() == ERROR_HOTKEY_ALREADY_REGISTERED.to_hresult();
                if !self.reported_busy.swap(true, Ordering::Relaxed) {
                    if is_busy {
                        // Usually another actiona-run instance, which stops us through the
                        // broadcast message anyway; it can also be an unrelated application.
                        info!(
                            "stop hotkey {} is held by another process, retrying",
                            self.hotkey
                        );
                    } else {
                        warn!("failed to register the stop hotkey {}: {err}", self.hotkey);
                    }
                }
                false
            }
        }
    }
}
