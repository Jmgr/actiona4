use std::time::Duration;

use rquickjs::{Ctx, Function, Result};
use tokio::select;

use crate::core::js::cancelable_promise::{JsCancelablePromise, cancelable_promise};

/// Pauses the execution.
/// @returns CancellablePromise<void>
pub fn sleep<'js>(ctx: Ctx<'js>, ms: f64) -> Result<JsCancelablePromise<'js>> {
    cancelable_promise(ctx, async move |token| {
        select! {
            _ = token.cancelled() => {},
            _ = tokio::time::sleep(Duration::from_secs_f64(ms / 1000.)) => {},
        }

        Ok(())
    })
}

pub(crate) fn register<'js>(ctx: &Ctx<'js>) -> Result<()> {
    ctx.globals()
        .prop("sleep", Function::new(ctx.clone(), sleep))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::runtime::Runtime;

    #[test]
    fn test_sleep() {
        Runtime::test_with_script_engine(async move |script_engine| {
            let start = Instant::now();

            script_engine
                .eval_async::<()>("await sleep(100)")
                .await
                .unwrap();

            let duration = Instant::now() - start;
            assert!(duration.as_millis() >= 100);
        });
    }
}
