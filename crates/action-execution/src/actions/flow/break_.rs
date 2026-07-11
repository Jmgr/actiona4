use action_definition::{actions::Break, post_run::PostRun};

use crate::{ExecutionContext, RunError, Runnable};

impl Runnable for Break {
    async fn run(&self, _context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        Ok(PostRun::Break)
    }
}
