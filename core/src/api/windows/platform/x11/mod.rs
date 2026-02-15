use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    sync::Arc,
};

use derive_more::{Deref, From};
use libwmctl::{Position, Shape, active, windows};
use parking_lot::Mutex;
use x11rb::{
    connection::Connection,
    protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask},
    rust_connection::RustConnection,
};

use crate::{
    api::{
        point::{Point, try_point},
        rect::{Rect, rect},
        size::{Size, try_size},
        windows::platform::{Registry, Result, WindowId, WindowsHandler},
    },
    runtime::Runtime,
    types::{
        si32::si32,
        su32::{Su32, su32},
    },
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
    // tested
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

    // tested
    fn title(&self, id: WindowId) -> Result<String> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        Ok(handle.name()?)
    }

    fn classname(&self, id: WindowId) -> Result<String> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        Ok(handle.class()?)
    }

    // TODO: untested
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
        Ok(Su32::from(handle.pid()?).into())
    }

    // tested
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

        Ok(rect(
            try_point(
                si32(coordinates.dst_x) + extents.left,
                si32(coordinates.dst_y) + extents.top,
            )?,
            try_size(
                su32(geometry.width) - extents.left - extents.right,
                su32(geometry.height) - extents.top - extents.bottom,
            )?,
        ))
    }

    // tested
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

    // TODO: untested
    fn minimize(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.lock().get_handle(id)?.clone();
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();
        let connection = x11_connection.sync_connection();

        connection.unmap_window(handle.id)?;
        connection.flush()?;

        Ok(())
    }

    // TODO: untested
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

    // tested
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

    // tested
    fn size(&self, id: WindowId) -> Result<Size> {
        Ok(self.rect(id)?.size())
    }

    fn is_active(&self, id: WindowId) -> Result<bool> {
        let window = WindowHandle(active());
        let res = window.state()?; // TMP
        println!("{res:?}");
        let handle = self.inner.lock().get_handle(id)?.clone();
        Ok(window == handle) // TODO: return an error if the window doesn't exist anymore
    }

    // tested
    fn active_window(&self) -> Result<WindowId> {
        let window = WindowHandle(active());
        Ok(self.inner.lock().get_or_insert(window))
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

    // TODO: subscribe to a window, would be used in WaitForClosed for a window or something like that
    /*
    async fn subscribe(&self, id: WindowId) -> Result<()> {
        let handle = self.inner.get_handle(id)?.clone();
        let platform = self.runtime.platform();
        let x11_connection = platform.x11_connection();

        let connection = x11_connection.async_connection();
        use x11rb_async::protocol::xproto::ConnectionExt;
        connection
            .change_window_attributes(
                handle.id,
                &ChangeWindowAttributesAux::new().event_mask(EventMask::STRUCTURE_NOTIFY),
            )
            .await?;
        x11rb_async::connection::Connection::flush(connection).await?;

        Ok(())
    }
    */

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

/*
#[cfg(test)]
mod tests {
    use std::{thread::sleep, time::Duration};

    use super::*;

    #[test]
    #[ignore]
    fn test_active_window() {
        Runtime::test(async move |runtime| {
            //let mut handler = X11WindowHandler::new(runtime.clone());
            /*
            let result = handler
                .all()
                .unwrap()
                .into_iter()
                .map(|id| (id, handler.title(id).unwrap()))
                .filter(|(_, title)| title.contains("domains"))
                .collect::<Vec<(WindowId, String)>>();
            let (window, _) = result.first().unwrap();
            handler.set_active(*window).unwrap();
            */
            //let window = handler.active_window().unwrap();
            //let mouse = Mouse::new(runtime).await.unwrap();
            //handler.subscribe(window).await.unwrap();
            loop {
                /*
                let title = handler.title(window).unwrap();
                let rect = handler.rect(window).unwrap();
                println!(
                    "{title} rect:{} bottom:{} mouse:{}",
                    rect,
                    rect.bottom_right(),
                    mouse.position().unwrap(),
                );
                */
                //println!("{:?}", handler.is_active(window));
                sleep(Duration::from_secs(1));
            }
        });
    }
}
*/
