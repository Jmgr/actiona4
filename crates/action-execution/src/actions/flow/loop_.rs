use action_definition::{actions::Loop, post_run::PostRun, tree::BranchKind};

use crate::{ExecutionContext, ResolveParam, RunError, RunReason, Runnable};

#[derive(Debug)]
struct RuntimeState {
    counter: u32,
}

impl Runnable for Loop {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let max_counter = self.max_counter.resolve(context).await?;
        let body_completed = matches!(
            &context.reason,
            RunReason::BranchCompleted(BranchKind::Body)
        );
        let loop_state = context.runtime_state_mut(|| RuntimeState { counter: 0 });

        if body_completed {
            loop_state.counter = loop_state.counter.saturating_add(1);
        }

        if loop_state.counter >= max_counter {
            return Ok(PostRun::NextSibling);
        }

        Ok(PostRun::Branch(BranchKind::Body))
    }
}
