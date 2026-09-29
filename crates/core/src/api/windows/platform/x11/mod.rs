use std::{
    collections::HashSet,
    fmt::{self, Debug},
    sync::Arc,
};

use color_eyre::eyre::eyre;
use itertools::Itertools;
use parking_lot::Mutex;
use satint::{SaturatingInto, Su32};
use tokio_util::sync::CancellationToken;
use types::{point, size};
use x11rb::{
    connection::Connection,
    protocol::xproto::{
        Atom, AtomEnum, ClientMessageEvent, ConfigureWindowAux, ConnectionExt as _, EventMask,
        Gravity, MapState, UNMAP_NOTIFY_EVENT, UnmapNotifyEvent, Window,
    },
    rust_connection::RustConnection,
};
use x11rb_async::protocol::xproto::{
    ChangeWindowAttributesAux, ConnectionExt as AsyncConnectionExt, EventMask as AsyncEventMask,
};

use crate::{
    api::{
        point::Point,
        rect::Rect,
        size::Size,
        windows::platform::{Registry, Result, WindowId, WindowsHandler},
    },
    cancel_on,
    platform::x11::Atoms,
    runtime::Runtime,
};

pub mod events;

// ICCCM WM_STATE values: 1 = NormalState, 3 = IconicState.
// WM_CHANGE_STATE expects one of these in data[0]; use IconicState for minimize.
// The WM_STATE property holds the current one.
const ICCCM_WM_STATE_ICONIC: u32 = 3;

// EWMH _NET_WM_STATE actions.
const NET_WM_STATE_REMOVE: u32 = 0;
const NET_WM_STATE_ADD: u32 = 1;

// EWMH source indication for requests made on behalf of the user, as automation tools do.
const NET_MOVERESIZE_SOURCE_PAGER: u32 = 2;

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct WindowHandle {
    pub id: Window,
}

impl Debug for WindowHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Window").field(&self.id).finish()
    }
}

#[derive(Debug)]
pub struct X11WindowHandler {
    inner: Mutex<Registry<WindowHandle>>,
    /// X11 IDs of the windows withdrawn by `hide()`, see `hidden_windows`.
    hidden: Mutex<HashSet<Window>>,
    runtime: Arc<Runtime>,
}

impl WindowsHandler for X11WindowHandler {
    fn all(&self) -> Result<Vec<WindowId>> {
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();
        let root = x11_connection.screen().root;

        // Without a window manager there is no client list, and so no windows to report.
        let mut windows = connection
            .get_property(
                false,
                root,
                platform.atoms()._NET_CLIENT_LIST,
                AtomEnum::WINDOW,
                0,
                u32::MAX,
            )?
            .reply()?
            .value32()
            .map(Itertools::collect_vec)
            .unwrap_or_default();
        let hidden = self.hidden_windows(connection, &windows);
        windows.extend(hidden);

        Ok(self
            .inner
            .lock()
            .update(windows.into_iter().map(|id| WindowHandle { id })))
    }

    fn is_visible(&self, id: WindowId) -> Result<bool> {
        let handle = self.handle(id)?;
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        let attributes = connection.get_window_attributes(handle.id)?.reply()?;

        Ok(attributes.map_state == MapState::VIEWABLE)
    }

    fn title(&self, id: WindowId) -> Result<String> {
        let handle = self.handle(id)?;
        let platform = self.runtime.platform();
        let atoms = platform.atoms();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        for name_atom in [atoms._NET_WM_VISIBLE_NAME, atoms._NET_WM_NAME] {
            let reply = connection
                .get_property(false, handle.id, name_atom, atoms.UTF8_STRING, 0, u32::MAX)?
                .reply()?;
            if !reply.value.is_empty() {
                return Ok(String::from_utf8_lossy(&reply.value).into_owned());
            }
        }

        // The legacy WM_NAME is usually Latin-1 (STRING), but can be any text type.
        let reply = connection
            .get_property(
                false,
                handle.id,
                AtomEnum::WM_NAME,
                AtomEnum::ANY,
                0,
                u32::MAX,
            )?
            .reply()?;

        Ok(if reply.type_ == atoms.UTF8_STRING {
            String::from_utf8_lossy(&reply.value).into_owned()
        } else {
            decode_latin1(&reply.value)
        })
    }

    fn classname(&self, id: WindowId) -> Result<String> {
        let handle = self.handle(id)?;
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        // WM_CLASS holds two null-terminated strings: the instance name, then the class name.
        let reply = connection
            .get_property(
                false,
                handle.id,
                AtomEnum::WM_CLASS,
                AtomEnum::STRING,
                0,
                u32::MAX,
            )?
            .reply()?;
        let class = reply.value.split(|&byte| byte == 0).nth(1).unwrap_or(&[]);

        Ok(decode_latin1(class))
    }

    fn close(&self, id: WindowId) -> Result<()> {
        let handle = self.handle(id)?;
        let close_atom = self.runtime.platform().atoms()._NET_CLOSE_WINDOW;

        self.send_client_message(handle, close_atom, [0, 0, 0, 0, 0])
    }

    fn process_id(&self, id: WindowId) -> Result<Su32> {
        let handle = self.handle(id)?;
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        let reply = connection
            .get_property(
                false,
                handle.id,
                platform.atoms()._NET_WM_PID,
                AtomEnum::CARDINAL,
                0,
                1,
            )?
            .reply()?;
        let pid = reply
            .value32()
            .and_then(|mut values| values.next())
            .ok_or_else(|| eyre!("window does not report its process ID"))?;

        Ok(pid.into())
    }

    fn rect(&self, id: WindowId) -> Result<Rect> {
        let handle = self.handle(id)?;
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        let geometry = connection.get_geometry(handle.id)?.reply()?;
        let coordinates = connection
            .translate_coordinates(handle.id, geometry.root, 0, 0)?
            .reply()?;
        let margins = self.margins(connection, handle)?;

        let x = i32::from(coordinates.dst_x) - margins.left;
        let y = i32::from(coordinates.dst_y) - margins.top;
        let width = i32::from(geometry.width) + margins.left + margins.right;
        let height = i32::from(geometry.height) + margins.top + margins.bottom;

        Ok(Rect::new(point(x, y), size(width.max(0), height.max(0))))
    }

    fn set_active(&self, id: WindowId) -> Result<()> {
        let handle = self.handle(id)?;
        let active_window_atom = self.runtime.platform().atoms()._NET_ACTIVE_WINDOW;

        let timestamp = 0;
        self.send_client_message(handle, active_window_atom, [1, timestamp, 0, 0, 0])
    }

    fn minimize(&self, id: WindowId) -> Result<()> {
        let handle = self.handle(id)?;
        let wm_change_state = self.runtime.platform().atoms().WM_CHANGE_STATE;

        self.send_client_message(handle, wm_change_state, [ICCCM_WM_STATE_ICONIC, 0, 0, 0, 0])
    }

    fn maximize(&self, id: WindowId) -> Result<()> {
        let handle = self.handle(id)?;
        self.set_maximized(handle, true)
    }

    fn restore(&self, id: WindowId) -> Result<()> {
        let handle = self.handle(id)?;
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        if self.is_iconic(connection, handle)? || self.hidden.lock().contains(&handle.id) {
            // ICCCM 4.1.4: mapping an iconic or withdrawn window returns it to the normal state.
            connection.map_window(handle.id)?.check()?;
        } else {
            self.set_maximized(handle, false)?;
        }

        Ok(())
    }

    fn hide(&self, id: WindowId) -> Result<()> {
        let handle = self.handle(id)?;
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();
        let root = connection.get_geometry(handle.id)?.reply()?.root;

        // ICCCM 4.1.4: withdraw the window by unmapping it, then send a synthetic UnmapNotify to
        // the root so that the window manager also withdraws it when it is iconic (already unmapped).
        connection.unmap_window(handle.id)?;
        connection.send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            UnmapNotifyEvent {
                response_type: UNMAP_NOTIFY_EVENT,
                sequence: 0,
                event: root,
                window: handle.id,
                from_configure: false,
            },
        )?;
        connection.flush()?;

        self.hidden.lock().insert(handle.id);

        Ok(())
    }

    fn show(&self, id: WindowId) -> Result<()> {
        let handle = self.handle(id)?;
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        connection.map_window(handle.id)?.check()?;

        Ok(())
    }

    fn set_position(&self, id: WindowId, position: Point) -> Result<()> {
        let handle = self.handle(id)?;
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        // Window managers ignore geometry changes on maximized windows.
        if self.is_maximized(connection, handle)? {
            self.set_maximized(handle, false)?;
        }

        // The position is that of the frame, which includes the decorations but also the shadows.
        let shadows = read_extents(connection, handle, self.atoms()._GTK_FRAME_EXTENTS)?;
        let x = i32::from(position.x) - shadows.left;
        let y = i32::from(position.y) - shadows.top;

        self.move_resize(connection, handle, &ConfigureWindowAux::new().x(x).y(y))
    }

    fn position(&self, id: WindowId) -> Result<Point> {
        Ok(self.rect(id)?.top_left())
    }

    fn set_size(&self, id: WindowId, size: Size) -> Result<()> {
        let handle = self.handle(id)?;
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        // Window managers ignore geometry changes on maximized windows.
        if self.is_maximized(connection, handle)? {
            self.set_maximized(handle, false)?;
        }

        let margins = self.margins(connection, handle)?;
        let width: i32 = size.width.saturating_into();
        let height: i32 = size.height.saturating_into();
        // X11 windows cannot be empty.
        let width = u32::try_from((width - margins.left - margins.right).max(1))?;
        let height = u32::try_from((height - margins.top - margins.bottom).max(1))?;

        self.move_resize(
            connection,
            handle,
            &ConfigureWindowAux::new().width(width).height(height),
        )
    }

    fn size(&self, id: WindowId) -> Result<Size> {
        Ok(self.rect(id)?.size())
    }

    fn is_active(&self, id: WindowId) -> Result<bool> {
        let Some(active_id) = self.read_active_window_id()? else {
            return Ok(false);
        };
        let handle = self.handle(id)?;
        Ok(handle.id == active_id)
    }

    fn active_window(&self) -> Result<Option<WindowId>> {
        let Some(active_id) = self.read_active_window_id()? else {
            return Ok(None);
        };
        let handle = WindowHandle { id: active_id };
        Ok(Some(self.inner.lock().get_or_insert(handle)))
    }

    async fn wait_for_closed(
        &self,
        id: WindowId,
        runtime: Arc<Runtime>,
        cancellation_token: CancellationToken,
    ) -> Result<()> {
        use tracing::info;
        let window_id = self.handle(id)?.id;
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

/// Decodes an ICCCM `STRING` property, which is Latin-1 encoded.
fn decode_latin1(bytes: &[u8]) -> String {
    bytes.iter().copied().map(char::from).collect()
}

#[derive(Default)]
struct Margins {
    left: i32,
    right: i32,
    top: i32,
    bottom: i32,
}

impl X11WindowHandler {
    #[must_use]
    pub fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            inner: Mutex::new(Registry::default()),
            hidden: Mutex::new(HashSet::new()),
            runtime,
        }
    }

    fn handle(&self, id: WindowId) -> Result<WindowHandle> {
        self.inner.lock().get_handle(id).copied()
    }

    fn atoms(&self) -> &Atoms {
        self.runtime.platform().atoms()
    }

    /// Returns the windows withdrawn by `hide()` that the window manager no longer lists.
    ///
    /// Window managers drop withdrawn windows from `_NET_CLIENT_LIST`, so without this their IDs
    /// would be pruned from the registry on the next enumeration, and `show()` could no longer
    /// reach them. Windows are forgotten once the window manager lists them again or they are
    /// destroyed.
    fn hidden_windows(&self, connection: &RustConnection, managed: &[Window]) -> Vec<Window> {
        let mut hidden = self.hidden.lock();
        hidden.retain(|id| {
            !managed.contains(id)
                && connection
                    .get_window_attributes(*id)
                    .is_ok_and(|cookie| cookie.reply().is_ok())
        });

        hidden.iter().copied().collect_vec()
    }

    /// Sends a client message to the window manager about `handle`.
    fn send_client_message(
        &self,
        handle: WindowHandle,
        message_type: Atom,
        data: [u32; 5],
    ) -> Result<()> {
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();
        let root = connection.get_geometry(handle.id)?.reply()?.root;

        connection.send_event(
            false,
            root,
            EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY,
            ClientMessageEvent::new(32, handle.id, message_type, data),
        )?;
        connection.flush()?;

        Ok(())
    }

    fn set_maximized(&self, handle: WindowHandle, maximized: bool) -> Result<()> {
        let atoms = self.atoms();
        let action = if maximized {
            NET_WM_STATE_ADD
        } else {
            NET_WM_STATE_REMOVE
        };

        self.send_client_message(
            handle,
            atoms._NET_WM_STATE,
            [
                action,
                atoms._NET_WM_STATE_MAXIMIZED_HORZ,
                atoms._NET_WM_STATE_MAXIMIZED_VERT,
                0,
                0,
            ],
        )
    }

    fn is_maximized(&self, connection: &RustConnection, handle: WindowHandle) -> Result<bool> {
        let atoms = self.atoms();
        let reply = connection
            .get_property(
                false,
                handle.id,
                atoms._NET_WM_STATE,
                AtomEnum::ATOM,
                0,
                u32::MAX,
            )?
            .reply()?;

        Ok(reply.value32().is_some_and(|mut states| {
            states.any(|state| {
                state == atoms._NET_WM_STATE_MAXIMIZED_HORZ
                    || state == atoms._NET_WM_STATE_MAXIMIZED_VERT
            })
        }))
    }

    fn is_iconic(&self, connection: &RustConnection, handle: WindowHandle) -> Result<bool> {
        let wm_state = self.atoms().WM_STATE;
        let reply = connection
            .get_property(false, handle.id, wm_state, wm_state, 0, 1)?
            .reply()?;

        Ok(reply.value32().and_then(|mut values| values.next()) == Some(ICCCM_WM_STATE_ICONIC))
    }

    fn read_active_window_id(&self) -> Result<Option<Window>> {
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();
        let root = x11_connection.screen().root;
        let atom = platform.atoms()._NET_ACTIVE_WINDOW;
        let reply = connection
            .get_property(false, root, atom, AtomEnum::WINDOW, 0, 1)?
            .reply()?;
        Ok(reply
            .value32()
            .and_then(|mut iter| iter.next())
            .filter(|&id| id != 0))
    }

    /// Returns the margins between the client area of a window and its visible bounds.
    ///
    /// The visible bounds include the decorations drawn by the window manager
    /// (`_NET_FRAME_EXTENTS`), but not the invisible shadows that client-side decorated windows
    /// draw inside their client area (`_GTK_FRAME_EXTENTS`).
    fn margins(&self, connection: &RustConnection, handle: WindowHandle) -> Result<Margins> {
        let atoms = self.atoms();
        let decorations = read_extents(connection, handle, atoms._NET_FRAME_EXTENTS)?;
        let shadows = read_extents(connection, handle, atoms._GTK_FRAME_EXTENTS)?;

        Ok(Margins {
            left: decorations.left - shadows.left,
            right: decorations.right - shadows.right,
            top: decorations.top - shadows.top,
            bottom: decorations.bottom - shadows.bottom,
        })
    }

    /// Moves and/or resizes a window: the position is that of its frame (the top-left corner of
    /// its decorations), and the size that of its client area.
    fn move_resize(
        &self,
        connection: &RustConnection,
        handle: WindowHandle,
        geometry: &ConfigureWindowAux,
    ) -> Result<()> {
        let atoms = self.atoms();

        if !self.is_supported(connection, atoms._NET_MOVERESIZE_WINDOW)? {
            // Window managers interpret this with the window's own gravity, which is almost
            // always north-west. Without a window manager, the window is its own frame.
            connection.configure_window(handle.id, geometry)?.check()?;
            return Ok(());
        }

        // With north-west gravity, the position is that of the frame. Static gravity would give
        // the client area instead, but offset by the border width the client had before being
        // reparented, which the window manager keeps to itself.
        let flags = u32::from(Gravity::NORTH_WEST)
            | u32::from(geometry.x.is_some()) << 8
            | u32::from(geometry.y.is_some()) << 9
            | u32::from(geometry.width.is_some()) << 10
            | u32::from(geometry.height.is_some()) << 11
            | NET_MOVERESIZE_SOURCE_PAGER << 12;

        self.send_client_message(
            handle,
            atoms._NET_MOVERESIZE_WINDOW,
            [
                flags,
                geometry.x.unwrap_or_default().cast_unsigned(),
                geometry.y.unwrap_or_default().cast_unsigned(),
                geometry.width.unwrap_or_default(),
                geometry.height.unwrap_or_default(),
            ],
        )
    }

    /// Returns whether the window manager supports an EWMH hint.
    fn is_supported(&self, connection: &RustConnection, hint: Atom) -> Result<bool> {
        let platform = self.runtime.platform();
        let root = platform.x11_connection().screen().root;
        let reply = connection
            .get_property(
                false,
                root,
                platform.atoms()._NET_SUPPORTED,
                AtomEnum::ATOM,
                0,
                u32::MAX,
            )?
            .reply()?;

        Ok(reply
            .value32()
            .is_some_and(|mut supported| supported.any(|atom| atom == hint)))
    }
}

/// Reads a left, right, top, bottom extents property; zero when the window does not have it.
fn read_extents(connection: &RustConnection, handle: WindowHandle, atom: Atom) -> Result<Margins> {
    let reply = connection
        .get_property(false, handle.id, atom, AtomEnum::CARDINAL, 0, 4)?
        .reply()?;
    let mut extents = reply
        .value32()
        .into_iter()
        .flatten()
        .map(|extent| i32::try_from(extent).unwrap_or(i32::MAX));

    Ok(Margins {
        left: extents.next().unwrap_or_default(),
        right: extents.next().unwrap_or_default(),
        top: extents.next().unwrap_or_default(),
        bottom: extents.next().unwrap_or_default(),
    })
}
