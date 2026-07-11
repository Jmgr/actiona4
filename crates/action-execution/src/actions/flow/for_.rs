use action_definition::{
    actions::For, parameters::variable::Variable, post_run::PostRun, tree::BranchKind,
};

use crate::{ExecutionContext, ResolveParam, RunError, Runnable};

#[derive(Debug)]
struct RuntimeState {
    count: u32,
    index: u32,
    index_variable: Variable,
}

async fn initialize_state(action: &For, context: &mut ExecutionContext) -> Result<(), RunError> {
    if context.runtime_state::<RuntimeState>().is_some() {
        return Ok(());
    }

    let count = action.count.resolve(context).await?;
    let index_variable = action.index_variable.resolve(context).await?;

    context.runtime_state_mut(|| RuntimeState {
        count,
        index: 0,
        index_variable,
    });
    Ok(())
}

impl Runnable for For {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        initialize_state(self, context).await?;

        let count = context
            .runtime_state::<RuntimeState>()
            .expect("For runtime state should be initialized")
            .count;

        Ok(if count == 0 {
            PostRun::default()
        } else {
            PostRun::Branch(BranchKind::Body)
        })
    }

    async fn on_body_enter(&self, context: &mut ExecutionContext) -> Result<(), RunError> {
        initialize_state(self, context).await?;

        let (index_variable, index) = {
            let state = context
                .runtime_state::<RuntimeState>()
                .expect("For runtime state should be initialized");
            (state.index_variable.clone(), state.index)
        };

        context.set_variable(index_variable.inner(), index).await?;
        Ok(())
    }

    async fn on_body_completed(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        initialize_state(self, context).await?;

        let has_next = {
            let state = context.runtime_state_mut::<RuntimeState>(|| {
                unreachable!("For runtime state should be initialized")
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
