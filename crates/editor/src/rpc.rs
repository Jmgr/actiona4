use action_definition::tree::ActionTree;
use macros::rpc;
use serde::{Deserialize, Serialize};

/// The transport the generated [`ApiClient`] talks through: JSON value in, JSON
/// value out, async. The wasm UI implements this over the webview bridge; tests
/// implement it as an in-process loopback.
pub trait Transport {
    type Error;

    fn request(
        &self,
        method: &'static str,
        input: serde_json::Value,
    ) -> impl Future<Output = Result<serde_json::Value, Self::Error>>;
}

/// Error surfaced by a generated client call.
#[derive(Debug)]
pub enum RpcError<E> {
    /// Serializing the request payload failed.
    Serialize(serde_json::Error),
    /// Deserializing the response payload failed.
    Deserialize(serde_json::Error),
    /// The transport itself failed (bridge unavailable, etc.).
    Transport(E),
}

#[rpc]
pub trait Api {
    async fn load_rows(&self);
    async fn exit(&self);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", content = "data")]
pub enum HostEvent {
    Tree(ActionTree),
}

/// Where the host pushes [`HostEvent`]s. The host wires this to the webview
/// bridge (`eval` a `CustomEvent`); tests record into a buffer. Kept as a trait
/// so [`Api`] implementations can emit events without depending on the webview.
pub trait EventSink: Send + Sync {
    fn send(&self, event: HostEvent);
}
