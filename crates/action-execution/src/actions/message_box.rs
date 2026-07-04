use action_definition::{actions::message_box::MessageBox, post_run::PostRun};

use crate::{ExecutionContext, Runnable, error::RunError};

impl Runnable for MessageBox {
    async fn run(&self, _context: &ExecutionContext) -> Result<PostRun, RunError> {
        /*
        let title = self.title.resolve(context).await?;
        let text = self.text.resolve(context).await?;
        let buttons = self.buttons;
        */

        // TODO

        Ok(PostRun::default())
    }
}
