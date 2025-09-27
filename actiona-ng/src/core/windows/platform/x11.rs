use std::sync::Arc;

use eyre::Result;
use x11rb_async::{
    protocol::xproto::{Atom, AtomEnum, ConnectionExt, GetPropertyReply, Window},
    rust_connection::RustConnection,
};

use crate::{core::displays::platform::x11, platform::x11::X11Connection};

async fn get_utf8_prop(
    connection: &RustConnection,
    window: Window,
    prop: u32,
    utf8_atom: u32,
) -> Result<Option<String>> {
    let reply = connection
        .get_property(false, window, prop, utf8_atom, 0, u32::MAX)
        .await?
        .reply()
        .await?;
    Ok(if reply.format == 8 && reply.type_ == utf8_atom {
        Some(String::from_utf8(reply.value)?)
    } else {
        None
    })
}

async fn get_any_prop(
    connection: &RustConnection,
    window: Window,
    prop: u32,
) -> Result<Option<String>> {
    // Ask for any type; if STRING, treat as Latin-1
    let reply: GetPropertyReply = connection
        .get_property(false, window, prop, Atom::default(), 0, u32::MAX)
        .await?
        .reply()
        .await?;

    // STRING is Latin-1 per ICCCM; convert lossy to UTF-8
    let string_atom: u8 = AtomEnum::STRING.into();
    Ok(if reply.format == 8 && reply.type_ == string_atom as u32 {
        Some(reply.value.iter().map(|&b| b as char).collect())
    } else {
        // COMPOUND_TEXT decoding is possible but non-trivial; skip here
        None
    })
}

pub async fn window_title(connection: &RustConnection, window: Window) -> Result<Option<String>> {
    let utf8_atom = connection
        .intern_atom(false, b"UTF8_STRING")
        .await?
        .reply()
        .await?
        .atom;
    let net_wm_name = connection
        .intern_atom(false, b"_NET_WM_NAME")
        .await?
        .reply()
        .await?
        .atom;

    if let Some(title) = get_utf8_prop(connection, window, net_wm_name, utf8_atom).await? {
        if !title.is_empty() {
            return Ok(Some(title));
        }
    }

    let result = get_any_prop(connection, window, AtomEnum::WM_NAME.into())
        .await?
        .map(|result| result.trim_end_matches('\0') .to_string()) // some WMs add NUL
       ;

    Ok(result)
}

#[cfg(test)]
mod tests {
    use x11rb_async::connection::Connection;

    use super::*;

    #[tokio::test]
    async fn test_subsystem() {
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
