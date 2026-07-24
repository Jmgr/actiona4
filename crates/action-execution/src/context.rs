use std::sync::Arc;

use action_definition::tree::{ActionTree, NodeId};
use actiona_core::{
    runtime::Runtime,
    scripting::{Engine as ScriptEngine, ScriptError},
};
use rquickjs::{Array, Error, IntoJs, Object, Value};
use tokio_util::sync::CancellationToken;

use crate::ExecutionState;

/// State and ambient services available while an action tree runs.
pub struct ExecutionContext {
    pub cancellation_token: CancellationToken,
    pub runtime: Arc<Runtime>,
    pub script_engine: ScriptEngine,
    current_node: Option<NodeId>,
    state: ExecutionState,
}

impl ExecutionContext {
    pub async fn set_variable<V>(&self, name: &str, value: V) -> Result<(), ScriptError>
    where
        V: for<'js> IntoJs<'js> + Send,
    {
        self.script_engine
            .with(|ctx| {
                let globals = ctx.globals();
                let vars = if globals.contains_key("vars")? {
                    globals.get::<_, Object>("vars")?
                } else {
                    let vars = Object::new(ctx.clone())?;
                    globals.set("vars", vars.clone())?;
                    vars
                };

                vars.set(name, value)
            })
            .await
    }

    pub(crate) async fn store_array(&self, source: &str) -> Result<u32, ScriptError> {
        let slot = self.array_slot();
        self.script_engine
            .eval_async_with(source, move |ctx, value| {
                let array = value.as_array().ok_or_else(|| {
                    Error::new_from_js_message(
                        "value",
                        "array",
                        "array parameter must resolve to an array",
                    )
                })?;
                let count = u32::try_from(array.len()).map_err(|_| {
                    Error::new_from_js_message("array", "u32", "array has too many items")
                })?;

                ctx.globals().set(slot, array.clone())?;
                Ok(count)
            })
            .await
    }

    pub(crate) async fn set_variable_from_stored_array(
        &self,
        name: &str,
        index: u32,
    ) -> Result<(), ScriptError> {
        let slot = self.array_slot();
        self.script_engine
            .with(|ctx| {
                let globals = ctx.globals();
                let array = globals.get::<_, Array>(&slot)?;
                let value = array.get::<Value>(index as usize)?;
                let vars = if globals.contains_key("vars")? {
                    globals.get::<_, Object>("vars")?
                } else {
                    let vars = Object::new(ctx.clone())?;
                    globals.set("vars", vars.clone())?;
                    vars
                };

                vars.set(name, value)
            })
            .await
    }

    fn array_slot(&self) -> String {
        format!("__actiona_for_each_{:?}", self.node_id())
    }
}

impl ExecutionContext {
    pub fn new(
        cancellation_token: CancellationToken,
        runtime: Arc<Runtime>,
        script_engine: ScriptEngine,
    ) -> Self {
        Self {
            cancellation_token,
            runtime,
            script_engine,
            current_node: None,
            state: ExecutionState::default(),
        }
    }

    pub(crate) fn prepare_action(
        &mut self,
        node_id: NodeId,
        tree: &ActionTree,
    ) -> Result<(), crate::RunError> {
        self.state.reconcile_to(tree, node_id)?;
        self.current_node = Some(node_id);
        Ok(())
    }

    #[must_use]
    pub const fn node_id(&self) -> NodeId {
        self.current_node
            .expect("execution context should have a current node while an action runs")
    }

    pub fn runtime_state_mut<T>(&mut self, create: impl FnOnce() -> T) -> &mut T
    where
        T: Send + Sync + 'static,
    {
        let node_id = self.node_id();
        self.state.runtime_state_mut_or_insert_with(node_id, create)
    }

    #[must_use]
    pub fn runtime_state<T>(&self) -> Option<&T>
    where
        T: 'static,
    {
        self.state.runtime_state(self.node_id())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actiona_core::runtime::{Runtime, RuntimeOptions, RuntimePlatformSetup};
    use parking_lot::Mutex;
    use tokio_util::sync::CancellationToken;

    use super::ExecutionContext;

    #[tokio::test]
    async fn set_variable_writes_to_vars_object() {
        let result = Arc::new(Mutex::new(None));
        let output = result.clone();
        let platform =
            RuntimePlatformSetup::new(false).expect("RuntimePlatformSetup::new should succeed");

        Runtime::run(
            platform,
            move |runtime, script_engine| async move {
                let context =
                    ExecutionContext::new(CancellationToken::new(), runtime, script_engine.clone());

                context.set_variable("answer", 42_i64).await?;

                let value = script_engine.eval_async::<i64>("vars.answer").await?;
                let bare_is_undefined = script_engine
                    .eval_async::<bool>("typeof answer === 'undefined'")
                    .await?;

                *output.lock() = Some((value, bare_is_undefined));

                Ok(())
            },
            RuntimeOptions {
                install_ctrl_c_handler: false,
                show_tray_icon: false,
                discover_extensions: false,
                ..Default::default()
            },
        )
        .await
        .expect("runtime should run set_variable test");

        assert_eq!(result.lock().take(), Some((42, true)));
    }
}
