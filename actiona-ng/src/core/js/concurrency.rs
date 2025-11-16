use std::pin::Pin;

use futures::future::select_all;
use rquickjs::{
    Array, Ctx, Function, JsLifetime, Promise, Result, Value, class::Trace, function::Args,
};

use crate::core::js::task::task;

// TODO: test
#[derive(Debug, JsLifetime, Trace)]
#[rquickjs::class]
pub struct JsConcurrency {}

#[rquickjs::methods]
impl JsConcurrency {
    /// @skip
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {}
    }

    /// @generic
    /// @param promises: Iterable<T|PromiseLike<T>>
    /// @returns Task<Awaited<T>>
    pub fn race<'js>(ctx: Ctx<'js>, promises: Array<'js>) -> Result<Promise<'js>> {
        task(ctx, async move |ctx, token| {
            let promises: Vec<Promise<'js>> = promises
                .iter::<Value<'js>>()
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .filter_map(|v| v.into_promise())
                .collect();

            if promises.is_empty() {
                return Ok(Value::new_undefined(ctx.clone()));
            }

            // Add the cancellation token to the futures, so this "race" can be stopped.
            let mut futures: Vec<Pin<Box<dyn Future<Output = Result<Value<'js>>> + 'js>>> =
                promises
                    .iter()
                    .map(
                        |p| -> Pin<Box<dyn Future<Output = Result<Value<'js>>> + 'js>> {
                            Box::pin(p.clone().into_future::<Value<'js>>())
                        },
                    )
                    .collect();

            // Add a *pure Rust* future for cancellation (no nested wrap_future!)
            let cancel_ctx = ctx.clone();
            let cancel_fut = async move {
                token.cancelled().await;
                Ok(Value::new_undefined(cancel_ctx))
            };
            futures.push(Box::pin(cancel_fut));

            let (result, idx, _rest) = select_all(futures).await;

            // Cancel all losers.
            for (i, p) in promises.iter().enumerate() {
                if i == idx {
                    continue;
                }
                if let Some(obj) = p.as_object()
                    && let Ok(cancel) = obj.get::<_, Function<'js>>("cancel")
                {
                    _ = cancel.call_arg::<()>(Args::new(ctx.clone(), 0));
                }
            }

            result
        })
    }
}

impl JsConcurrency {
    /// @skip
    pub fn register<'js>(ctx: &Ctx<'js>) -> Result<()> {
        ctx.globals().prop("Concurrency", Self::new())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::runtime::Runtime;

    #[test]
    fn test_race() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let start = Instant::now();

            script_engine
                .eval_async::<()>("await Concurrency.race([sleep(100), sleep(1000)])")
                .await
                .unwrap();

            let duration = Instant::now() - start;
            assert!(duration.as_millis() >= 100 && duration.as_millis() < 1000);
        });
    }

    #[test]
    fn test_race_of_race() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let start = Instant::now();

            script_engine
                .eval_async::<()>(
                    "await Concurrency.race([Concurrency.race([sleep(100)]), sleep(1000)])",
                )
                .await
                .unwrap();

            let duration = Instant::now() - start;
            assert!(duration.as_millis() >= 100 && duration.as_millis() < 1000);
        });
    }
}
