use std::iter;

use action_definition::{actions::Switch, post_run::PostRun, tree::BranchKind};
use actiona_core::{api::js::deep_equal, scripting::ScriptError};
use itertools::Itertools;
use rquickjs::Value as JsValue;

use crate::{ExecutionContext, ResolveParamError, RunErrorKind, Runnable, error::RunError};

/// Identifies which `Switch` parameter a resolved script belongs to, so a
/// resolution failure can be attributed to the right place.
enum Slot {
    /// The value being switched on.
    Value(&'static str),
    /// A named case branch.
    Case(String),
}

impl Slot {
    fn resolve_error(&self, source: ScriptError) -> RunError {
        match self {
            Self::Value(parameter) => RunError::new(ResolveParamError::new(parameter, source)),
            Self::Case(branch) => RunError::new(RunErrorKind::SwitchBranchValueResolveFailed {
                branch: branch.clone(),
                source,
            }),
        }
    }
}

fn selected_branch<'js>(
    selected_value: &JsValue<'js>,
    case_values: &[JsValue<'js>],
    branch_names: &[String],
) -> Result<Option<String>, RunError> {
    for (case_value, branch_name) in case_values.iter().zip(branch_names) {
        if deep_equal(selected_value, case_value).map_err(|source| {
            RunError::new(RunErrorKind::SwitchBranchCompareFailed {
                branch: branch_name.clone(),
                source,
            })
        })? {
            return Ok(Some(branch_name.clone()));
        }
    }

    Ok(None)
}

impl Runnable for Switch {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        // Slot 0 is the value being switched on; the rest are the case values, in order.
        let slots = iter::once(Slot::Value(self.value.name()))
            .chain(self.cases.iter().map(|case| Slot::Case(case.name.clone())))
            .collect_vec();
        let scripts = iter::once(self.value.inner())
            .chain(self.cases.iter().map(|case| case.value.inner()))
            .collect_vec();
        let branch_names = self
            .cases
            .iter()
            .map(|case| case.name.clone())
            .collect_vec();

        let branch = context
            .script_engine
            .eval_async_values_fn_result(
                &scripts,
                move |values| {
                    let Some((selected_value, case_values)) = values.split_first() else {
                        return Ok(None);
                    };

                    selected_branch(selected_value, case_values, &branch_names)
                },
                move |index, source| slots[index].resolve_error(source),
            )
            .await?;

        Ok(PostRun::Branch(match branch {
            Some(branch) => BranchKind::Named(branch),
            None => BranchKind::Default,
        }))
    }
}
