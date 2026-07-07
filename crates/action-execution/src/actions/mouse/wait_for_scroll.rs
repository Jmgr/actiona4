use action_definition::{actions::WaitForScroll, post_run::PostRun};
use actiona_core::api::mouse::ScrollConditions;

use crate::{
    ExecutionContext, ResolveParam, RunError, Runnable, actions::mouse::scroll::to_core_axis,
};

impl Runnable for WaitForScroll {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let axis = self.axis.resolve(context).await?.map(to_core_axis);

        let mouse = context.runtime.mouse()?;

        mouse
            .wait_for_scroll(
                ScrollConditions { axis },
                context.cancellation_token.clone(),
            )
            .await?;

        Ok(PostRun::default())
    }
}
