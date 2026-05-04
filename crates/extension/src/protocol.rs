use ipc_rpc::UserMessage;
use serde::{Deserialize, Serialize};

pub trait Protocol: std::fmt::Debug + Clone + 'static {
    type HostRequest: UserMessage;
    type HostResponse: UserMessage;
    type ExtensionRequest: UserMessage;
    type ExtensionResponse: UserMessage;
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Never {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub(crate) enum WireMessage<P: Protocol> {
    HostRequest(P::HostRequest),
    HostResponse(Result<P::HostResponse, String>),
    ExtensionRequest(P::ExtensionRequest),
    ExtensionResponse(Result<P::ExtensionResponse, String>),
}
