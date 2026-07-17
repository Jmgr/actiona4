use std::time::Duration;

use action_definition::scriptable::Scriptable;
use actiona_core::scripting::Engine as ScriptEngine;
use tokio::{select, time::sleep};

use crate::{PreparedWait, ResolveParamError, RunError, RunErrorKind};

pub fn prepare_wait_condition(
    condition: Scriptable<bool>,
    condition_parameter: &'static str,
    poll_interval: Duration,
    script_engine: ScriptEngine,
    completion_value: bool,
) -> PreparedWait {
    PreparedWait::new(move |token| async move {
        loop {
            let condition_value = select! {
                biased;
                () = token.cancelled() => return Err(RunError::new(RunErrorKind::Canceled)),
                result = async {
                    match &condition {
                        Scriptable::Static { value } => Ok(*value),
                        Scriptable::Script { source } => script_engine
                            .eval_async(source)
                            .await
                            .map_err(|source| ResolveParamError::new(condition_parameter, source)),
                    }
                } => result?,
            };

            if condition_value == completion_value {
                return Ok(());
            }

            select! {
                biased;
                () = token.cancelled() => return Err(RunError::new(RunErrorKind::Canceled)),
                () = sleep(poll_interval) => {}
            }
        }
    })
}
