#![allow(unsafe_code, dead_code)]

use std::{fmt, mem};

use tracing::error;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE, HWND},
    System::StationsAndDesktops::{CloseDesktop, HDESK},
    UI::{
        Accessibility::{HWINEVENTHOOK, UnhookWinEvent},
        WindowsAndMessaging::{DestroyWindow, HHOOK, UnhookWindowsHookEx},
    },
};

pub trait Handle {
    type Raw: Copy;

    const NAME: &'static str;

    fn is_valid(raw: Self::Raw) -> bool;
    fn close(raw: Self::Raw) -> windows_result::Result<()>;
}

#[repr(transparent)]
pub struct Safe<H: Handle> {
    raw: H::Raw,
}

impl<H: Handle> Safe<H> {
    pub const fn new(raw: H::Raw) -> Self {
        Self { raw }
    }

    pub fn try_new(raw: H::Raw) -> windows_result::Result<Self> {
        if H::is_valid(raw) {
            Ok(Self::new(raw))
        } else {
            Err(windows_result::Error::from_thread())
        }
    }

    pub const fn leak(self) -> H::Raw {
        let raw = self.raw;
        // Intentionally prevent `Drop`: ownership of the raw handle is transferred to the caller.
        #[expect(
            clippy::mem_forget,
            reason = "leaking transfers ownership of the raw Win32 handle"
        )]
        mem::forget(self);
        raw
    }

    pub const fn as_raw(&self) -> H::Raw {
        self.raw
    }
}

impl<H: Handle> Drop for Safe<H> {
    fn drop(&mut self) {
        if let Err(err) = H::close(self.raw) {
            error!("{} failed: {err}", H::NAME);
        }
    }
}

impl<K: Handle> fmt::Debug for Safe<K>
where
    K::Raw: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(K::NAME).field(&self.raw).finish()
    }
}

impl<K: Handle> AsRef<K::Raw> for Safe<K> {
    fn as_ref(&self) -> &K::Raw {
        &self.raw
    }
}

pub struct KindHandle;
impl Handle for KindHandle {
    type Raw = HANDLE;
    const NAME: &'static str = "CloseHandle";
    fn is_valid(raw: Self::Raw) -> bool {
        !raw.is_invalid()
    }
    fn close(raw: Self::Raw) -> windows_result::Result<()> {
        // SAFETY: `raw` is owned by `SafeHandle` and closed exactly once by its `Drop` implementation.
        unsafe { CloseHandle(raw) }
    }
}
pub type SafeHandle = Safe<KindHandle>;

pub struct KindDesktop;
impl Handle for KindDesktop {
    type Raw = HDESK;
    const NAME: &'static str = "CloseDesktop";
    fn is_valid(raw: Self::Raw) -> bool {
        !raw.is_invalid()
    }
    fn close(raw: Self::Raw) -> windows_result::Result<()> {
        // SAFETY: `raw` is owned by `SafeDesktopHandle` and closed exactly once on drop.
        unsafe { CloseDesktop(raw) }
    }
}
pub type SafeDesktopHandle = Safe<KindDesktop>;

pub struct KindHook;
impl Handle for KindHook {
    type Raw = HHOOK;
    const NAME: &'static str = "UnhookWindowsHookEx";
    fn is_valid(raw: Self::Raw) -> bool {
        !raw.is_invalid()
    }
    fn close(raw: Self::Raw) -> windows_result::Result<()> {
        // SAFETY: `raw` is owned by `SafeHookHandle` and unhooked exactly once on drop.
        unsafe { UnhookWindowsHookEx(raw) }
    }
}
pub type SafeHookHandle = Safe<KindHook>;
// SAFETY: the owning wrapper prevents duplicate cleanup and Win32 hook handles are thread-safe.
unsafe impl Send for SafeHookHandle {}

pub struct KindWindow;
impl Handle for KindWindow {
    type Raw = HWND;
    const NAME: &'static str = "DestroyWindow";
    fn is_valid(raw: Self::Raw) -> bool {
        !raw.is_invalid()
    }
    fn close(raw: Self::Raw) -> windows_result::Result<()> {
        // SAFETY: `raw` is owned by `SafeWindowHandle` and destroyed exactly once on drop.
        unsafe { DestroyWindow(raw) }
    }
}
pub type SafeWindowHandle = Safe<KindWindow>;
// SAFETY: the owning wrapper prevents duplicate cleanup and HWND values may cross threads.
unsafe impl Send for SafeWindowHandle {}

pub struct KindWinEventHook;
impl Handle for KindWinEventHook {
    type Raw = HWINEVENTHOOK;
    const NAME: &'static str = "UnhookWinEvent";
    fn is_valid(raw: Self::Raw) -> bool {
        !raw.is_invalid()
    }
    fn close(raw: Self::Raw) -> windows_result::Result<()> {
        // SAFETY: `raw` is owned by `SafeWinEventHook` and unhooked exactly once on drop.
        unsafe {
            let _ = UnhookWinEvent(raw);
        };
        Ok(())
    }
}
pub type SafeWinEventHook = Safe<KindWinEventHook>;
// SAFETY: the owning wrapper prevents duplicate cleanup and the hook can be released from this thread.
unsafe impl Send for SafeWinEventHook {}
