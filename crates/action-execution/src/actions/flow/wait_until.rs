use action_definition::{actions::WaitUntil, post_run::PostRun};

use super::wait_condition::prepare_wait_condition;
use crate::{
    ExecutionContext, PreparedWait, ResolveParam, RunError, Runnable, Waitable, run_prepared_wait,
};

impl Waitable for WaitUntil {
    async fn prepare(&self, context: &ExecutionContext) -> Result<PreparedWait, RunError> {
        let poll_interval = self.poll_interval.resolve(context).await?.into();
        Ok(prepare_wait_condition(
            self.condition.value().clone(),
            self.condition.name(),
            poll_interval,
            context.script_engine.clone(),
            true,
        ))
    }
}

impl Runnable for WaitUntil {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        run_prepared_wait(self, context).await?;
        Ok(PostRun::default())
    }
}
