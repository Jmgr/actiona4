#![allow(unsafe_code)]

use tracing::error;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::StationsAndDesktops::{CloseDesktop, HDESK},
};

#[repr(transparent)]
#[derive(derive_more::Deref)]
pub struct SafeHandle {
    inner: HANDLE,
}

unsafe impl Send for SafeHandle {}
unsafe impl Sync for SafeHandle {}

impl Drop for SafeHandle {
    fn drop(&mut self) {
        unsafe {
            if let Err(err) = CloseHandle(self.inner) {
                error!("CloseHandle failed: {err}");
            }
        }
    }
}

impl From<HANDLE> for SafeHandle {
    fn from(value: HANDLE) -> Self {
        Self { inner: value }
    }
}

#[repr(transparent)]
#[derive(derive_more::Deref)]
pub struct SafeDesktopHandle {
    inner: HDESK,
}

unsafe impl Send for SafeDesktopHandle {}
unsafe impl Sync for SafeDesktopHandle {}

impl Drop for SafeDesktopHandle {
    fn drop(&mut self) {
        unsafe {
            if let Err(err) = CloseDesktop(self.inner) {
                error!("CloseDesktop failed: {err}");
            }
        }
    }
}

impl From<HDESK> for SafeDesktopHandle {
    fn from(value: HDESK) -> Self {
        Self { inner: value }
    }
}
