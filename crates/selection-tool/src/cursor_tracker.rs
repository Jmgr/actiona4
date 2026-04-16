use std::{thread, time::Duration};

use winit::{
    dpi::PhysicalPosition,
    event_loop::EventLoopProxy,
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::Window,
};
use x11rb::{
    connection::Connection,
    protocol::{
        Event,
        xproto::{ConnectionExt, EventMask, GrabMode, GrabStatus},
    },
};

use crate::events::AppEvent;

pub fn get_window_xid(window: &Window) -> Option<u32> {
    match window.window_handle().ok()?.as_raw() {
        RawWindowHandle::Xcb(handle) => Some(handle.window.get()),
        RawWindowHandle::Xlib(handle) => Some(handle.window as u32),
        _ => None,
    }
}

/// Grab the pointer globally so we receive motion and button release even when
/// the cursor is over windows above ours.
pub fn spawn_cursor_tracker(
    proxy: EventLoopProxy<AppEvent>,
    window_xid: u32,
    desktop_origin: PhysicalPosition<i32>,
) {
    thread::spawn(move || {
        let Ok((connection, screen_number)) = x11rb::connect(None) else {
            return;
        };
        let root_window = connection.setup().roots[screen_number].root;

        let grab_succeeded = (|| -> Option<()> {
            let reply = connection
                .grab_pointer(
                    true,
                    window_xid,
                    EventMask::POINTER_MOTION | EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE,
                    GrabMode::ASYNC,
                    GrabMode::ASYNC,
                    root_window,
                    x11rb::NONE,
                    x11rb::CURRENT_TIME,
                )
                .ok()?
                .reply()
                .ok()?;
            if reply.status != GrabStatus::SUCCESS {
                return None;
            }
            connection.flush().ok()?;
            Some(())
        })();

        if grab_succeeded.is_none() {
            loop {
                if let Ok(Ok(reply)) = connection
                    .query_pointer(root_window)
                    .map(|cookie| cookie.reply())
                {
                    let x_position = reply.root_x as f64 - f64::from(desktop_origin.x);
                    let y_position = reply.root_y as f64 - f64::from(desktop_origin.y);
                    if proxy
                        .send_event(AppEvent::CursorMoved(PhysicalPosition::new(
                            x_position, y_position,
                        )))
                        .is_err()
                    {
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(8));
            }
            return;
        }

        while let Ok(event) = connection.wait_for_event() {
            match event {
                Event::MotionNotify(motion_event) => {
                    let x_position = motion_event.root_x as f64 - f64::from(desktop_origin.x);
                    let y_position = motion_event.root_y as f64 - f64::from(desktop_origin.y);
                    if proxy
                        .send_event(AppEvent::CursorMoved(PhysicalPosition::new(
                            x_position, y_position,
                        )))
                        .is_err()
                    {
                        break;
                    }
                }
                Event::ButtonRelease(button_event) if button_event.detail == 1 => {
                    let x_position = button_event.root_x as f64 - f64::from(desktop_origin.x);
                    let y_position = button_event.root_y as f64 - f64::from(desktop_origin.y);
                    let _ = proxy.send_event(AppEvent::Click(PhysicalPosition::new(
                        x_position, y_position,
                    )));
                    break;
                }
                _ => {}
            }
        }
    });
}
