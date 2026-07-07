use action_definition::{actions::Exit, post_run::PostRun};

use crate::{ExecutionContext, RunError, Runnable};

impl Runnable for Exit {
    async fn run(&self, _context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        Ok(PostRun::Exit)
    }
}
