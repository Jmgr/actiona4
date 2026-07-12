use action_definition::{actions::RandomBranch, post_run::PostRun, tree::BranchKind};

use crate::{ExecutionContext, RunError, Runnable};

impl Runnable for RandomBranch {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        if self.branches.is_empty() {
            return Ok(PostRun::default());
        }

        let index = context.runtime.rng().random_range(0..self.branches.len());
        Ok(PostRun::Branch(BranchKind::Named(
            self.branches[index].clone(),
        )))
    }
}
