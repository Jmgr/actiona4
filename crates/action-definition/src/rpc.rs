//! Typed RPC protocol shared by the host (native) and the UI (wasm).
//!
//! A single `#[rpc] trait Api` declaration generates everything (the tarpc /
//! tonic pattern):
//!
//! * `trait Api` — the **host** implements it with plain `async fn`s.
//! * `ApiClient<T>` — the **UI** calls `client.add_ten(5).await`, fully typed.
//! * `api_serve(api, cmd, json)` — host-side dispatch by command name.
//! * `__rpc_<method>` modules — the by-name argument structs, shared by both
//!   ends so the wire format can never drift.
//!
//! Because the trait and its argument/return types live here in `common`, both
//! Rust ends are checked against the same definition at compile time — the type
//! safety a JS frontend (Tauri) only recovers via a codegen step like
//! `tauri-specta`.

use std::future::Future;

use macros::rpc;
use serde::{Deserialize, Serialize};

use crate::tree::ActionTree;

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

/// The RPC surface between the UI and the host.
///
/// Add a method here and both the typed client and the host dispatcher pick it
/// up automatically; the host just has to implement the new `async fn`.
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
