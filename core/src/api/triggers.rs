use std::sync::Arc;

use rquickjs::{AsyncContext, Ctx, Value, async_with};
use tracing::warn;

use crate::{
    api::macros::{MacroData, PlayConfig, js::JsMacro, player::MacroPlayer},
    runtime::WithUserData,
    scripting::callbacks::FunctionKey,
};

/// The action to perform when a trigger fires.
#[derive(Clone)]
pub enum TriggerAction {
    /// Play a macro.
    Macro(Arc<MacroData>),
    /// Invoke a JS callback; if it returns a `Macro`, play it.
    Callback(AsyncContext, FunctionKey),
}

impl TriggerAction {
    /// Fire the action, calling the callback with no arguments.
    ///
    /// Uses `call_sync_returning` so the `async_with!` closure does not yield,
    /// preserving the rquickjs scheduler's queue waker.
    pub async fn fire(self, macro_player: &Arc<MacroPlayer>, label: &'static str) {
        match self {
            Self::Macro(data) => macro_player.play_detached(data, PlayConfig::default()),
            Self::Callback(context, function_key) => {
                let macro_player_clone = macro_player.clone();
                async_with!(context => |ctx| {
                    fire_callback(&ctx, function_key, &macro_player_clone, vec![], label);
                })
                .await;
            }
        }
    }
}

/// Execute a callback action inside an `async_with!` block.
///
/// Calls `call_sync_returning` so the closure does not yield. If the callback
/// returns a `Promise`, it is spawned via `ctx.spawn` so the caller's event
/// loop is not blocked. If it returns a `Macro` (sync or resolved from an
/// async callback), the macro is played via `MacroPlayer`.
pub fn fire_callback<'js>(
    ctx: &Ctx<'js>,
    function_key: FunctionKey,
    macro_player: &Arc<MacroPlayer>,
    args: Vec<Value<'js>>,
    label: &'static str,
) {
    let value = ctx
        .user_data()
        .callbacks()
        .call_sync_returning(ctx, function_key, args);

    if let Some(promise) = value.as_promise() {
        let promise = promise.clone();
        let player_for_spawn = macro_player.clone();
        ctx.spawn(async move {
            match promise.into_future::<Value<'_>>().await {
                Ok(resolved) => {
                    if let Ok(r#macro) = resolved.get::<JsMacro>() {
                        player_for_spawn.play_detached(r#macro.data(), PlayConfig::default());
                    }
                }
                Err(error) => {
                    warn!(?function_key, error = %error, "{label} async callback failed");
                }
            }
        });
    } else if let Ok(r#macro) = value.get::<JsMacro>() {
        macro_player.play_detached(r#macro.data(), PlayConfig::default());
    }
}
