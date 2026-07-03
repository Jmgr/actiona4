use action_definition::actions::test::Test;

use crate::{ExecutionContext, PostRun, RunError, Runnable};

impl Runnable for Test {
    async fn run(&self, _context: &ExecutionContext) -> Result<PostRun, RunError> {
        // TODO

        Ok(PostRun::NextSibling)
    }
}
