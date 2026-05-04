use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    sync::Arc,
};

use derive_more::{Deref, From};
use libwmctl::{Position, Shape, window, windows};
use parking_lot::Mutex;
use satint::{Si32, Su32};
use tokio_util::sync::CancellationToken;
use types::{point::point, size::size};
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt as _, EventMask},
    rust_connection::RustConnection,
};
use x11rb_async::protocol::xproto::{
    ChangeWindowAttributesAux, ConnectionExt as AsyncConnectionExt, EventMask as AsyncEventMask,
};

use crate::{
    api::{
        point::Point,
        rect::{Rect, rect},
        size::Size,
        windows::platform::{Registry, Result, WindowId, WindowsHandler},
    },
    cancel_on,
    runtime::Runtime,
};

pub mod events;

#[derive(Clone, Deref, From)]
pub struct WindowHandle(libwmctl::Window);

impl Debug for WindowHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Window").field(&self.0.id).finish()
    }
}

impl PartialEq for WindowHandle {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

impl Eq for WindowHandle {}

impl Hash for WindowHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.id.hash(state);
    }
}

#[derive(Debug)]
pub struct X11WindowHandler {
    inner: Mutex<Registry<WindowHandle>>,
    runtime: Arc<Runtime>,
}

impl WindowsHandler for X11WindowHandler {
    fn all(&self) -> Result<Vec<WindowId>> {
        let windows = windows(false)?;

        Ok(self
            .inner
            .lock()
            .update(windows.into_iter().map(WindowHandle)))
    }

    fn is_visible(&self, id: WindowId) -> Result<bool> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        let state = handle.mapped()?;

        use libwmctl::MapState::*;
        Ok(match state {
            Unmapped | Unviewable => false,
            Viewable => true,
        })
    }

    fn title(&self, id: WindowId) -> Result<String> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        Ok(handle.name()?)
    }

    fn classname(&self, id: WindowId) -> Result<String> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        Ok(handle.class()?)
    }

    fn close(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        let geometry = connection.get_geometry(handle.id)?.reply()?;
        let root = geometry.root;
        let close_atom = platform.atoms()._NET_CLOSE_WINDOW;

        connection.send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            ClientMessageEvent::new(32, handle.id, close_atom, [0, 0, 0, 0, 0]),
        )?;
        connection.flush()?;

        Ok(())
    }

    fn process_id(&self, id: WindowId) -> Result<u32> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        Ok(Si32::from(handle.pid()?).to_unsigned().into())
    }

    fn rect(&self, id: WindowId) -> Result<Rect> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        let geometry = connection.get_geometry(handle.id)?.reply()?;
        let coordinates = connection
            .translate_coordinates(handle.id, geometry.root, 0, 0)?
            .reply()?;
        let extents = self.frame_extents(connection, &handle)?.unwrap_or_default();
        let geometry_size = size(geometry.width, geometry.height);
        let coordinates = point(coordinates.dst_x, coordinates.dst_y);
        let size = Size::new(
            geometry_size.width - extents.left - extents.right,
            geometry_size.height - extents.top - extents.bottom,
        );

        Ok(rect(
            Point::new(
                coordinates.x + extents.left.to_signed(),
                coordinates.y + extents.top.to_signed(),
            ),
            size,
        ))
    }

    fn set_active(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        let geometry = connection.get_geometry(handle.id)?.reply()?;
        let root = geometry.root;
        let active_window_atom = platform.atoms()._NET_ACTIVE_WINDOW;

        let timestamp = 0;
        connection.send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            ClientMessageEvent::new(32, handle.id, active_window_atom, [1, timestamp, 0, 0, 0]),
        )?;

        connection.flush()?;

        Ok(())
    }

    fn minimize(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();
        let geometry = connection.get_geometry(handle.id)?.reply()?;
        let root = geometry.root;
        let wm_change_state = connection
            .intern_atom(false, b"WM_CHANGE_STATE")?
            .reply()?
            .atom;

        // ICCCM WM_STATE values: 1 = NormalState, 3 = IconicState.
        // WM_CHANGE_STATE expects one of these in data[0]; use IconicState for minimize.
        const ICCCM_WM_STATE_ICONIC: u32 = 3;

        connection.send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            ClientMessageEvent::new(
                32,
                handle.id,
                wm_change_state,
                [ICCCM_WM_STATE_ICONIC, 0, 0, 0, 0],
            ),
        )?;
        connection.flush()?;

        Ok(())
    }

    fn maximize(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        handle.maximize()?;
        Ok(())
    }

    fn set_position(&self, id: WindowId, position: Point) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        <libwmctl::Window as Clone>::clone(&handle)
            .pos(Position::Static(position.x.into(), position.y.into()))
            .place()?;
        Ok(())
    }

    fn position(&self, id: WindowId) -> Result<Point> {
        Ok(self.rect(id)?.top_left())
    }

    fn set_size(&self, id: WindowId, size: Size) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        <libwmctl::Window as Clone>::clone(&handle)
            .shape(Shape::Static(size.width.into(), size.height.into()))
            .place()?;
        Ok(())
    }

    fn size(&self, id: WindowId) -> Result<Size> {
        Ok(self.rect(id)?.size())
    }

    fn is_active(&self, id: WindowId) -> Result<bool> {
        let Some(active_id) = self.read_active_window_id()? else {
            return Ok(false);
        };
        let handle = self.inner.lock().get_handle(id)?.clone();
        handle.state()?;
        Ok(handle.0.id == active_id)
    }

    fn active_window(&self) -> Result<Option<WindowId>> {
        let Some(active_id) = self.read_active_window_id()? else {
            return Ok(None);
        };
        let handle = WindowHandle(window(active_id));
        Ok(Some(self.inner.lock().get_or_insert(handle)))
    }

    async fn wait_for_closed(
        &self,
        id: WindowId,
        runtime: Arc<Runtime>,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        use tracing::info;
        let window_id = self.inner.lock().get_handle(id)?.0.id;
        info!("wait_for_closed: waiting for window {:#x}", window_id);

        // Subscribe before doing anything async so we cannot miss an event that
        // arrives between here and the first recv() call.
        let mut receiver = runtime.platform().subscribe_window_events();

        // Subscribe to STRUCTURE_NOTIFY on the specific client window so that the
        // event loop receives DestroyNotify for that exact window ID.  With only
        // SUBSTRUCTURE_NOTIFY on root, DestroyNotify arrives for the WM frame
        // (a different window ID) rather than the client window, so the ID
        // comparison below would never match.
        let x11_connection = runtime.platform().x11_connection();
        let async_conn = x11_connection.async_connection();
        // Ignore errors: the window may already be gone.
        let _ = async_conn
            .change_window_attributes(
                window_id,
                &ChangeWindowAttributesAux::new().event_mask(AsyncEventMask::STRUCTURE_NOTIFY),
            )
            .await;

        // Use the same async connection for the existence check so the request
        // is ordered after change_window_attributes on the same socket.  A sync
        // connection check would race with the async connection: the server could
        // process the sync request before the STRUCTURE_NOTIFY registration, then
        // destroy the window, and we would never receive the DestroyNotify.
        let still_alive = match async_conn.get_window_attributes(window_id).await {
            Ok(cookie) => cookie.reply().await.is_ok(),
            Err(_) => false,
        };
        if !still_alive {
            info!("wait_for_closed: window {:#x} already gone", window_id);
            return Ok(());
        }

        loop {
            let event = cancel_on(&cancellation_token, receiver.recv()).await??;

            if let events::WindowEvent::Closed(closed_handle) = &event {
                info!(
                    "wait_for_closed: got Closed for {:#x}, waiting for {:#x}",
                    closed_handle.id, window_id
                );
            }

            if let events::WindowEvent::Closed(closed_handle) = event
                && closed_handle.id == window_id
            {
                return Ok(());
            }
        }
    }
}

#[derive(Default)]
struct FrameExtents {
    left: Su32,
    right: Su32,
    top: Su32,
    bottom: Su32,
}

impl X11WindowHandler {
    #[must_use]
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            inner: Mutex::new(Registry::default()),
            runtime,
        }
    }

    fn read_active_window_id(&self) -> Result<Option<u32>> {
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();
        let root = connection.setup().roots[0].root;
        let atom = platform.atoms()._NET_ACTIVE_WINDOW;
        let reply = connection
            .get_property(false, root, atom, AtomEnum::WINDOW, 0, 1)?
            .reply()?;
        Ok(reply
            .value32()
            .and_then(|mut iter| iter.next())
            .filter(|&id| id != 0))
    }

    fn frame_extents(
        &self,
        connection: &RustConnection,
        windows: &WindowHandle,
    ) -> Result<Option<FrameExtents>> {
        let platform = self.runtime.platform();
        let extents = connection
            .get_property(
                false,
                windows.id,
                platform.atoms()._NET_FRAME_EXTENTS,
                AtomEnum::CARDINAL,
                0,
                4,
            )?
            .reply()?;
        let extents = extents.value32();
        if let Some(mut extents) = extents {
            return Ok(Some(FrameExtents {
                left: extents.next().unwrap_or_default().into(),
                right: extents.next().unwrap_or_default().into(),
                top: extents.next().unwrap_or_default().into(),
                bottom: extents.next().unwrap_or_default().into(),
            }));
        }

        let extents = connection
            .get_property(
                false,
                windows.id,
                platform.atoms()._GTK_FRAME_EXTENTS,
                AtomEnum::CARDINAL,
                0,
                4,
            )?
            .reply()?;
        let extents = extents.value32();
        Ok(extents.map(|mut extents| FrameExtents {
            left: extents.next().unwrap_or_default().into(),
            right: extents.next().unwrap_or_default().into(),
            top: extents.next().unwrap_or_default().into(),
            bottom: extents.next().unwrap_or_default().into(),
        }))
    }
}
