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
    use macros::rpc_protocol;

    #[rpc_protocol]
    trait ExtensionCallProtocol {
        #[extension_call]
        async fn notify(message: String);
    }

    #[allow(dead_code)]
    struct ExtensionCallHost;

    impl ExtensionCallProtocolHost for ExtensionCallHost {
        async fn notify(&self, _message: String) -> color_eyre::Result<()> {
            Ok(())
        }
    }
}
