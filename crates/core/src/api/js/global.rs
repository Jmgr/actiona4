use rquickjs::{Ctx, Function, Promise, Result, Value, prelude::Rest};
use tokio::select;
use tracing::instrument;

use crate::{
    IntoJsResult,
    api::{
        console::js::JsConsole,
        js::{duration::JsDuration, task::task},
    },
    error::CommonError,
    runtime::WithUserData,
};

/// Pauses the execution for the given duration.
///
/// ```ts
/// // Wait 500 milliseconds
/// await sleep(500);
///
/// // Wait 1 second
/// await sleep("1s");
///
/// // Wait 1 hour
/// await sleep("1h");
/// ```
/// Numeric values are interpreted as milliseconds.
/// @returns Task<void>
pub fn sleep<'js>(ctx: Ctx<'js>, duration: JsDuration) -> Result<Promise<'js>> {
    task(ctx, async move |ctx, token| {
        select! {
            _ = token.cancelled() => { Err(CommonError::Cancelled).into_js_result(&ctx) },
            _ = tokio::time::sleep(duration.0) => { Ok(()) },
        }
    })
}

/// Stops the script execution immediately.
///
/// ```ts
/// if (errorCondition) {
///   exit();
/// }
/// ```
pub fn exit<'js>(ctx: Ctx<'js>) {
    let token = ctx.user_data().cancellation_token();

    token.cancel();
}

/// Prints values without a trailing newline.
///
/// Alias for `console.print(...)`.
/// @rest
pub fn print<'js>(ctx: Ctx<'js>, data: Rest<Value<'js>>) {
    JsConsole::do_print(&ctx, data);
}

/// Prints values followed by a newline.
///
/// Alias for `console.println(...)`.
/// @rest
pub fn println<'js>(ctx: Ctx<'js>, data: Rest<Value<'js>>) {
    JsConsole::do_println(&ctx, data);
}

/// Pretty-prints values using an inspect-style multiline format.
///
/// Alias for `console.inspect(...)`.
/// @rest
pub fn inspect<'js>(ctx: Ctx<'js>, data: Rest<Value<'js>>) {
    JsConsole::do_inspect(&ctx, data);
}

#[instrument(skip_all)]
pub(crate) fn register<'js>(ctx: &Ctx<'js>) -> Result<()> {
    let target = super::classes::registration_target(ctx);
    target.prop("sleep", Function::new(ctx.clone(), sleep))?;
    target.prop("exit", Function::new(ctx.clone(), exit))?;
    target.prop("print", Function::new(ctx.clone(), print))?;
    target.prop("println", Function::new(ctx.clone(), println))?;
    target.prop("inspect", Function::new(ctx.clone(), inspect))?;
    Ok(())
}
