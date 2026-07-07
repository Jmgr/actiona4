use action_definition::{actions::DoubleClick, post_run::PostRun};
use actiona_core::api::mouse::{ClickOptions, DoubleClickOptions, PressOptions};

use crate::{
    ExecutionContext, Runnable, actions::mouse::click::to_core_button, error::RunError,
    resolve_param::ResolveParam,
};

impl Runnable for DoubleClick {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let position = self.position.resolve(context).await?;
        let button = self.button.resolve(context).await?;
        let relative_position = self.relative_position.resolve(context).await?;
        let delay = self.delay.resolve(context).await?;

        let mut options = DoubleClickOptions {
            click: ClickOptions {
                press: PressOptions {
                    button: to_core_button(button),
                    relative_position,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        if let Some(position) = position {
            options.click.press.position = Some(position.into());
        }

        if let Some(delay) = delay {
            options.delay = delay.into_inner().into();
        }

        let mouse = context.runtime.mouse()?;
        mouse
            .double_click(options, context.cancellation_token.clone())
            .await?;

        Ok(PostRun::default())
    }
}
