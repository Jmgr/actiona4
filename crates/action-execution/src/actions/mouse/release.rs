use action_definition::{actions::Release, post_run::PostRun};

use crate::{
    ExecutionContext, Runnable, actions::mouse::click::to_core_button, error::RunError,
    resolve_param::ResolveParam,
};

impl Runnable for Release {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let button = self.button.resolve(context).await?.map(to_core_button);

        let mouse = context.runtime.mouse()?;
        mouse.release(button)?;

        Ok(PostRun::default())
    }
}
