use action_definition::{
    parameters::{
        Param, ParamSpec, ParameterKind, duration::DurationValue, source_code::SourceCode,
    },
    scriptable::Scriptable,
};
use actiona_core::{
    api::action_result::{ActionResult, js::JsActionResult},
    api::js::duration::JsDuration,
    api::point::{Point, js::JsPoint},
    scripting::ScriptError,
};
use rquickjs::FromJs;
use unicode_segmentation::UnicodeSegmentation;

use crate::ExecutionContext;

#[derive(Debug, thiserror::Error)]
#[error("failed to resolve parameter `{parameter}`: {error}")]
pub struct ResolveParamError {
    parameter: &'static str,
    error: String,
    line: Option<u32>,
    column: Option<u32>,
}

impl ResolveParamError {
    pub fn new(parameter: &'static str, script_error: ScriptError) -> Self {
        Self {
            parameter,
            error: script_error.to_string(),
            line: script_error.line(),
            column: script_error.column(),
        }
    }

    pub fn validation(parameter: &'static str, error: impl Into<String>) -> Self {
        Self {
            parameter,
            error: error.into(),
            line: None,
            column: None,
        }
    }

    pub const fn parameter(&self) -> &'static str {
        self.parameter
    }

    pub fn error(&self) -> &str {
        &self.error
    }

    pub const fn line(&self) -> Option<u32> {
        self.line
    }

    pub const fn column(&self) -> Option<u32> {
        self.column
    }
}

/// Resolves a named action parameter against an execution context.
#[allow(async_fn_in_trait)]
pub trait ResolveParam<T> {
    async fn resolve(&self, context: &ExecutionContext) -> Result<T, ResolveParamError>;
}

/// Resolves a parameter storage value when the parameter name is supplied by
/// the typed [`Param`] wrapper.
#[allow(async_fn_in_trait)]
pub trait ResolveParamValue<T> {
    async fn resolve_value(
        &self,
        parameter: &'static str,
        context: &ExecutionContext,
    ) -> Result<T, ResolveParamError>;
}

impl<T, N, R> ResolveParam<R> for Param<T, N>
where
    T: ResolveParamValue<R>,
    N: ParamSpec,
    R: ValidateParamValue,
{
    async fn resolve(&self, context: &ExecutionContext) -> Result<R, ResolveParamError> {
        let value = self.value().resolve_value(self.name(), context).await?;
        value
            .validate_param(&N::KIND)
            .map_err(|err| ResolveParamError::validation(self.name(), err))?;
        Ok(value)
    }
}

pub trait ValidateParamValue {
    fn validate_param(&self, kind: &ParameterKind) -> Result<(), String>;
}

impl ValidateParamValue for () {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), String> {
        Ok(())
    }
}

impl ValidateParamValue for ActionResult {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), String> {
        Ok(())
    }
}

impl ValidateParamValue for String {
    fn validate_param(&self, kind: &ParameterKind) -> Result<(), String> {
        let ParameterKind::Text(settings) = kind else {
            return Ok(());
        };

        if let Some(max_length) = settings.max_length {
            let length = self.graphemes(true).count() as u64;
            if length > max_length {
                return Err(format!("must be at most {max_length} characters"));
            }
        }

        Ok(())
    }
}

impl ValidateParamValue for Point {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), String> {
        Ok(())
    }
}

impl ValidateParamValue for bool {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), String> {
        Ok(())
    }
}

impl ValidateParamValue for DurationValue {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), String> {
        Ok(())
    }
}

impl ValidateParamValue for i64 {
    fn validate_param(&self, kind: &ParameterKind) -> Result<(), String> {
        let ParameterKind::Integer(settings) = kind else {
            return Ok(());
        };

        if let Some(min) = settings.min
            && *self < min
        {
            return Err(format!("must be at least {min}"));
        }

        if let Some(max) = settings.max
            && *self > max
        {
            return Err(format!("must be at most {max}"));
        }

        Ok(())
    }
}

impl<T> ValidateParamValue for Option<T>
where
    T: ValidateParamValue,
{
    fn validate_param(&self, kind: &ParameterKind) -> Result<(), String> {
        match self {
            Some(value) => value.validate_param(kind),
            None => Ok(()),
        }
    }
}

pub trait ScriptableParamValue: Clone {
    type ScriptValue: for<'js> FromJs<'js> + Send + 'static;

    fn from_script_value(value: Self::ScriptValue) -> Self;
}

impl<T> ResolveParamValue<T> for Scriptable<T>
where
    T: ScriptableParamValue,
{
    async fn resolve_value(
        &self,
        parameter: &'static str,
        context: &ExecutionContext,
    ) -> Result<T, ResolveParamError> {
        match self {
            Scriptable::Static { value } => Ok(value.clone()),
            Scriptable::Script { source } => {
                let value = context
                    .script_engine
                    .eval_async::<T::ScriptValue>(source)
                    .await
                    .map_err(|err| ResolveParamError::new(parameter, err))?;
                Ok(T::from_script_value(value))
            }
        }
    }
}

impl ScriptableParamValue for String {
    type ScriptValue = String;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        value
    }
}

impl ScriptableParamValue for Point {
    type ScriptValue = JsPoint;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        value.into()
    }
}

impl ScriptableParamValue for i64 {
    type ScriptValue = i64;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        value
    }
}

impl ScriptableParamValue for bool {
    type ScriptValue = bool;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        value
    }
}

impl ScriptableParamValue for DurationValue {
    type ScriptValue = JsDuration;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        Self::new(value.into())
    }
}

impl<T> ScriptableParamValue for Option<T>
where
    T: ScriptableParamValue,
{
    type ScriptValue = Option<T::ScriptValue>;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        value.map(T::from_script_value)
    }
}

impl ResolveParamValue<()> for SourceCode {
    async fn resolve_value(
        &self,
        parameter: &'static str,
        context: &ExecutionContext,
    ) -> Result<(), ResolveParamError> {
        context
            .script_engine
            .eval_async::<()>(self.inner())
            .await
            .map_err(|err| ResolveParamError::new(parameter, err))?;
        Ok(())
    }
}

impl ResolveParamValue<Option<ActionResult>> for SourceCode {
    async fn resolve_value(
        &self,
        parameter: &'static str,
        context: &ExecutionContext,
    ) -> Result<Option<ActionResult>, ResolveParamError> {
        let result = context
            .script_engine
            .eval_async::<Option<JsActionResult>>(self.inner())
            .await
            .map_err(|err| ResolveParamError::new(parameter, err))?;
        Ok(result.map(JsActionResult::into_inner))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use action_definition::{
        actions::click::Click,
        parameters::{Param, ParamName, ParamSpec, ParameterKind, text::TextParameter},
        scriptable::Scriptable,
    };
    use actiona_core::runtime::{Runtime, RuntimeOptions, RuntimePlatformSetup};
    use tokio_util::sync::CancellationToken;

    use super::ResolveParam;
    use crate::{ExecutionContext, ResolveParamError};

    struct TitleParam;

    impl ParamName for TitleParam {
        const NAME: &'static str = "title";
    }

    impl ParamSpec for TitleParam {
        const KIND: ParameterKind = ParameterKind::Text(TextParameter {
            max_length: Some(5),
        });
    }

    async fn resolve_click_amount(source: &str) -> Result<Option<i64>, ResolveParamError> {
        let result = Arc::new(Mutex::new(None));
        let output = result.clone();
        let source = source.to_owned();
        let platform =
            RuntimePlatformSetup::new(false).expect("RuntimePlatformSetup::new should succeed");

        Runtime::run(
            platform,
            move |runtime, script_engine| async move {
                let click = Click {
                    amount: Scriptable::new_script(source).into(),
                    ..Default::default()
                };
                let context = ExecutionContext {
                    cancellation_token: CancellationToken::new(),
                    runtime,
                    script_engine,
                };

                *output.lock().expect("result mutex should not be poisoned") =
                    Some(click.amount.resolve(&context).await);

                Ok(())
            },
            RuntimeOptions {
                install_ctrl_c_handler: false,
                show_tray_icon: false,
                ..Default::default()
            },
        )
        .await
        .expect("runtime should run parameter resolution test");

        result
            .lock()
            .expect("result mutex should not be poisoned")
            .take()
            .expect("test should resolve parameter")
    }

    async fn resolve_title(source: &str) -> Result<String, ResolveParamError> {
        let result = Arc::new(Mutex::new(None));
        let output = result.clone();
        let source = source.to_owned();
        let platform =
            RuntimePlatformSetup::new(false).expect("RuntimePlatformSetup::new should succeed");

        Runtime::run(
            platform,
            move |runtime, script_engine| async move {
                let title: Param<Scriptable<String>, TitleParam> =
                    Scriptable::new_script(source).into();
                let context = ExecutionContext {
                    cancellation_token: CancellationToken::new(),
                    runtime,
                    script_engine,
                };

                *output.lock().expect("result mutex should not be poisoned") =
                    Some(title.resolve(&context).await);

                Ok(())
            },
            RuntimeOptions {
                install_ctrl_c_handler: false,
                show_tray_icon: false,
                ..Default::default()
            },
        )
        .await
        .expect("runtime should run parameter resolution test");

        result
            .lock()
            .expect("result mutex should not be poisoned")
            .take()
            .expect("test should resolve parameter")
    }

    #[tokio::test]
    async fn resolve_allows_zero_click_amount() {
        let amount = resolve_click_amount("0").await.unwrap();

        assert_eq!(amount, Some(0));
    }

    #[tokio::test]
    async fn resolve_rejects_click_amount_below_min() {
        let error = resolve_click_amount("-1").await.unwrap_err();

        assert_eq!(error.parameter(), "amount");
        assert_eq!(error.error(), "must be at least 0");
        assert_eq!(error.line(), None);
        assert_eq!(error.column(), None);
    }

    #[tokio::test]
    async fn resolve_rejects_click_amount_above_max() {
        let error = resolve_click_amount("2147483648").await.unwrap_err();

        assert_eq!(error.parameter(), "amount");
        assert_eq!(error.error(), "must be at most 2147483647");
        assert_eq!(error.line(), None);
        assert_eq!(error.column(), None);
    }

    #[tokio::test]
    async fn resolve_allows_text_at_max_length() {
        let title = resolve_title("'hello'").await.unwrap();

        assert_eq!(title, "hello");
    }

    #[tokio::test]
    async fn resolve_counts_text_max_length_in_graphemes() {
        let title = resolve_title("'a\\u0302👍🏽🇬🇧'").await.unwrap();

        assert_eq!(title, "a\u{0302}👍🏽🇬🇧");
    }

    #[tokio::test]
    async fn resolve_rejects_text_above_max_length() {
        let error = resolve_title("'hellos'").await.unwrap_err();

        assert_eq!(error.parameter(), "title");
        assert_eq!(error.error(), "must be at most 5 characters");
        assert_eq!(error.line(), None);
        assert_eq!(error.column(), None);
    }
}
