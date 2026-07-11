use action_definition::{actions::And, post_run::PostRun};

use crate::{ExecutionContext, RunError, Runnable, join_waits, prepare_inputs};

impl Runnable for And {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let prepared = prepare_inputs(&self.inputs, context, "And").await?;
        join_waits(prepared, &context.cancellation_token).await?;
        Ok(PostRun::default())
    }
}
