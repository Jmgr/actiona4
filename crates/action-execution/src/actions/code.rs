use action_definition::actions::code::Code;

use crate::{ExecutionContext, PostRun, RunError, Runnable};

impl Runnable for Code {
    async fn run(&self, _context: &ExecutionContext) -> Result<PostRun, RunError> {
        // TODO

        Ok(PostRun::NextSibling)
    }
}
