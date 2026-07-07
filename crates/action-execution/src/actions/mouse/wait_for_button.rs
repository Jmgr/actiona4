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
    ExecutionContext, ResolveParam, RunError, Runnable,
    actions::mouse::click::to_core_button,
    resolve_param::{ScriptableParamValue, ValidateParamValue, ValidationError},
};

fn to_core_direction(direction: ButtonDirection) -> CoreButtonDirection {
    match direction {
        ButtonDirection::Press => CoreButtonDirection::Press,
        ButtonDirection::Release => CoreButtonDirection::Release,
    }
}

fn from_js_direction(direction: JsButtonDirection) -> ButtonDirection {
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

impl Runnable for WaitForButton {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let button = self.button.resolve(context).await?.map(to_core_button);
        let direction = self
            .direction
            .resolve(context)
            .await?
            .map(to_core_direction);

        let mouse = context.runtime.mouse()?;

        mouse
            .wait_for_button(
                ButtonConditions { button, direction },
                context.cancellation_token.clone(),
            )
            .await?;

        Ok(PostRun::default())
    }
}
