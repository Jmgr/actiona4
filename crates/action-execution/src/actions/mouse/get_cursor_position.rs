use action_definition::{
    actions::mouse::get_cursor_position::GetCursorPosition, post_run::PostRun,
};
use actiona_core::api::point::js::JsPoint;

use crate::{ExecutionContext, ResolveParam, RunError, Runnable};

impl Runnable for GetCursorPosition {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let result = self.result.resolve(context).await?;

        let mouse = context.runtime.mouse()?;
        let position = JsPoint::from(mouse.position()?);

        context.set_variable(result.inner(), position).await?;

        Ok(PostRun::default())
    }
}
