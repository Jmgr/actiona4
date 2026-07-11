use action_definition::{actions::Loop, post_run::PostRun, tree::BranchKind};

use crate::{ExecutionContext, RunError, Runnable};

impl Runnable for Loop {
    async fn run(&self, _context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        Ok(PostRun::Branch(BranchKind::Body))
    }

    async fn on_body_completed(
        &self,
        _context: &mut ExecutionContext,
    ) -> Result<PostRun, RunError> {
        Ok(PostRun::Branch(BranchKind::Body))
    }
}
