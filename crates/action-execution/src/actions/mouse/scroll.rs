use action_definition::{
    actions::{Scroll, scroll::Axis},
    parameters::ParameterKind,
    post_run::PostRun,
};
use actiona_core::api::mouse::{Axis as CoreAxis, JsAxis};
use satint::SaturatingInto;

use crate::{
    ExecutionContext, Runnable,
    error::RunError,
    resolve_param::{ResolveParam, ScriptableParamValue, ValidateParamValue, ValidationError},
};

pub(crate) fn to_core_axis(axis: Axis) -> CoreAxis {
    match axis {
        Axis::Horizontal => CoreAxis::Horizontal,
        Axis::Vertical => CoreAxis::Vertical,
    }
}

fn from_core_axis(axis: CoreAxis) -> Axis {
    match axis {
        CoreAxis::Horizontal => Axis::Horizontal,
        CoreAxis::Vertical => Axis::Vertical,
    }
}

impl ScriptableParamValue for Axis {
    type ScriptValue = JsAxis;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        from_core_axis(value)
    }
}

impl ValidateParamValue for Axis {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl Runnable for Scroll {
    async fn run(&self, context: &mut ExecutionContext) -> Result<PostRun, RunError> {
        let amount = self.amount.resolve(context).await?;
        let axis = self.axis.resolve(context).await?;

        let mouse = context.runtime.mouse()?;
        mouse.scroll(amount.saturating_into(), to_core_axis(axis))?;

        Ok(PostRun::default())
    }
}
