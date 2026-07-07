use std::sync::Arc;

use action_definition::tree::{BranchKind, NodeId};
use actiona_core::{
    runtime::Runtime,
    scripting::{Engine as ScriptEngine, ScriptError},
};
use rquickjs::{IntoJs, Object};
use tokio_util::sync::CancellationToken;

use crate::ExecutionState;

/// State and ambient services available while an action tree runs.
pub struct ExecutionContext {
    pub cancellation_token: CancellationToken,
    pub runtime: Arc<Runtime>,
    pub script_engine: ScriptEngine,
    pub reason: RunReason,
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
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum RunReason {
    #[default]
    Normal,
    BranchCompleted(BranchKind),
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
            reason: RunReason::Normal,
            current_node: None,
            state: ExecutionState::default(),
        }
    }

    pub(crate) fn prepare_action(
        &mut self,
        node_id: NodeId,
        reason: RunReason,
        tree: &action_definition::tree::ActionTree,
    ) -> Result<(), crate::RunError> {
        self.state.reconcile_to(tree, node_id)?;
        self.current_node = Some(node_id);
        self.reason = reason;
        Ok(())
    }

    pub fn node_id(&self) -> NodeId {
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
                ..Default::default()
            },
        )
        .await
        .expect("runtime should run set_variable test");

        assert_eq!(result.lock().take(), Some((42, true)));
    }
}
