use action_definition::{actions::WaitForClipboardChanged, post_run::PostRun};
use actiona_core::api::clipboard::{ClipboardMode, WaitForChangedOptions};

use crate::{
    ExecutionContext, PreparedWait, ResolveParam, RunError, Runnable, Waitable, run_prepared_wait,
};

impl Waitable for WaitForClipboardChanged {
    async fn prepare(&self, context: &ExecutionContext) -> Result<PreparedWait, RunError> {
        let check_interval = self.check_interval.resolve(context).await?;
        let selection = self.selection.resolve(context).await?;
        let clipboard = context.runtime.clipboard();
        let options = WaitForChangedOptions {
            mode: Some(if selection {
                ClipboardMode::Selection
            } else {
                ClipboardMode::Clipboard
            }),
            interval: check_interval.into(),
        };

        Ok(PreparedWait::new(move |token| async move {
            clipboard.wait_for_changed(options, token).await?;
            Ok(())
        }))
    }
}

impl Runnable for WaitForClipboardChanged {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        run_prepared_wait(self, context).await?;
        Ok(PostRun::default())
    }
}
