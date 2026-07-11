use action_definition::{actions::WaitForMovement, post_run::PostRun};

use crate::{ExecutionContext, PreparedWait, RunError, Runnable, Waitable, run_prepared_wait};

impl Waitable for WaitForMovement {
    async fn prepare(&self, context: &ExecutionContext) -> Result<PreparedWait, RunError> {
        let mouse = context.runtime.mouse()?;

        Ok(PreparedWait::new(move |token| async move {
            mouse.wait_for_movement(token).await?;
            Ok(())
        }))
    }
}

impl Runnable for WaitForMovement {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        run_prepared_wait(self, context).await?;
        Ok(PostRun::default())
    }
}
