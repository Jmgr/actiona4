use action_definition::{actions::WaitForScroll, post_run::PostRun};
use actiona_core::api::mouse::ScrollConditions;

use crate::{
    ExecutionContext, PreparedWait, ResolveParam, RunError, Runnable, Waitable,
    actions::mouse::scroll::to_core_axis, run_prepared_wait,
};

impl Waitable for WaitForScroll {
    async fn prepare(&self, context: &ExecutionContext) -> Result<PreparedWait, RunError> {
        let axis = self.axis.resolve(context).await?.map(to_core_axis);

        let mouse = context.runtime.mouse()?;

        Ok(PreparedWait::new(move |token| async move {
            mouse
                .wait_for_scroll(ScrollConditions { axis }, token)
                .await?;
            Ok(())
        }))
    }
}

impl Runnable for WaitForScroll {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        run_prepared_wait(self, context).await?;
        Ok(PostRun::default())
    }
}
