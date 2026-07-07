use action_definition::{actions::If, post_run::PostRun, tree::BranchKind};

use crate::{ExecutionContext, ResolveParam, RunError, Runnable};

impl Runnable for If {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let value = self.value.resolve(context).await?;

        Ok(if value {
            PostRun::Branch(BranchKind::True)
        } else {
            PostRun::Branch(BranchKind::False)
        })
    }
}
