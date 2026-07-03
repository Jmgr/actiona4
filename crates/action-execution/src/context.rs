use tokio_util::sync::CancellationToken;

/// Ambient state available to an action while it runs.
pub struct ExecutionContext {
    pub cancellation_token: CancellationToken,
}
