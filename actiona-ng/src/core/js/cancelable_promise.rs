use rquickjs::{Ctx, Function, IntoJs, Promise, Result, Value};
use tokio_util::sync::CancellationToken;

use crate::runtime::JsUserData;

// TODO: replace more Promise with CancelablePromise when relevant

/// Represents a promise that can be cancelled.
/// @generic
/// @extends Promise<T>
/// @method cancel(): void
#[derive(Clone, Debug)]
pub struct JsCancelablePromise<'js> {
    inner: Value<'js>,
}

impl<'js> JsCancelablePromise<'js> {
    /// @skip
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

        Ok(Self {
            inner: promise.into_value(),
        })
    }
}

impl<'js> IntoJs<'js> for JsCancelablePromise<'js> {
    fn into_js(self, ctx: &Ctx<'js>) -> Result<Value<'js>> {
        self.inner.into_js(&ctx)
    }
}

impl<'js> JsCancelablePromise<'js> {
    /// @skip
    pub fn register(ctx: &Ctx<'_>) -> Result<()> {
        ctx.eval::<(), _>("class CancelablePromise extends Promise {}")
    }
}

/// Cancellable future wrapper.
pub(crate) fn cancelable_promise<'js, R, Fut, F>(
    ctx: Ctx<'js>,
    future: F,
) -> Result<JsCancelablePromise<'js>>
where
    F: FnOnce(CancellationToken) -> Fut,
    Fut: Future<Output = Result<R>> + 'js,
    R: IntoJs<'js> + 'js,
{
    let token = ctx
        .userdata::<JsUserData>()
        .expect("userdata not set")
        .child_cancellation_token();

    let fut = future(token.clone());

    let local_ctx = ctx.clone();
    let fut = async move {
        let result = fut.await?;
        result.into_js(&local_ctx)
    };

    JsCancelablePromise::new(ctx.clone(), Promise::wrap_future(&ctx, fut)?, token)
}
