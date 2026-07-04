use action_definition::{
    actions::click::{Click, MouseButton},
    parameters::ParameterKind,
    post_run::PostRun,
};
use actiona_core::api::mouse::{Button, ClickOptions, PressOptions};

use crate::{
    ExecutionContext, Runnable,
    error::RunError,
    resolve_param::{ResolveParam, ScriptableParamValue, ValidateParamValue},
};

fn to_core_button(button: MouseButton) -> Button {
    match button {
        MouseButton::Left => Button::Left,
        MouseButton::Middle => Button::Middle,
        MouseButton::Right => Button::Right,
        MouseButton::Back => Button::Back,
        MouseButton::Forward => Button::Forward,
    }
}

fn from_core_button(button: Button) -> MouseButton {
    match button {
        Button::Left => MouseButton::Left,
        Button::Middle => MouseButton::Middle,
        Button::Right => MouseButton::Right,
        Button::Back => MouseButton::Back,
        Button::Forward => MouseButton::Forward,
    }
}

impl ScriptableParamValue for MouseButton {
    type ScriptValue = Button;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        from_core_button(value)
    }
}

impl ValidateParamValue for MouseButton {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), String> {
        Ok(())
    }
}

impl Runnable for Click {
    async fn run(&self, context: &ExecutionContext) -> Result<PostRun, RunError> {
        let position = self.position.resolve(context).await?;
        let button = self.button.resolve(context).await?;
        let relative_position = self.relative_position.resolve(context).await?;
        let amount = self.amount.resolve(context).await?;
        let interval = self.interval.resolve(context).await?;
        let duration = self.duration.resolve(context).await?;

        let mut options = ClickOptions {
            press: PressOptions {
                button: to_core_button(button),
                relative_position,
                ..Default::default()
            },
            ..Default::default()
        };

        if let Some(position) = position {
            options.press.position = Some(position.into());
        }

        if let Some(amount) = amount {
            options.amount =
                i32::try_from(amount).map_err(|err| eyre::eyre!("invalid click amount: {err}"))?;
        }

        if let Some(interval) = interval {
            options.interval = interval.into_inner().into();
        }

        if let Some(duration) = duration {
            options.duration = duration.into_inner().into();
        }

        let mouse = context.runtime.mouse()?;
        mouse
            .click(options, context.cancellation_token.clone())
            .await?;

        Ok(PostRun::default())
    }
}
