use action_definition::actions::message_box::MessageBox;

use crate::{ExecutionContext, PostRun, RunError, Runnable};

impl Runnable for MessageBox {
    async fn run(&self, _context: &ExecutionContext) -> Result<PostRun, RunError> {
        // TODO

        Ok(PostRun::NextSibling)
    }
}
