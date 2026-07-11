use action_definition::{actions::While, post_run::PostRun, tree::BranchKind};

use crate::{ExecutionContext, ResolveParam, RunError, Runnable};

impl Runnable for While {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let condition = self.condition.resolve(context).await?;

        Ok(if condition {
            PostRun::Branch(BranchKind::Body)
        } else {
            PostRun::default()
        })
    }

    async fn on_body_completed(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        self.run(context).await
    }
}
