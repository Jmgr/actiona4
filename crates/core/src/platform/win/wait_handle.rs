#![allow(unsafe_code, dead_code, clippy::non_send_fields_in_send_ty)]

use std::{
    ffi::c_void,
    io,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::sync::oneshot;
use tracing::error;
use windows::Win32::{
    Foundation::{HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0},
    System::Threading::{
        INFINITE, RegisterWaitForSingleObject, UnregisterWaitEx, WT_EXECUTEINWAITTHREAD,
        WT_EXECUTEONLYONCE, WaitForSingleObject,
    },
};

/// A live `RegisterWaitForSingleObject` registration together with the callback state it points
/// at.
///
/// The two are inseparable: the boxed sender may only be freed once the registration is gone and
/// no callback can still be running, so both are released together, and in that order, by `Drop`.
struct WaitRegistration {
    // Not a `SafeHandle`: a wait object is not a kernel object handle and is released with
    // `UnregisterWaitEx`, never with `CloseHandle`.
    wait_object: HANDLE,
    tx_ptr: *mut Option<oneshot::Sender<()>>,
}

// SAFETY: the registration owns the callback state outright, and the callback only completes the
// one-shot sender it was given.
unsafe impl Sync for WaitRegistration {}
// SAFETY: as above; nothing in the registration is bound to the thread that created it.
unsafe impl Send for WaitRegistration {}

impl WaitRegistration {
    /// Registers `handle` with the thread pool, returning the registration and the receiver that
    /// the callback completes once the handle is signaled.
    fn register(handle: HANDLE) -> io::Result<(Self, oneshot::Receiver<()>)> {
        let (tx, rx) = oneshot::channel();
        let tx_ptr = Box::into_raw(Box::new(Some(tx)));
        let mut wait_object = INVALID_HANDLE_VALUE;

        // SAFETY: `wait_object` and `tx_ptr` are writable valid storage; the callback owns the
        // boxed sender until the registration is unregistered.
        if let Err(err) = unsafe {
            RegisterWaitForSingleObject(
                &raw mut wait_object,
                handle,
                Some(callback),
                #[allow(clippy::as_conversions)] // pointer cast
                Some(tx_ptr.cast::<c_void>()),
                INFINITE,
                WT_EXECUTEINWAITTHREAD | WT_EXECUTEONLYONCE,
            )
        } {
            // SAFETY: registration failed, so no callback can access the boxed sender.
            drop(unsafe { Box::from_raw(tx_ptr) });
            return Err(err.into());
        }

        Ok((
            Self {
                wait_object,
                tx_ptr,
            },
            rx,
        ))
    }
}

impl Drop for WaitRegistration {
    fn drop(&mut self) {
        // Passing `INVALID_HANDLE_VALUE` as the completion event makes `UnregisterWaitEx` block
        // until any in-flight callback has returned. With a null event it returns
        // `ERROR_IO_PENDING` while the callback is still running, and freeing the sender below
        // would then race with it. The callback only completes a one-shot channel, so the wait is
        // short, and it never runs on this thread, so it cannot deadlock.
        // SAFETY: `wait_object` was produced by `RegisterWaitForSingleObject` and is unregistered
        // exactly once, here.
        let unregistered =
            unsafe { UnregisterWaitEx(self.wait_object, Some(INVALID_HANDLE_VALUE)) };
        if let Err(err) = unregistered {
            // The callback may still hold the sender, so leak it rather than risk a use after free.
            error!("UnregisterWaitEx failed: {err}");
            return;
        }
        // SAFETY: the registration is gone and all callbacks have completed, so nothing else can
        // reach the boxed sender created alongside `wait_object`.
        drop(unsafe { Box::from_raw(self.tx_ptr) });
    }
}

struct Waiting {
    rx: oneshot::Receiver<()>,
    /// Kept alive for as long as the wait is pending; releasing it cancels the wait.
    registration: WaitRegistration,
}

pub struct WaitHandle {
    handle: HANDLE,
    waiting: Option<Waiting>,
}

impl WaitHandle {
    pub const fn new(handle: HANDLE) -> Self {
        Self {
            handle,
            waiting: None,
        }
    }

    fn is_signaled(&self) -> bool {
        // SAFETY: `handle` is supplied by the caller and is only queried with a zero timeout.
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

            let (registration, rx) = match WaitRegistration::register(inner.handle) {
                Ok(registered) => registered,
                Err(err) => return Poll::Ready(Err(err)),
            };

            inner.waiting = Some(Waiting { rx, registration });
        }
    }
}

#[allow(clippy::as_conversions)] // pointer cast required by Windows callback API
unsafe extern "system" fn callback(ptr: *mut c_void, _timer_fired: bool) {
    // SAFETY: `ptr` was created from `Box<Option<Sender>>` when the wait was registered.
    let complete = unsafe { &mut *ptr.cast::<Option<oneshot::Sender<()>>>() };
    if let Some(sender) = complete.take() {
        _ = sender.send(());
    }
}

#[cfg(test)]
mod tests {
    use std::{future::poll_fn, task::Poll, time::Duration};

    use tokio::{
        join,
        time::{sleep, timeout},
    };
    use windows::Win32::{
        Foundation::{HANDLE, WAIT_OBJECT_0},
        System::Threading::{
            CREATE_EVENT_INITIAL_SET, CREATE_EVENT_MANUAL_RESET, CreateEventExW, EVENT_ALL_ACCESS,
            SetEvent, WaitForSingleObject,
        },
    };

    use super::WaitHandle;
    use crate::platform::win::safe_handle::SafeHandle;

    fn create_event(initial_set: bool) -> HANDLE {
        let mut flags = CREATE_EVENT_MANUAL_RESET;
        if initial_set {
            flags |= CREATE_EVENT_INITIAL_SET;
        }
        // SAFETY: CreateEventExW is called with valid flags and no optional names or security attributes.
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
            // SAFETY: `handle` wraps the event created for this test.
            unsafe { SetEvent(handle.as_raw()).unwrap() };
        };

        timeout(Duration::from_secs(2), async {
            let (wait_res, ()) = join!(waiter, signaler);
            wait_res
        })
        .await
        .expect("WaitHandle should resolve after SetEvent")
        .unwrap();
    }

    /// Dropping the future right after signaling unregisters the wait while the callback is very
    /// likely still running in the wait thread. That is the case `UnregisterWaitEx` reports as
    /// `ERROR_IO_PENDING` unless it is told to wait for in-flight callbacks, and the case where
    /// freeing the callback state too early would be a use after free.
    #[tokio::test]
    async fn wait_handle_drop_racing_with_callback() {
        for _ in 0..200 {
            let handle = SafeHandle::try_new(create_event(false)).unwrap();
            let mut waiter = Box::pin(WaitHandle::new(handle.as_raw()));

            // The first poll registers the wait; the event is not signaled yet, so it is pending.
            let first = poll_fn(|cx| Poll::Ready(waiter.as_mut().poll(cx))).await;
            assert!(first.is_pending(), "the event has not been signaled yet");

            // SAFETY: `handle` wraps the event created for this iteration.
            unsafe { SetEvent(handle.as_raw()).unwrap() };

            drop(waiter);
        }
    }

    #[tokio::test]
    #[expect(
        clippy::unnecessary_safety_comment,
        reason = "the test groups related event-handle FFI calls in one audited block"
    )]
    async fn wait_handle_cancel_safe_drop() {
        // SAFETY: all calls use the event handle created within this test.
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
    #[expect(
        clippy::unnecessary_safety_comment,
        reason = "the test groups related event-handle FFI calls in one audited block"
    )]
    async fn wait_handle_manual_polling_matches_native() {
        // SAFETY: all calls use the event handle created within this test.
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
