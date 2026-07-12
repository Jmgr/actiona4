use action_definition::{actions::system::code::Code, post_run::PostRun, tree::BranchKind};
use actiona_core::api::action_result::{ActionBranch, ActionResult};

use crate::{ExecutionContext, ResolveParam, Runnable, error::RunError};

fn branch_kind_from_action_branch(branch: ActionBranch) -> BranchKind {
    match branch {
        ActionBranch::Yes => BranchKind::Yes,
        ActionBranch::No => BranchKind::No,
        ActionBranch::Cancel => BranchKind::Cancel,
        ActionBranch::True => BranchKind::True,
        ActionBranch::False => BranchKind::False,
        ActionBranch::Custom(name) => BranchKind::Named(name),
    }
}

fn action_result_to_post_run(result: ActionResult) -> PostRun {
    match result {
        ActionResult::NextSibling => PostRun::NextSibling,
        ActionResult::NextChild => PostRun::NextChild,
        ActionResult::Branch(branch) => PostRun::Branch(branch_kind_from_action_branch(branch)),
        ActionResult::GotoLabel(label) => PostRun::GotoLabel(label),
        ActionResult::Stop => PostRun::Stop,
    }
}

impl Runnable for Code {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let action_result: Option<ActionResult> = self.source.resolve(context).await?;

        Ok(action_result
            .map(action_result_to_post_run)
            .unwrap_or_default())
    }
}
