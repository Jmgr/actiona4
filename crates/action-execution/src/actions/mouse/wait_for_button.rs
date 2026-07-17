use action_definition::{
    actions::{WaitForButton, wait_for_button::ButtonDirection},
    parameters::ParameterKind,
    post_run::PostRun,
};
use actiona_core::{
    api::mouse::{ButtonConditions, JsButtonDirection},
    types::input::Direction as CoreButtonDirection,
};

use crate::{
    ExecutionContext, PreparedWait, ResolveParam, RunError, Runnable, Waitable,
    actions::mouse::click::to_core_button,
    resolve_param::{ScriptableParamValue, ValidateParamValue, ValidationError},
    run_prepared_wait,
};

const fn to_core_direction(direction: ButtonDirection) -> CoreButtonDirection {
    match direction {
        ButtonDirection::Press => CoreButtonDirection::Press,
        ButtonDirection::Release => CoreButtonDirection::Release,
    }
}

const fn from_js_direction(direction: JsButtonDirection) -> ButtonDirection {
    match direction {
        JsButtonDirection::Press => ButtonDirection::Press,
        JsButtonDirection::Release => ButtonDirection::Release,
    }
}

impl ScriptableParamValue for ButtonDirection {
    type ScriptValue = JsButtonDirection;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        from_js_direction(value)
    }
}

impl ValidateParamValue for ButtonDirection {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl Waitable for WaitForButton {
    async fn prepare(&self, context: &ExecutionContext) -> Result<PreparedWait, RunError> {
        let button = self.button.resolve(context).await?.map(to_core_button);
        let direction = self
            .direction
            .resolve(context)
            .await?
            .map(to_core_direction);

        let mouse = context.runtime.mouse()?;

        Ok(PreparedWait::new(move |token| async move {
            mouse
                .wait_for_button(ButtonConditions { button, direction }, token)
                .await?;
            Ok(())
        }))
    }
}

impl Runnable for WaitForButton {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        run_prepared_wait(self, context).await?;
        Ok(PostRun::default())
    }
}
