use action_definition::{actions::Press, post_run::PostRun};
use actiona_core::api::mouse::PressOptions;

use crate::{
    ExecutionContext, Runnable, actions::mouse::click::to_core_button, error::RunError,
    resolve_param::ResolveParam,
};

impl Runnable for Press {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let position = self.position.resolve(context).await?;
        let button = self.button.resolve(context).await?;
        let relative_position = self.relative_position.resolve(context).await?;

        let mut options = PressOptions {
            button: to_core_button(button),
            relative_position,
            ..Default::default()
        };

        if let Some(position) = position {
            options.position = Some(position.into());
        }

        let mouse = context.runtime.mouse()?;
        mouse.press(options)?;

        Ok(PostRun::default())
    }
}
