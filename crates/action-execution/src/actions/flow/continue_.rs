use action_definition::{actions::Continue, post_run::PostRun};

use crate::{ExecutionContext, RunError, Runnable};

impl Runnable for Continue {
    async fn run(&self, _context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        Ok(PostRun::Continue)
    }
}
