use std::pin::Pin;

use futures::future::select_all;
use macros::{js_class, js_methods};
use rquickjs::{
    Array, Ctx, Function, JsLifetime, Promise, Result, Value, class::Trace, function::Args,
};
use tracing::instrument;

use crate::api::js::task::task;

/// Utilities for concurrent operations.
///
/// ```ts
/// // Race two promises, resolving with whichever finishes first, cancelling the other.
/// // Note that this is different from `Promises.race`, which doesn't cancel any promise.
/// const result = await concurrency.race([sleep("100ms"), sleep("1s")]);
/// ```
///
/// @singleton
#[derive(Debug, JsLifetime, Trace)]
#[js_class]
pub struct JsConcurrency {}

#[js_methods]
impl JsConcurrency {
    /// @skip
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {}
    }

    /// Races multiple promises, returning the result of the first one to settle.
    /// Losing tasks will be cancelled automatically.
    ///
    /// ```ts
    /// // Resolve with the first successful result.
    /// const result = await concurrency.race([
    ///   sleep("200ms").then(() => "fast"),
    ///   sleep("1s").then(() => "slow"),
    /// ]);
    /// // result === "fast"
    /// ```
    ///
    /// ```ts
    /// // Use race to implement a timeout.
    /// const result = await concurrency.race([
    ///   fetchData(),
    ///   sleep("5s").then(() => { throw new Error("Timeout"); })
    /// ]);
    /// ```
    ///
    /// ```ts
    /// // Rejections also win the race.
    /// // Here the error is thrown quickly and the slower task is cancelled.
    /// try {
    ///   await concurrency.race([
    ///     sleep("50ms").then(() => { throw new Error("Failed quickly"); }),
    ///     sleep("2s"),
    ///   ]);
    /// } catch (e) {
    ///   console.println(e); // Error: Failed quickly
    /// }
    /// ```
    ///
    /// ```ts
    /// // You can cancel the race task itself.
    /// const t = concurrency.race([
    ///   sleep("5s"),
    ///   sleep("8s"),
    /// ]);
    /// t.cancel();
    /// await t; // throws "Error: Cancelled"
    /// ```
    ///
    /// ```ts
    /// // Empty or non-promise-only inputs resolve to undefined.
    /// const a = await concurrency.race([]);
    /// const b = await concurrency.race([1, "text", null]);
    /// // a === undefined, b === undefined
    /// ```
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
    #[instrument(skip_all)]
    pub fn register<'js>(ctx: &Ctx<'js>) -> Result<()> {
        super::classes::registration_target(ctx).prop("concurrency", Self::new())
    }
}
