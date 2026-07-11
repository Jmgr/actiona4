use action_definition::{
    actions::ForEach, parameters::variable::Variable, post_run::PostRun, tree::BranchKind,
};

use crate::{ExecutionContext, ResolveParam, ResolveParamError, RunError, Runnable};

#[derive(Debug)]
struct RuntimeState {
    count: u32,
    index: u32,
    item_variable: Variable,
}

async fn initialize_state(
    action: &ForEach,
    context: &mut ExecutionContext,
) -> Result<(), RunError> {
    if context.runtime_state::<RuntimeState>().is_some() {
        return Ok(());
    }

    let array = action.array.resolve(context).await?;
    let item_variable = action.item_variable.resolve(context).await?;
    let count = context
        .store_array(array.inner())
        .await
        .map_err(|source| ResolveParamError::new(action.array.name(), source))?;

    context.runtime_state_mut(|| RuntimeState {
        count,
        index: 0,
        item_variable,
    });
    Ok(())
}

impl Runnable for ForEach {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        initialize_state(self, context).await?;

        let count = context
            .runtime_state::<RuntimeState>()
            .expect("ForEach runtime state should be initialized")
            .count;

        Ok(if count == 0 {
            PostRun::default()
        } else {
            PostRun::Branch(BranchKind::Body)
        })
    }

    async fn on_body_enter(&self, context: &mut ExecutionContext) -> Result<(), RunError> {
        initialize_state(self, context).await?;

        let (item_variable, index) = {
            let state = context
                .runtime_state::<RuntimeState>()
                .expect("ForEach runtime state should be initialized");
            (state.item_variable.clone(), state.index)
        };

        context
            .set_variable_from_stored_array(item_variable.inner(), index)
            .await?;
        Ok(())
    }

    async fn on_body_completed(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        initialize_state(self, context).await?;

        let has_next = {
            let state = context.runtime_state_mut::<RuntimeState>(|| {
                unreachable!("ForEach runtime state should be initialized")
            });
            state.index = state.index.saturating_add(1);
            state.index < state.count
        };

        Ok(if has_next {
            PostRun::Branch(BranchKind::Body)
        } else {
            PostRun::default()
        })
    }
}
