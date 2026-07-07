use action_definition::{
    actions::mouse::set_cursor_position::SetCursorPosition, post_run::PostRun,
};
use actiona_core::api::mouse::Coordinate;

use crate::{ExecutionContext, ResolveParam, RunError, Runnable};

impl Runnable for SetCursorPosition {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let position = self.position.resolve(context).await?;
        let relative_position = self.relative_position.resolve(context).await?;

        let mouse = context.runtime.mouse()?;
        mouse.set_position(
            position,
            if relative_position {
                Coordinate::Rel
            } else {
                Coordinate::Abs
            },
        )?;

        Ok(PostRun::default())
    }
}
