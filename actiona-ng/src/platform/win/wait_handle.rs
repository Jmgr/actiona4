#![allow(unsafe_code)]

use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::sync::oneshot;
use windows::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0},
    System::Threading::{
        INFINITE, RegisterWaitForSingleObject, UnregisterWaitEx, WT_EXECUTEINWAITTHREAD,
        WT_EXECUTEONLYONCE, WaitForSingleObject,
    },
};

use crate::platform::win::safe_handle::SafeHandle;

struct Waiting {
    rx: oneshot::Receiver<()>,
    wait_object: SafeHandle,
    tx_ptr: *mut Option<oneshot::Sender<()>>,
}

unsafe impl Sync for Waiting {}
unsafe impl Send for Waiting {}

impl Drop for Waiting {
    fn drop(&mut self) {
        unsafe {
            if let Err(err) = UnregisterWaitEx(self.wait_object.as_raw(), None) {
                panic!("failed to unregister: {}", err);
            }
            drop(Box::from_raw(self.tx_ptr));
        }
    }
}

pub(crate) struct WaitHandle {
    handle: HANDLE,
    waiting: Option<Waiting>,
}

impl WaitHandle {
    pub fn new(handle: HANDLE) -> Self {
        Self {
            handle,
            waiting: None,
        }
    }

    fn is_signaled(&self) -> bool {
        unsafe { WaitForSingleObject(self.handle, 0) == WAIT_OBJECT_0 }
    }
}

impl Future for WaitHandle {
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let inner = Pin::get_mut(self);
        loop {
            if let Some(ref mut w) = inner.waiting {
                match Pin::new(&mut w.rx).poll(cx) {
                    Poll::Ready(Ok(())) => {}
                    Poll::Ready(Err(_)) => panic!("should not be canceled"),
                    Poll::Pending => return Poll::Pending,
                }
                return Poll::Ready(Ok(()));
            }

            if inner.is_signaled() {
                return Poll::Ready(Ok(()));
            }

            let (tx, rx) = oneshot::channel();
            let tx_ptr = Box::into_raw(Box::new(Some(tx)));
            let mut wait_object = INVALID_HANDLE_VALUE;

            if let Err(err) = unsafe {
                RegisterWaitForSingleObject(
                    &mut wait_object,
                    inner.handle,
                    Some(callback),
                    Some(tx_ptr as *mut _),
                    INFINITE,
                    WT_EXECUTEINWAITTHREAD | WT_EXECUTEONLYONCE,
                )
            } {
                drop(unsafe { Box::from_raw(tx_ptr) });
                return Poll::Ready(Err(err.into()));
            };

            inner.waiting = Some(Waiting {
                rx,
                wait_object: SafeHandle::new(wait_object),
                tx_ptr,
            });
        }
    }
}

unsafe extern "system" fn callback(ptr: *mut std::ffi::c_void, _timer_fired: bool) {
    let complete = unsafe { &mut *(ptr as *mut Option<oneshot::Sender<()>>) };
    let _ = complete.take().unwrap().send(());
}

#[cfg(test)]
mod tests {
    use crate::platform::win::safe_handle::SafeHandle;

    use super::WaitHandle;
    use std::time::Duration;
    use tokio::join;
    use tokio::time::{sleep, timeout};
    use windows::Win32::Foundation::WAIT_OBJECT_0;
    use windows::Win32::{
        Foundation::HANDLE,
        System::Threading::{
            CREATE_EVENT_INITIAL_SET, CREATE_EVENT_MANUAL_RESET, CreateEventExW, EVENT_ALL_ACCESS,
            SetEvent, WaitForSingleObject,
        },
    };

    fn create_event(initial_set: bool) -> HANDLE {
        let mut flags = CREATE_EVENT_MANUAL_RESET;
        if initial_set {
            flags |= CREATE_EVENT_INITIAL_SET;
        }
        unsafe {
            CreateEventExW(None, None, flags, EVENT_ALL_ACCESS.0).expect("CreateEventExW failed")
        }
    }

    #[tokio::test]
    async fn wait_handle_signals_immediately() {
        let handle = SafeHandle::try_new(create_event(true)).unwrap();
        timeout(Duration::from_secs(1), WaitHandle::new(handle.as_raw()))
            .await
            .expect("WaitHandle should resolve immediately")
            .unwrap();
    }

    #[tokio::test]
    async fn wait_handle_signals_later_without_spawn() {
        let handle = SafeHandle::try_new(create_event(false)).unwrap();

        // Two futures in the SAME task: one waits, one sleeps then signals.
        let waiter = WaitHandle::new(handle.as_raw());
        let signaler = async {
            sleep(Duration::from_millis(50)).await;
            unsafe { SetEvent(handle.as_raw()).unwrap() };
        };

        timeout(Duration::from_secs(2), async {
            let (wait_res, _) = join!(waiter, signaler);
            wait_res
        })
        .await
        .expect("WaitHandle should resolve after SetEvent")
        .unwrap();
    }

    #[tokio::test]
    async fn wait_handle_cancel_safe_drop() {
        unsafe {
            let handle = SafeHandle::try_new(create_event(false)).unwrap();

            // Start waiting but enforce a short timeout so the future gets dropped while pending.
            let waiter = WaitHandle::new(handle.as_raw());
            let timed = timeout(Duration::from_millis(50), waiter).await;
            assert!(
                timed.is_err(),
                "expect timeout so the WaitHandle future is dropped while a wait is registered"
            );

            // Now signal the handle after we've dropped the first future.
            SetEvent(handle.as_raw()).unwrap();

            // A fresh WaitHandle on the same HANDLE should resolve immediately.
            let res = timeout(Duration::from_secs(1), WaitHandle::new(handle.as_raw())).await;
            assert!(res.is_ok(), "new WaitHandle should resolve after signaling");
        }
    }

    #[tokio::test]
    async fn wait_handle_manual_polling_matches_native() {
        unsafe {
            let handle = SafeHandle::try_new(create_event(false)).unwrap();

            // Before signaling, both APIs should report "not signaled".
            assert_ne!(WaitForSingleObject(handle.as_raw(), 0), WAIT_OBJECT_0);
            assert!(
                timeout(Duration::from_millis(50), WaitHandle::new(handle.as_raw()))
                    .await
                    .is_err()
            );

            // After signaling, both are signaled.
            SetEvent(handle.as_raw()).ok().unwrap();
            assert_eq!(WaitForSingleObject(handle.as_raw(), 0), WAIT_OBJECT_0);
            assert!(
                timeout(Duration::from_secs(1), WaitHandle::new(handle.as_raw()))
                    .await
                    .is_ok()
            );
        }
    }
}
