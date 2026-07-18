use action_definition::{actions::Goto, post_run::PostRun};

use crate::{ExecutionContext, ResolveParam, RunError, Runnable};

impl Runnable for Goto {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let target = self.target.resolve(context).await?;

        Ok(PostRun::GotoLabel(target.inner().to_owned()))
    }
}
