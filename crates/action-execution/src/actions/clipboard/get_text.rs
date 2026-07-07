use action_definition::{actions::GetClipboardText, post_run::PostRun};
use actiona_core::api::clipboard::ClipboardMode;

use crate::{ExecutionContext, ResolveParam, RunError, Runnable};

impl Runnable for GetClipboardText {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let result = self.result.resolve(context).await?;
        let selection = self.selection.resolve(context).await?;

        let value = context.runtime.clipboard().get_text(Some(if selection {
            ClipboardMode::Selection
        } else {
            ClipboardMode::Clipboard
        }))?;

        context.set_variable(result.inner(), value).await?;

        Ok(PostRun::default())
    }
}
