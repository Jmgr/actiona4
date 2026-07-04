use std::sync::Arc;

use actiona_core::{runtime::Runtime, scripting::Engine as ScriptEngine};
use tokio_util::sync::CancellationToken;

/// Ambient state available to an action while it runs.
pub struct ExecutionContext {
    pub cancellation_token: CancellationToken,
    pub runtime: Arc<Runtime>,
    pub script_engine: ScriptEngine,
}
