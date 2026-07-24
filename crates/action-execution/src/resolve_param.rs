use action_definition::{
    parameters::{
        Param, ParamSpec, ParameterKind, array::Array, duration::DurationValue, label::Label,
        source_code::SourceCode, value::Value, variable::Variable,
    },
    scriptable::Scriptable,
};
use actiona_core::{
    api::{
        action_result::{ActionResult, js::JsActionResult},
        js::duration::JsDuration,
        point::{Point, js::JsPoint},
    },
    scripting::ScriptError,
};
use rquickjs::FromJs;
use unicode_segmentation::UnicodeSegmentation;

use crate::ExecutionContext;

#[derive(Debug, thiserror::Error)]
enum ResolveParamErrorMessage {
    #[error("{0}")]
    Script(String),
    #[error(transparent)]
    Validation(#[from] ValidationError),
}

#[derive(Debug, thiserror::Error)]
#[error("failed to resolve parameter `{parameter}`: {message}")]
pub struct ResolveParamError {
    parameter: &'static str,
    message: ResolveParamErrorMessage,
    line: Option<u32>,
    column: Option<u32>,
}

impl ResolveParamError {
    #[must_use]
    pub fn new(parameter: &'static str, script_error: &ScriptError) -> Self {
        Self {
            parameter,
            message: ResolveParamErrorMessage::Script(script_error.to_string()),
            line: script_error.line(),
            column: script_error.column(),
        }
    }

    #[must_use]
    pub const fn validation(parameter: &'static str, error: ValidationError) -> Self {
        Self {
            parameter,
            message: ResolveParamErrorMessage::Validation(error),
            line: None,
            column: None,
        }
    }

    #[must_use]
    pub const fn parameter(&self) -> &'static str {
        self.parameter
    }

    /// The validation error, if this failure came from parameter validation
    /// rather than script evaluation. A future localization layer can match
    /// on its variant instead of parsing [`Self::error`]'s text.
    #[must_use]
    pub const fn validation_error(&self) -> Option<&ValidationError> {
        match &self.message {
            ResolveParamErrorMessage::Validation(error) => Some(error),
            ResolveParamErrorMessage::Script(_) => None,
        }
    }

    #[must_use]
    pub fn error(&self) -> String {
        self.message.to_string()
    }

    #[must_use]
    pub const fn line(&self) -> Option<u32> {
        self.line
    }

    #[must_use]
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

/// A validation failure for a resolved parameter value.
///
/// Kept as a typed enum (rather than a formatted `String`) so a future
/// localization layer can match on the variant and its fields instead of
/// parsing English text.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("must be at least {min}")]
    IntegerBelowMin { min: i64 },
    #[error("must be at most {max}")]
    IntegerAboveMax { max: i64 },
    #[error("must be at least {min}")]
    DecimalBelowMin { min: f64 },
    #[error("must be at most {max}")]
    DecimalAboveMax { max: f64 },
    #[error("must be a finite number")]
    DecimalNotFinite,
    #[error("must be at least {min}")]
    UnsignedIntegerBelowMin { min: u32 },
    #[error("must be at most {max}")]
    UnsignedIntegerAboveMax { max: u32 },
    #[error("must be at most {max_length} characters")]
    TextTooLong { max_length: u64 },
}

pub trait ValidateParamValue {
    fn validate_param(&self, kind: &ParameterKind) -> Result<(), ValidationError>;
}

impl ValidateParamValue for () {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl ValidateParamValue for ActionResult {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl ValidateParamValue for String {
    fn validate_param(&self, kind: &ParameterKind) -> Result<(), ValidationError> {
        let ParameterKind::Text(settings) = kind else {
            return Ok(());
        };

        if let Some(max_length) = settings.max_length {
            let length = self.graphemes(true).count() as u64;
            if length > max_length {
                return Err(ValidationError::TextTooLong { max_length });
            }
        }

        Ok(())
    }
}

impl ValidateParamValue for Point {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl ValidateParamValue for bool {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl ValidateParamValue for DurationValue {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl ValidateParamValue for i64 {
    fn validate_param(&self, kind: &ParameterKind) -> Result<(), ValidationError> {
        let ParameterKind::Integer(settings) = kind else {
            return Ok(());
        };

        if let Some(min) = settings.min
            && *self < min
        {
            return Err(ValidationError::IntegerBelowMin { min });
        }

        if let Some(max) = settings.max
            && *self > max
        {
            return Err(ValidationError::IntegerAboveMax { max });
        }

        Ok(())
    }
}

impl ValidateParamValue for f64 {
    fn validate_param(&self, kind: &ParameterKind) -> Result<(), ValidationError> {
        if !self.is_finite() {
            return Err(ValidationError::DecimalNotFinite);
        }

        let ParameterKind::Decimal(settings) = kind else {
            return Ok(());
        };

        if let Some(min) = settings.min
            && *self < min
        {
            return Err(ValidationError::DecimalBelowMin { min });
        }

        if let Some(max) = settings.max
            && *self > max
        {
            return Err(ValidationError::DecimalAboveMax { max });
        }

        Ok(())
    }
}

impl ValidateParamValue for u32 {
    fn validate_param(&self, kind: &ParameterKind) -> Result<(), ValidationError> {
        let ParameterKind::UnsignedInteger(settings) = kind else {
            return Ok(());
        };

        if let Some(min) = settings.min
            && *self < min
        {
            return Err(ValidationError::UnsignedIntegerBelowMin { min });
        }

        if let Some(max) = settings.max
            && *self > max
        {
            return Err(ValidationError::UnsignedIntegerAboveMax { max });
        }

        Ok(())
    }
}

impl<T> ValidateParamValue for Option<T>
where
    T: ValidateParamValue,
{
    fn validate_param(&self, kind: &ParameterKind) -> Result<(), ValidationError> {
        match self {
            Some(value) => value.validate_param(kind),
            None => Ok(()),
        }
    }
}

impl ValidateParamValue for Label {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl ValidateParamValue for Variable {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), ValidationError> {
        // TODO: validate
        Ok(())
    }
}

impl ValidateParamValue for Array {
    fn validate_param(&self, _kind: &ParameterKind) -> Result<(), ValidationError> {
        Ok(())
    }
}

impl ResolveParamValue<Self> for Variable {
    async fn resolve_value(
        &self,
        _parameter: &'static str,
        _context: &ExecutionContext,
    ) -> Result<Self, ResolveParamError> {
        Ok(self.clone())
    }
}

impl ResolveParamValue<Self> for Array {
    async fn resolve_value(
        &self,
        _parameter: &'static str,
        _context: &ExecutionContext,
    ) -> Result<Self, ResolveParamError> {
        Ok(self.clone())
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
            Self::Static { value } => Ok(value.clone()),
            Self::Script { source } => {
                let value = context
                    .script_engine
                    .eval_async::<T::ScriptValue>(source)
                    .await
                    .map_err(|err| ResolveParamError::new(parameter, &err))?;
                Ok(T::from_script_value(value))
            }
        }
    }
}

impl ScriptableParamValue for String {
    type ScriptValue = Self;

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
    type ScriptValue = Self;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        value
    }
}

impl ScriptableParamValue for f64 {
    type ScriptValue = Self;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        value
    }
}

impl ScriptableParamValue for u32 {
    type ScriptValue = Self;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        value
    }
}

impl ScriptableParamValue for bool {
    type ScriptValue = Self;

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

impl ScriptableParamValue for Label {
    type ScriptValue = String;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        Self::new(value)
    }
}

impl ScriptableParamValue for Variable {
    type ScriptValue = String;

    fn from_script_value(value: Self::ScriptValue) -> Self {
        Self::new(value)
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
            .map_err(|err| ResolveParamError::new(parameter, &err))?;
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
            .map_err(|err| ResolveParamError::new(parameter, &err))?;
        Ok(result.map(JsActionResult::into_inner))
    }
}

impl<T> ResolveParamValue<T> for Value
where
    for<'any_js> T: FromJs<'any_js> + Send + 'static,
{
    async fn resolve_value(
        &self,
        parameter: &'static str,
        context: &ExecutionContext,
    ) -> Result<T, ResolveParamError> {
        let result = context
            .script_engine
            .eval_async::<T>(self.inner())
            .await
            .map_err(|err| ResolveParamError::new(parameter, &err))?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use action_definition::{
        actions::mouse::click::Click,
        parameters::{Param, ParamName, ParamSpec, ParameterKind, text::TextParameter},
        scriptable::Scriptable,
    };
    use actiona_core::runtime::{Runtime, RuntimeOptions, RuntimePlatformSetup};
    use parking_lot::Mutex;
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
                let context =
                    ExecutionContext::new(CancellationToken::new(), runtime, script_engine);

                *output.lock() = Some(click.amount.resolve(&context).await);

                Ok(())
            },
            RuntimeOptions {
                install_ctrl_c_handler: false,
                show_tray_icon: false,
                discover_extensions: false,
                ..Default::default()
            },
        )
        .await
        .expect("runtime should run parameter resolution test");

        result.lock().take().expect("test should resolve parameter")
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
                let context =
                    ExecutionContext::new(CancellationToken::new(), runtime, script_engine);

                *output.lock() = Some(title.resolve(&context).await);

                Ok(())
            },
            RuntimeOptions {
                install_ctrl_c_handler: false,
                show_tray_icon: false,
                discover_extensions: false,
                ..Default::default()
            },
        )
        .await
        .expect("runtime should run parameter resolution test");

        result.lock().take().expect("test should resolve parameter")
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

    #[test]
    fn validate_rejects_non_finite_decimal() {
        use action_definition::parameters::{ParameterKind, decimal::DecimalParameter};

        use super::{ValidateParamValue, ValidationError};

        let kind = ParameterKind::Decimal(DecimalParameter {
            min: Some(0.0),
            max: None,
        });

        assert!(matches!(
            f64::NAN.validate_param(&kind),
            Err(ValidationError::DecimalNotFinite)
        ));
        assert!(matches!(
            f64::INFINITY.validate_param(&kind),
            Err(ValidationError::DecimalNotFinite)
        ));
        assert!(matches!(
            f64::NEG_INFINITY.validate_param(&kind),
            Err(ValidationError::DecimalNotFinite)
        ));
        assert!(1.5_f64.validate_param(&kind).is_ok());
    }
}
