//! File dialogs shown by the xdg-desktop-portal file chooser, over D-Bus.
//!
//! A portal call returns a request object, and the result arrives later as that object's
//! `Response` signal. Calling the request's `Close` method closes the dialog, which is done when
//! the dialog's future is dropped before the response arrives.

use std::{
    collections::HashMap,
    ffi::OsString,
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

use futures_util::StreamExt;
use tokio::runtime::Handle;
use tracing::debug;
use zbus::{
    Connection, MatchRule, MessageStream, Proxy,
    message::Type,
    proxy::{Builder, CacheProperties},
    zvariant::{ObjectPath, OwnedObjectPath, OwnedValue, Value},
};

use crate::{Error, FileDialogOptions, Result, options::OpenMode};

const DESTINATION: &str = "org.freedesktop.portal.Desktop";
const DESKTOP_PATH: &str = "/org/freedesktop/portal/desktop";
const FILE_CHOOSER_INTERFACE: &str = "org.freedesktop.portal.FileChooser";
const REQUEST_INTERFACE: &str = "org.freedesktop.portal.Request";
/// Where request objects live.
const REQUESTS_PATH: &str = "/org/freedesktop/portal/desktop/request";

/// Version of the file chooser interface that added folder selection.
const DIRECTORY_VERSION: u32 = 3;

/// The portal's response code for a request the user cancelled.
const RESPONSE_CANCELLED: u32 = 1;

/// Connection to the file chooser portal.
#[derive(Clone, Debug)]
pub struct Portal {
    connection: Connection,
    version: u32,
}

impl Portal {
    /// Connects to the session bus and reads the file chooser's version. Returns `None` if there
    /// is no session bus or no file chooser portal.
    pub async fn connect() -> Option<Self> {
        match Self::try_connect().await {
            Ok(portal) => Some(portal),
            Err(error) => {
                debug!("xdg-desktop-portal file chooser unavailable: {error}");
                None
            }
        }
    }

    async fn try_connect() -> zbus::Result<Self> {
        let connection = Connection::session().await?;
        let file_chooser = proxy(&connection, DESKTOP_PATH, FILE_CHOOSER_INTERFACE).await?;
        let version = file_chooser.get_property("version").await?;

        Ok(Self {
            connection,
            version,
        })
    }

    /// Whether this portal can show an open dialog in this mode.
    pub const fn supports(&self, mode: OpenMode) -> bool {
        !mode.directory || self.version >= DIRECTORY_VERSION
    }

    pub async fn open(
        &self,
        options: &FileDialogOptions,
        mode: OpenMode,
    ) -> Result<Option<Vec<PathBuf>>> {
        let mut portal_options = common_options(options);
        portal_options.insert("multiple", mode.multiple.into());
        portal_options.insert("directory", mode.directory.into());

        self.request("OpenFile", &options.title, portal_options)
            .await
    }

    pub async fn save(&self, options: &FileDialogOptions) -> Result<Option<PathBuf>> {
        let mut portal_options = common_options(options);
        if let Some(name) = &options.file_name {
            portal_options.insert("current_name", name.as_str().into());
        }

        let paths = self
            .request("SaveFile", &options.title, portal_options)
            .await?;

        Ok(paths.and_then(|paths| paths.into_iter().next()))
    }

    /// Calls a file chooser method and waits for its response. Returns `None` if the user
    /// cancelled.
    async fn request(
        &self,
        method: &str,
        title: &str,
        mut options: HashMap<&str, Value<'_>>,
    ) -> Result<Option<Vec<PathBuf>>> {
        let token = next_token();
        options.insert("handle_token", token.as_str().into());

        // Listen for every request's response before making the call, so that the response
        // cannot be missed, whatever path the request gets: the token chooses it, except with
        // portals older than 0.9, which ignore it and choose their own.
        let rule = MatchRule::builder()
            .msg_type(Type::Signal)
            .interface(REQUEST_INTERFACE)?
            .member("Response")?
            .path_namespace(REQUESTS_PATH)?
            .build();
        let mut responses = MessageStream::for_match_rule(rule, &self.connection, None).await?;
        let mut close_guard = CloseOnDrop {
            connection: self.connection.clone(),
            request: Some(
                request_path(&self.connection, &token)?
                    .try_into()
                    .map_err(zbus::Error::from)?,
            ),
        };

        let file_chooser = proxy(&self.connection, DESKTOP_PATH, FILE_CHOOSER_INTERFACE).await?;
        let handle: OwnedObjectPath = file_chooser.call(method, &("", title, options)).await?;
        close_guard.request = Some(handle.clone());

        let message = loop {
            let message = responses.next().await.ok_or_else(|| {
                Error::Backend("the portal ended the request without a response".to_owned())
            })??;
            if message
                .header()
                .path()
                .is_some_and(|path| path.as_str() == handle.as_str())
            {
                break message;
            }
        };
        close_guard.request = None;

        let (response, mut results): (u32, HashMap<String, OwnedValue>) =
            message.body().deserialize()?;

        match response {
            0 => {
                let uris: Vec<String> = match results.remove("uris") {
                    Some(uris) => uris.try_into().map_err(zbus::Error::from)?,
                    None => Vec::new(),
                };
                uris.iter()
                    .map(|uri| {
                        uri_to_path(uri).ok_or_else(|| {
                            Error::Backend(format!("the portal returned a non-local file: {uri}"))
                        })
                    })
                    .collect::<Result<Vec<_>>>()
                    .map(Some)
            }
            RESPONSE_CANCELLED => Ok(None),
            _ => Err(Error::Backend(format!(
                "the portal ended the request with response {response}"
            ))),
        }
    }
}

/// Calls `Close` on `request` when dropped, unless it is `None`.
struct CloseOnDrop {
    connection: Connection,
    request: Option<OwnedObjectPath>,
}

impl Drop for CloseOnDrop {
    fn drop(&mut self) {
        let Some(request) = self.request.take() else {
            return;
        };
        let Ok(runtime) = Handle::try_current() else {
            return;
        };

        let connection = self.connection.clone();
        runtime.spawn(async move {
            let closed = async {
                proxy(&connection, request, REQUEST_INTERFACE)
                    .await?
                    .call_method("Close", &())
                    .await
            };
            if let Err(error) = closed.await {
                debug!("closing the portal request failed: {error}");
            }
        });
    }
}

async fn proxy<'p>(
    connection: &Connection,
    path: impl TryInto<ObjectPath<'p>, Error = impl Into<zbus::Error>>,
    interface: &'static str,
) -> zbus::Result<Proxy<'static>> {
    let path = path.try_into().map_err(Into::into)?.into_owned();

    Builder::new(connection)
        .destination(DESTINATION)?
        .path(path)?
        .interface(interface)?
        .cache_properties(CacheProperties::No)
        .build()
        .await
}

/// Options shared by `OpenFile` and `SaveFile`.
fn common_options(options: &FileDialogOptions) -> HashMap<&'static str, Value<'static>> {
    let mut portal_options = HashMap::new();
    portal_options.insert("modal", true.into());

    if !options.filters.is_empty() {
        let filters: Vec<(String, Vec<(u32, String)>)> = options
            .filters
            .iter()
            .map(|filter| {
                let globs = filter
                    .extensions
                    .iter()
                    .map(|extension| (0, format!("*.{extension}")))
                    .collect();
                (filter.name.clone(), globs)
            })
            .collect();
        portal_options.insert("filters", filters.into());
    }

    if let Some(directory) = &options.directory {
        portal_options.insert("current_folder", path_bytes(directory).into());
    }

    portal_options
}

/// A path as a null-terminated byte array, the form the portal expects.
fn path_bytes(path: &Path) -> Vec<u8> {
    let mut bytes = path.as_os_str().as_bytes().to_vec();
    bytes.push(0);
    bytes
}

/// A token unique to this process, used to name the request object.
fn next_token() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("actiona_{}_{count}", process::id())
}

/// The request object's path: `/org/freedesktop/portal/desktop/request/SENDER/TOKEN`, where
/// SENDER is the connection's unique name without its leading `:` and with `.` replaced by `_`.
fn request_path(connection: &Connection, token: &str) -> Result<String> {
    let unique_name = connection
        .unique_name()
        .ok_or_else(|| Error::Backend("the D-Bus connection has no unique name".to_owned()))?;
    let sender = unique_name.trim_start_matches(':').replace('.', "_");

    Ok(format!("{DESKTOP_PATH}/request/{sender}/{token}"))
}

/// Converts a `file://` URI to a path. Returns `None` for other schemes and remote hosts.
fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let path = rest.strip_prefix("localhost").unwrap_or(rest);
    if !path.starts_with('/') {
        return None;
    }

    let mut bytes = Vec::with_capacity(path.len());
    let mut iter = path.bytes();
    while let Some(byte) = iter.next() {
        if byte == b'%' {
            let high = hex_value(iter.next()?)?;
            let low = hex_value(iter.next()?)?;
            bytes.push(high << 4 | low);
        } else {
            bytes.push(byte);
        }
    }

    Some(OsString::from_vec(bytes).into())
}

const fn hex_value(digit: u8) -> Option<u8> {
    match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        b'A'..=b'F' => Some(digit - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf, time::Duration};

    use tokio::{sync::oneshot, time::timeout};
    use zbus::{
        Connection, connection, fdo,
        message::Header,
        zvariant::{OwnedObjectPath, OwnedValue, Value},
    };

    use super::{DESKTOP_PATH, DESTINATION, Portal, REQUESTS_PATH, path_bytes, uri_to_path};
    use crate::{FileDialogOptions, options::OpenMode};

    /// What the fake file chooser does with a request.
    enum Behaviour {
        /// Responds before the call returns, on the path the handle token asks for or, like
        /// portals older than 0.9, on a path of its own.
        Respond { own_path: bool },
        /// Never responds, and reports when the request is closed.
        Wait(Option<oneshot::Sender<()>>),
    }

    struct FakeFileChooser(Behaviour);

    #[zbus::interface(name = "org.freedesktop.portal.FileChooser")]
    impl FakeFileChooser {
        #[zbus(property, name = "version")]
        #[allow(clippy::unused_self)]
        const fn version(&self) -> u32 {
            3
        }

        async fn open_file(
            &mut self,
            #[zbus(connection)] connection: &Connection,
            #[zbus(header)] header: Header<'_>,
            parent: &str,
            title: &str,
            options: HashMap<String, OwnedValue>,
        ) -> fdo::Result<OwnedObjectPath> {
            // The method's signature needs them, but the fake has no use for them.
            _ = (parent, title);
            let sender = header
                .sender()
                .map(|sender| sender.trim_start_matches(':').replace('.', "_"))
                .unwrap_or_default();
            let token = options
                .get("handle_token")
                .and_then(|token| token.downcast_ref::<String>().ok())
                .unwrap_or_default();

            match &mut self.0 {
                Behaviour::Respond { own_path } => {
                    let path = if *own_path {
                        format!("{REQUESTS_PATH}/{sender}/legacy")
                    } else {
                        format!("{REQUESTS_PATH}/{sender}/{token}")
                    };
                    let results = HashMap::from([(
                        "uris",
                        Value::from(vec!["file:///tmp/picked%20file".to_owned()]),
                    )]);
                    connection
                        .emit_signal(
                            None::<()>,
                            path.as_str(),
                            "org.freedesktop.portal.Request",
                            "Response",
                            &(0_u32, results),
                        )
                        .await?;
                    Ok(OwnedObjectPath::try_from(path).map_err(zbus::Error::from)?)
                }
                Behaviour::Wait(closed) => {
                    let path = format!("{REQUESTS_PATH}/{sender}/{token}");
                    connection
                        .object_server()
                        .at(path.as_str(), FakeRequest(closed.take()))
                        .await?;
                    Ok(OwnedObjectPath::try_from(path).map_err(zbus::Error::from)?)
                }
            }
        }
    }

    struct FakeRequest(Option<oneshot::Sender<()>>);

    #[zbus::interface(name = "org.freedesktop.portal.Request")]
    impl FakeRequest {
        fn close(&mut self) {
            if let Some(closed) = self.0.take() {
                _ = closed.send(());
            }
        }
    }

    async fn serve(behaviour: Behaviour) -> Connection {
        connection::Builder::session()
            .unwrap()
            .name(DESTINATION)
            .unwrap()
            .serve_at(DESKTOP_PATH, FakeFileChooser(behaviour))
            .unwrap()
            .build()
            .await
            .unwrap()
    }

    const PICK_FILE: OpenMode = OpenMode {
        multiple: false,
        directory: false,
    };

    // These tests own the portal's name, so they need a session bus of their own:
    // `dbus-run-session -- cargo test -p dialogs --lib portal -- --ignored`.

    #[tokio::test]
    #[ignore = "needs a private session bus"]
    async fn receives_a_response_sent_before_the_call_returns() {
        for own_path in [false, true] {
            let _service = serve(Behaviour::Respond { own_path }).await;
            let portal = Portal::connect().await.unwrap();

            let paths = timeout(
                Duration::from_secs(5),
                portal.open(&FileDialogOptions::default(), PICK_FILE),
            )
            .await
            .expect("the response was missed")
            .unwrap();
            assert_eq!(paths, Some(vec![PathBuf::from("/tmp/picked file")]));
        }
    }

    #[tokio::test]
    #[ignore = "needs a private session bus"]
    async fn closes_the_request_when_dropped() {
        let (closed, was_closed) = oneshot::channel();
        let _service = serve(Behaviour::Wait(Some(closed))).await;
        let portal = Portal::connect().await.unwrap();

        let result = timeout(
            Duration::from_millis(500),
            portal.open(&FileDialogOptions::default(), PICK_FILE),
        )
        .await;
        assert!(result.is_err(), "the fake portal never responds");

        timeout(Duration::from_secs(5), was_closed)
            .await
            .expect("the request was not closed")
            .unwrap();
    }

    #[test]
    fn converts_file_uris() {
        assert_eq!(
            uri_to_path("file:///home/user/a%20b%C3%A9.txt"),
            Some(PathBuf::from("/home/user/a bé.txt"))
        );
        assert_eq!(
            uri_to_path("file://localhost/tmp/x"),
            Some(PathBuf::from("/tmp/x"))
        );
        assert_eq!(uri_to_path("file://server/share/x"), None);
        assert_eq!(uri_to_path("https://example.com/x"), None);
        assert_eq!(uri_to_path("file:///bad%2"), None);
        assert_eq!(uri_to_path("file:///bad%zz"), None);
    }

    #[test]
    fn path_bytes_are_null_terminated() {
        assert_eq!(path_bytes(&PathBuf::from("/tmp")), b"/tmp\0");
    }
}
