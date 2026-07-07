use action_definition::{actions::SetClipboardText, post_run::PostRun};
use actiona_core::api::clipboard::ClipboardMode;

use crate::{ExecutionContext, ResolveParam, RunError, Runnable};

impl Runnable for SetClipboardText {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let text = self.text.resolve(context).await?;
        let selection = self.selection.resolve(context).await?;

        context.runtime.clipboard().set_text(
            text,
            Some(if selection {
                ClipboardMode::Selection
            } else {
                ClipboardMode::Clipboard
            }),
        )?;

        Ok(PostRun::default())
    }
}
