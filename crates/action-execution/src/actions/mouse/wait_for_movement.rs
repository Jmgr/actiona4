use action_definition::{actions::WaitForMovement, post_run::PostRun};

use crate::{ExecutionContext, RunError, Runnable};

impl Runnable for WaitForMovement {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let mouse = context.runtime.mouse()?;

        mouse
            .wait_for_movement(context.cancellation_token.clone())
            .await?;

        Ok(PostRun::default())
    }
}
