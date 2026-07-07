// The generated impl refers to `ActionInstance` and each variant's field type
// by the bare names used in `action_definition`'s enum definition, so they must be in
// scope here under those exact names.
use action_definition::{actions::*, post_run::PostRun};

use crate::{ExecutionContext, error::RunError};

#[static_dispatch::setup]
#[allow(async_fn_in_trait)]
pub trait Runnable {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError>;
}

// Each `ActionInstance` variant holds `WithCommon<Action>`, so the dispatched
// impl calls `<WithCommon<Action> as Runnable>::run`. Common parameters don't
// affect how an action runs, so forward straight to the inner action.
impl<T: Runnable> Runnable for WithCommon<T> {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        self.action.run(context).await
    }
}

// `ActionInstance` must be named by its full path here: `static_dispatch` derives
// the data-macro name from the last path segment, and the macro is re-exported at
// `common::actions::__macro_data_ActionInstance`, not in this crate's scope.
static_dispatch::implementation!(Runnable for action_definition::actions::ActionInstance);
