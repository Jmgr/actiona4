use action_definition::{actions::ButtonCondition, post_run::PostRun, tree::BranchKind};

use crate::{
    ExecutionContext, Runnable, actions::mouse::click::to_core_button, error::RunError,
    resolve_param::ResolveParam,
};

impl Runnable for ButtonCondition {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let button = self.button.resolve(context).await?;

        let mouse = context.runtime.mouse()?;

        Ok(if mouse.is_pressed(to_core_button(button))? {
            PostRun::Branch(BranchKind::Pressed)
        } else {
            PostRun::Branch(BranchKind::Released)
        })
    }
}
