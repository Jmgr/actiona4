use action_definition::{actions::WaitForClipboardChanged, post_run::PostRun};
use actiona_core::api::clipboard::{ClipboardMode, WaitForChangedOptions};

use crate::{ExecutionContext, ResolveParam, RunError, Runnable};

impl Runnable for WaitForClipboardChanged {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let check_interval = self.check_interval.resolve(context).await?;
        let selection = self.selection.resolve(context).await?;

        context
            .runtime
            .clipboard()
            .wait_for_changed(
                WaitForChangedOptions {
                    mode: Some(if selection {
                        ClipboardMode::Selection
                    } else {
                        ClipboardMode::Clipboard
                    }),
                    interval: check_interval.into(),
                },
                context.cancellation_token.clone(),
            )
            .await?;

        Ok(PostRun::default())
    }
}
