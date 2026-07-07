use action_definition::{actions::Stop, post_run::PostRun};

use crate::{ExecutionContext, RunError, Runnable};

impl Runnable for Stop {
    async fn run(&self, _context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        Ok(PostRun::Stop)
    }
}
