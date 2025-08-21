use rquickjs::{Ctx, Function, IntoJs, Promise, Result};
use tokio_util::sync::CancellationToken;

use crate::runtime::WithUserData;

// TODO: use cancelable_promise in more places

/// Cancellable future wrapper.
pub(crate) fn cancelable_promise<'js, R, Fut, F>(ctx: Ctx<'js>, future: F) -> Result<Promise<'js>>
where
    F: FnOnce(CancellationToken) -> Fut,
    Fut: Future<Output = Result<R>> + 'js,
    R: IntoJs<'js> + 'js,
{
    //let user_data = JsUserData::from_ctx(&ctx);
    let user_data = ctx.user_data();
    let token = user_data.child_cancellation_token();

    let fut = future(token.clone());

    // TMP
    //let local_ctx = ctx.clone();
    //let fut = async move {
    //    let result = fut.await?;
    //    result.into_js(&local_ctx)
    //};

    let promise = Promise::wrap_future(&ctx, fut)?;

    let cancel_fn = Function::new(ctx.clone(), move || token.cancel())?;

    promise
        .as_object()
        .expect("Promise should be an Object")
        .set("cancel", cancel_fn)?;

    Ok(promise)
}
