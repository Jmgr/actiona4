use action_definition::{actions::Marker, post_run::PostRun};

use crate::{ExecutionContext, RunError, Runnable};

impl Runnable for Marker {
    async fn run(&self, _context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        Ok(PostRun::default())
    }
}
