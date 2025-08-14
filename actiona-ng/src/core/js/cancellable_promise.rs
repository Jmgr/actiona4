use rquickjs::{Ctx, Function, IntoJs, Promise, Result, Value};
use tokio_util::sync::CancellationToken;

use crate::runtime::JsUserData;

/// Represents a promise that can be cancelled.
pub struct JsCancellablePromise<'js>(Value<'js>);

impl<'js> JsCancellablePromise<'js> {
    pub fn new(
        ctx: Ctx<'js>,
        promise: Promise<'js>,
        token: CancellationToken,
    ) -> rquickjs::Result<Self> {
        let cancel_fn = Function::new(ctx.clone(), move || token.cancel())?;

        promise
            .as_object()
            .expect("Promise should be an Object")
            .set("cancel", cancel_fn)?;

        Ok(Self(promise.into_value()))
    }
}

impl<'js> IntoJs<'js> for JsCancellablePromise<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        self.0.into_js(&ctx)
    }
}

/// Cancellable future wrapper.
pub(crate) fn cancellable_future<'js, R, Fut, F>(
    ctx: Ctx<'js>,
    future: F,
) -> Result<JsCancellablePromise<'js>>
where
    F: FnOnce(CancellationToken) -> Fut,
    Fut: Future<Output = Result<R>> + 'js,
    R: IntoJs<'js> + 'js,
{
    let token = ctx
        .userdata::<JsUserData>()
        .expect("userdata not set")
        .child_cancellation_token();

    let r = future(token.clone());

    let local_ctx = ctx.clone();
    let fut = async move {
        let r = r.await?;
        r.into_js(&local_ctx)
    };

    JsCancellablePromise::new(ctx.clone(), Promise::wrap_future(&ctx, fut)?, token)
}
