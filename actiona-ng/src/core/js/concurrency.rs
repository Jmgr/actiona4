use futures::future::select_all;
use rquickjs::{
    Array, Ctx, Function, JsLifetime, Promise, Result, Value, class::Trace, function::Args,
};

use crate::core::js::cancellable_promise::{JsCancellablePromise, cancellable_future};

// TODO: test
#[derive(Debug, JsLifetime, Trace)]
#[rquickjs::class]
pub struct JsConcurrency {}

#[rquickjs::methods]
impl JsConcurrency {
    pub fn new() -> Self {
        Self {}
    }

    #[qjs(static)]
    pub fn race<'js>(ctx: Ctx<'js>, promises: Array<'js>) -> Result<JsCancellablePromise<'js>> {
        let local_ctx = ctx.clone();
        cancellable_future(ctx, async move |token| {
            let promises: Vec<Promise<'js>> = promises
                .iter::<Value<'js>>()
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .filter_map(|v| v.into_promise())
                .collect();

            let mut futures: Vec<_> = promises
                .iter()
                .map(|p| p.clone().into_future::<Value<'js>>())
                .collect();

            // Add the cancellation token to the futures, so this "race" can be stopped.
            let cancelled_promise =
                Promise::wrap_future(&local_ctx, async move { token.cancelled().await })?;
            futures.push(cancelled_promise.into_future::<Value<'js>>());

            let (result, idx, _rest) = select_all(futures).await;

            // Cancel all losers.
            for (i, p) in promises.iter().enumerate() {
                if i == idx {
                    continue;
                }
                if let Some(obj) = p.as_object() {
                    if let Ok(cancel) = obj.get::<_, Function<'js>>("cancel") {
                        let _ = cancel.call_arg::<()>(Args::new(local_ctx.clone(), 0));
                    }
                }
            }

            result
        })
    }
}

impl JsConcurrency {
    pub fn register<'js>(ctx: &Ctx<'js>) -> Result<()> {
        ctx.globals().prop("Concurrency", JsConcurrency::new())
    }
}
