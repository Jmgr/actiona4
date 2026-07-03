use action_definition::actions::click::Click;

use crate::{ExecutionContext, PostRun, RunError, Runnable, resolve::Resolve};

impl Runnable for Click {
    async fn run(&self, context: &ExecutionContext) -> Result<PostRun, RunError> {
        let _position = self.position.resolve(context)?;

        // TODO: click at position

        Ok(PostRun::NextSibling)
    }
}
