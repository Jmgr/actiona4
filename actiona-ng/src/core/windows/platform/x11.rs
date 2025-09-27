use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

use derive_more::Deref;
use libwmctl::{ErrorWrapper, windows};

use crate::core::windows::platform::{Error, Registry, Result, WindowId, WindowsHandler};

#[derive(Deref, Clone)]
struct WindowHandle(libwmctl::Window);

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

#[derive(Debug, Default)]
pub struct X11WindowHandler {
    inner: Registry<WindowHandle>,
}

impl From<ErrorWrapper> for Error {
    fn from(value: ErrorWrapper) -> Self {
        Self::Other(value.into())
    }
}

impl WindowsHandler for X11WindowHandler {
    fn all_windows(&mut self) -> Result<Vec<WindowId>> {
        let windows = windows(false)?;
        let len = windows.len();

        Ok(self
            .inner
            .update(windows.into_iter().map(|window| WindowHandle(window)), len))
    }

    fn is_window_visible(&self, id: WindowId) -> Result<bool> {
        let handle = self.inner.get_handle(id)?;
        let state = handle.mapped()?;

        use libwmctl::MapState::*;
        Ok(match state {
            Unmapped | Unviewable => false,
            Viewable => true,
        })
    }

    fn window_title(&self, id: WindowId) -> Result<String> {
        let handle = self.inner.get_handle(id)?;
        Ok(handle.name()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subsystem() {
        let mut handler = X11WindowHandler::default();
        println!(
            "{:?}",
            handler
                .all_windows()
                .unwrap()
                .into_iter()
                .map(|id| handler.window_title(id))
                .collect::<Result<Vec<String>>>()
                .unwrap()
                .join(", ")
        );

        /*
        let (conn, screen_num, reader) = RustConnection::connect(None).await.unwrap();
        tokio::spawn(reader);

        let root = conn.setup().roots[screen_num].root;
        let net_active = conn
            .intern_atom(false, b"_NET_ACTIVE_WINDOW")
            .await
            .unwrap()
            .reply()
            .await
            .unwrap()
            .atom;
        let win: u8 = AtomEnum::WINDOW.into();
        let reply = conn
            .get_property(false, root, net_active, win, 0, 1)
            .await
            .unwrap()
            .reply()
            .await
            .unwrap();
        if reply.value.len() >= 4 {
            let win = u32::from_ne_bytes(reply.value[..4].try_into().unwrap());
            if let Some(title) = window_title(&conn, win).await.unwrap() {
                println!("Title: {title}");
            }
        }
        */
        let win = libwmctl::active();
        println!("{}", win.name().unwrap());
    }
}
