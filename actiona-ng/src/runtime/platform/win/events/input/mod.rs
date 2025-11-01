#![allow(unsafe_code)]

use std::marker::PhantomData;

use eyre::Result;
use tracing::error;
use windows::Win32::UI::WindowsAndMessaging::{
    HOOKPROC, MSG, SetWindowsHookExW, WINDOWS_HOOK_ID, WM_APP,
};

use crate::{
    platform::win::safe_handle::SafeHookHandle, runtime::platform::win::MessagePumpRunner,
};

pub mod keyboard;
pub mod mouse;

const MSG_START: u32 = WM_APP + 1;
const MSG_STOP: u32 = WM_APP + 2;

#[derive(Debug, Default)]
pub struct LowLevelHookRunner<HS: HookSpec> {
    hook: Option<SafeHookHandle>,
    _phantom_data: PhantomData<HS>,
}

impl<HS: HookSpec + Default> MessagePumpRunner for LowLevelHookRunner<HS> {
    fn new() -> Result<Self>
    where
        Self: Sized,
    {
        Ok(Self::default())
    }
    fn on_message(&mut self, msg: &MSG) {
        match msg.message {
            MSG_START => self.start(),
            MSG_STOP => self.stop(),
            _ => {
                error!("unknown message: {:?}", msg);
            }
        }
    }
}

impl<HS: HookSpec + Default> LowLevelHookRunner<HS> {
    pub fn start(&mut self) {
        match unsafe { SetWindowsHookExW(HS::ID, HS::proc(), None, 0) } {
            Ok(hook) => {
                if hook.is_invalid() {
                    error!("SetWindowsHookExW returned an invalid Windows hook");
                    return;
                }

                self.hook = Some(SafeHookHandle::new(hook));
            }
            Err(err) => {
                error!("SetWindowsHookExW failed with {err}");
            }
        }
    }

    pub fn stop(&mut self) {
        drop(self.hook.take());
    }
}

pub trait HookSpec {
    const ID: WINDOWS_HOOK_ID;
    fn proc() -> HOOKPROC;
}
