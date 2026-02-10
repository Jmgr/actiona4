use std::time::Duration;

use rquickjs::{Ctx, Function, Promise, Result, Value, prelude::Rest};
use tokio::select;
use tracing::instrument;

use crate::{
    IntoJsResult,
    api::{console::js::JsConsole, js::task::task},
    error::CommonError,
    runtime::WithUserData,
};

/// Pauses the execution for the given number of milliseconds.
///
/// ```ts
/// // Wait 1 second
/// await sleep(1000);
/// ```
/// @returns Task<void>
pub fn sleep<'js>(ctx: Ctx<'js>, ms: f64) -> Result<Promise<'js>> {
    task(ctx, async move |ctx, token| {
        select! {
            _ = token.cancelled() => { Err(CommonError::Cancelled).into_js_result(&ctx) },
            _ = tokio::time::sleep(Duration::from_secs_f64(ms / 1000.)) => { Ok(()) },
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
/// @category Console
/// @rest
pub fn print<'js>(ctx: Ctx<'js>, data: Rest<Value<'js>>) {
    JsConsole::do_print(&ctx, data);
}

/// Prints values followed by a newline.
///
/// Alias for `console.println(...)`.
/// @category Console
/// @rest
pub fn println<'js>(ctx: Ctx<'js>, data: Rest<Value<'js>>) {
    JsConsole::do_println(&ctx, data);
}

/// Pretty-prints values using an inspect-style multiline format.
///
/// Alias for `console.inspect(...)`.
/// @category Console
/// @rest
pub fn inspect<'js>(ctx: Ctx<'js>, data: Rest<Value<'js>>) {
    JsConsole::do_inspect(&ctx, data);
}

#[instrument(skip_all)]
pub(crate) fn register<'js>(ctx: &Ctx<'js>) -> Result<()> {
    ctx.globals()
        .prop("sleep", Function::new(ctx.clone(), sleep))?;
    ctx.globals()
        .prop("exit", Function::new(ctx.clone(), exit))?;
    ctx.globals()
        .prop("print", Function::new(ctx.clone(), print))?;
    ctx.globals()
        .prop("println", Function::new(ctx.clone(), println))?;
    ctx.globals()
        .prop("inspect", Function::new(ctx.clone(), inspect))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::runtime::Runtime;

    #[test]
    fn test_sleep() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let start = Instant::now();

            script_engine
                .eval_async::<()>("await sleep(100)")
                .await
                .unwrap();

            let duration = Instant::now() - start;
            assert!(duration.as_millis() >= 100);
        });
    }

    #[test]
    fn test_exit() {
        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine.eval::<()>("exit()").await.unwrap();
        });
    }

    #[test]
    fn test_global_console_aliases() {
        Runtime::test_with_script_engine(|script_engine| async move {
            script_engine
                .eval::<()>(
                    r#"
                    print("hello");
                    println({ a: 1 });
                    println([1, 2]);
                    inspect({ nested: [1, { value: "ok" }] });
                    "#,
                )
                .await
                .unwrap();
        });
    }
}
