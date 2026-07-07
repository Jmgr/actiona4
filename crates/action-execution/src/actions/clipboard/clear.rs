use action_definition::{actions::ClearClipboard, post_run::PostRun};
use actiona_core::api::clipboard::ClipboardMode;

use crate::{ExecutionContext, ResolveParam, RunError, Runnable};

impl Runnable for ClearClipboard {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let selection = self.selection.resolve(context).await?;

        context.runtime.clipboard().clear(Some(if selection {
            ClipboardMode::Selection
        } else {
            ClipboardMode::Clipboard
        }))?;

        Ok(PostRun::default())
    }
}
