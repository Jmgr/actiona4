use action_definition::{actions::Or, post_run::PostRun, tree::BranchKind};

use crate::{ExecutionContext, RunError, Runnable, prepare_inputs, race_waits};

impl Runnable for Or {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let prepared = prepare_inputs(&self.inputs, context, "Or").await?;
        let winner = race_waits(prepared, &context.cancellation_token).await?;
        Ok(PostRun::Branch(BranchKind::Named(winner.to_string())))
    }
}
