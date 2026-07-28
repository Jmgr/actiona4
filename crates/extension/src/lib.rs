use std::time::Duration;

pub mod extension;
pub mod host;
pub mod ipc;
pub mod protocol;
pub mod protocols;

pub use extension::Extension;
pub use host::Host;
pub use ipc::{extension::Extension as IpcExtension, host::Host as IpcHost};

const RESTART_DELAY: Duration = Duration::from_secs(1);

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use macros::rpc_protocol;
    use parking_lot::Mutex;
    use tokio::{sync::oneshot, time::timeout};

    #[rpc_protocol]
    trait ExtensionCallProtocol {
        #[host_call(no_reply)]
        fn ping(message: String);

        #[extension_call]
        async fn report(message: String);

        /// Handed to the transport and forgotten.
        #[extension_call(no_reply)]
        fn note(message: String);
    }

    struct ExtensionCallHost {
        noted: Mutex<Option<oneshot::Sender<String>>>,
    }

    /// Both calls are handled the same way: only the caller's side of a
    /// no-reply call differs.
    impl ExtensionCallProtocolHost for ExtensionCallHost {
        async fn report(&self, _message: String) -> color_eyre::Result<()> {
            Ok(())
        }

        async fn note(&self, message: String) -> color_eyre::Result<()> {
            let noted = self.noted.lock().take();
            if let Some(noted) = noted {
                let _ = noted.send(message);
            }
            Ok(())
        }
    }

    /// A no-reply call is a plain function: there is no future to await, and
    /// this stops that from silently changing.
    const _: fn(&crate::Extension<ExtensionCallProtocol>, String) -> color_eyre::Result<()> =
        crate::Extension::<ExtensionCallProtocol>::note;

    #[tokio::test]
    async fn no_reply_reaches_the_handler() {
        let (noted, note) = oneshot::channel();
        let handler = Arc::new(ExtensionCallHost {
            noted: Mutex::new(Some(noted)),
        });
        let host = crate::IpcHost::<ExtensionCallProtocol>::with_handler(Duration::from_secs(1), {
            let handler = Arc::clone(&handler);
            move |request| {
                let handler = Arc::clone(&handler);
                async move { handler.handle_request(request).await }
            }
        })
        .await
        .unwrap();
        let extension =
            crate::IpcExtension::<ExtensionCallProtocol>::new(host.key(), Duration::from_secs(1))
                .await
                .unwrap();

        extension
            .notify(ExtensionCallExtensionRequest::Note {
                message: "sample".to_owned(),
            })
            .unwrap();

        assert_eq!(
            timeout(Duration::from_secs(1), note)
                .await
                .unwrap()
                .unwrap(),
            "sample"
        );
    }

    #[tokio::test]
    async fn no_reply_host_call_requires_a_connection() {
        let host = crate::IpcHost::<ExtensionCallProtocol>::new(Duration::from_secs(1))
            .await
            .unwrap();

        assert!(
            host.notify(ExtensionCallHostRequest::Ping {
                message: "ping".to_owned(),
            })
            .is_err()
        );
    }
}
