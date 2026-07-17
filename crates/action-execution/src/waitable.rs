use std::{future::Future, pin::Pin};

use action_definition::actions::{ActionInstance, WithCommon, WithDefinition};
use futures_util::{StreamExt, stream::FuturesUnordered};
use tokio_util::sync::CancellationToken;

use crate::{ExecutionContext, RunError, RunErrorKind};

pub type WaitFuture = Pin<Box<dyn Future<Output = Result<(), RunError>> + Send + 'static>>;

/// An action input whose parameters have been resolved and which can be
/// started later with a caller-provided cancellation token.
pub struct PreparedWait {
    start: Box<dyn FnOnce(CancellationToken) -> WaitFuture + Send>,
}

impl PreparedWait {
    pub fn new<F, Fut>(start: F) -> Self
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), RunError>> + Send + 'static,
    {
        Self {
            start: Box::new(move |token| Box::pin(start(token))),
        }
    }

    #[must_use]
    pub fn start(self, token: CancellationToken) -> WaitFuture {
        (self.start)(token)
    }
}

/// Prepares an action for use as an `And` or `Or` input.
///
/// Preparation may resolve scripts, but the returned wait does not borrow the
/// execution context and can therefore run concurrently with other waits.
#[allow(async_fn_in_trait)]
pub trait Waitable {
    async fn prepare(&self, context: &ExecutionContext) -> Result<PreparedWait, RunError>;
}

impl<T: Waitable> Waitable for WithCommon<T> {
    async fn prepare(&self, context: &ExecutionContext) -> Result<PreparedWait, RunError> {
        // Input action timing is owned by the enclosing And/Or action.
        self.action.prepare(context).await
    }
}

impl Waitable for ActionInstance {
    async fn prepare(&self, context: &ExecutionContext) -> Result<PreparedWait, RunError> {
        if !self.definition().is_waitable {
            return Err(RunError::new(RunErrorKind::NonWaitableInput {
                action: self.definition().id,
            }));
        }

        match self {
            Self::Wait(action) => action.prepare(context).await,
            Self::WaitUntil(action) => action.prepare(context).await,
            Self::WaitWhile(action) => action.prepare(context).await,
            Self::WaitForClipboardChanged(action) => action.prepare(context).await,
            Self::WaitForMovement(action) => action.prepare(context).await,
            Self::WaitForScroll(action) => action.prepare(context).await,
            Self::WaitForButton(action) => action.prepare(context).await,
            _ => Err(RunError::new(RunErrorKind::NonWaitableInput {
                action: self.definition().id,
            })),
        }
    }
}

pub async fn run_prepared_wait(
    waitable: &impl Waitable,
    context: &ExecutionContext,
) -> Result<(), RunError> {
    waitable
        .prepare(context)
        .await?
        .start(context.cancellation_token.clone())
        .await
}

pub async fn prepare_inputs(
    inputs: &[ActionInstance],
    context: &ExecutionContext,
    action: &'static str,
) -> Result<Vec<PreparedWait>, RunError> {
    if inputs.is_empty() {
        return Err(RunError::new(RunErrorKind::EmptyWaitInputs { action }));
    }

    let mut prepared = Vec::with_capacity(inputs.len());
    for input in inputs {
        prepared.push(input.prepare(context).await?);
    }
    Ok(prepared)
}

pub async fn join_waits(
    prepared: Vec<PreparedWait>,
    parent_token: &CancellationToken,
) -> Result<(), RunError> {
    let conditions_token = parent_token.child_token();
    let mut waits = FuturesUnordered::new();
    for wait in prepared {
        waits.push(wait.start(conditions_token.child_token()));
    }

    while let Some(result) = waits.next().await {
        if let Err(error) = result {
            conditions_token.cancel();
            return Err(error);
        }
    }

    Ok(())
}

pub async fn race_waits(
    prepared: Vec<PreparedWait>,
    parent_token: &CancellationToken,
) -> Result<usize, RunError> {
    let conditions_token = parent_token.child_token();
    let mut waits = FuturesUnordered::new();
    for (index, wait) in prepared.into_iter().enumerate() {
        let token = conditions_token.child_token();
        waits.push(async move { (index, wait.start(token).await) });
    }

    let mut first_error = None;
    while let Some((index, result)) = waits.next().await {
        match result {
            Ok(()) => {
                conditions_token.cancel();
                return Ok(index);
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }

    Err(first_error.expect("non-empty wait set should produce a result"))
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use tokio::{sync::Notify, time::sleep};
    use tokio_util::sync::CancellationToken;

    use super::{PreparedWait, join_waits, race_waits};
    use crate::{RunError, RunErrorKind};

    fn succeeds_after(duration: Duration) -> PreparedWait {
        PreparedWait::new(move |token| async move {
            tokio::select! {
                () = token.cancelled() => Err(RunError::new(RunErrorKind::Canceled)),
                () = sleep(duration) => Ok(()),
            }
        })
    }

    fn fails() -> PreparedWait {
        PreparedWait::new(|_| async { Err(RunError::new(RunErrorKind::Canceled)) })
    }

    #[tokio::test]
    async fn starts_with_the_token_provided_by_the_caller() {
        let started = Arc::new(Notify::new());
        let observed = started.clone();
        let token = CancellationToken::new();
        token.cancel();

        let wait = PreparedWait::new(move |token| async move {
            assert!(token.is_cancelled());
            observed.notify_one();
            Ok(())
        });

        wait.start(token).await.unwrap();
        started.notified().await;
    }

    #[tokio::test]
    async fn join_waits_until_every_input_completes() {
        let token = CancellationToken::new();
        join_waits(
            vec![
                succeeds_after(Duration::from_millis(1)),
                succeeds_after(Duration::from_millis(5)),
            ],
            &token,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn join_waits_propagates_an_input_error() {
        let token = CancellationToken::new();
        let error = join_waits(
            vec![succeeds_after(Duration::from_secs(1)), fails()],
            &token,
        )
        .await
        .unwrap_err();

        assert!(matches!(error.kind, RunErrorKind::Canceled));
    }

    #[tokio::test]
    async fn race_waits_uses_the_first_success() {
        let token = CancellationToken::new();
        let winner = race_waits(
            vec![
                succeeds_after(Duration::from_millis(5)),
                succeeds_after(Duration::from_millis(1)),
            ],
            &token,
        )
        .await
        .unwrap();

        assert_eq!(winner, 1);
    }

    #[tokio::test]
    async fn race_waits_ignores_errors_when_another_input_succeeds() {
        let token = CancellationToken::new();
        let winner = race_waits(
            vec![fails(), succeeds_after(Duration::from_millis(1))],
            &token,
        )
        .await
        .unwrap();

        assert_eq!(winner, 1);
    }
}
