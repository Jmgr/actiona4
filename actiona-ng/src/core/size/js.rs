use rquickjs::{
    Ctx, Exception, JsLifetime, Result, Value,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::{FromParam, ParamRequirement, ParamsAccessor},
    prelude::*,
};

use crate::{
    IntoJsResult,
    core::{ResultExt, js::classes::ValueClass, size::try_size},
};

pub struct JsSizeParam(pub super::Size);

impl<'js> FromParam<'js> for JsSizeParam {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::exhaustive()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        Ok(Self(match params.len() {
            n if n >= 2 => super::Size::new(params.arg().get()?, params.arg().get()?),
            n if n >= 1 => {
                let value = params.arg();

                // Also accept a JsSize as a parameter
                if let Ok(js_size) = value.get::<JsSize>() {
                    return Ok(Self(js_size.into()));
                }

                let object = value
                    .as_object()
                    .or_throw_message(params.ctx(), "Expected an object")?;

                super::Size::new(object.get("width")?, object.get("height")?)
            }
            n => {
                return Err(Exception::throw_message(
                    params.ctx(),
                    &format!("Unexpected number of parameter: {n}"),
                ));
            }
        }))
    }
}

/// A size.
///
/// @prop width: number // width
/// @prop height: number // height
///
/// ```js
/// let p = new Size(1, 2);
/// ```
#[derive(Clone, Copy, Debug, Eq, JsLifetime, PartialEq)]
#[rquickjs::class(rename = "Size")]
pub struct JsSize {
    inner: super::Size,
}

impl ValueClass<'_> for JsSize {}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsSize {
    /// Constructor.
    ///
    /// @constructor
    ///
    /// @overload
    /// Constructor with two numbers.
    /// @param width: number // width
    /// @param height: number // height
    ///
    /// @overload
    /// Constructor with an object.
    /// @param o: {width: number, height: number} // Object containing the width and height
    ///
    /// @overload
    /// Constructor with another Size.
    /// @param p: Size // Other size
    #[qjs(constructor)]
    pub fn new<'js>(ctx: Ctx<'js>, args: Rest<Value<'js>>) -> Result<Self> {
        let (size, _) = Self::from_args(&ctx, &args.0)?;
        Ok(size)
    }

    /// Constructs a Size from an argument slice.
    /// Accepted forms:
    /// new Size(other_size)
    /// new Size({width: 0, height: 1})
    /// new Size(0, 1)
    ///
    /// @skip
    #[qjs(skip)]
    pub fn from_args<'a, 'js>(
        // TODO: remove?
        ctx: &Ctx<'js>,
        args: &'a [Value<'js>],
    ) -> Result<(Self, &'a [Value<'js>])> {
        // Get the mandatory first argument
        let (first_arg, rest) = args
            .split_first()
            .or_throw_message(ctx, "Expected at least one argument")?;

        // If the first argument is a number, expect a second argument to also be a number
        if let Some(first_arg) = first_arg.as_number() {
            let (second_arg, rest) = rest
                .split_first()
                .or_throw_message(ctx, "Expected second argument")?;
            let second_arg = second_arg
                .as_number()
                .or_throw_message(ctx, "Expected second argument to be a number")?;

            let size = try_size(first_arg, second_arg).into_js_result(ctx)?;

            return Ok((size.into(), rest));
        }

        // If it's a Size then get a copy
        if let Ok(other_size) = first_arg.get::<JsSize>() {
            return Ok((other_size, rest));
        }

        // If it's an object, then get its width and height properties
        if let Some(first_arg) = first_arg.as_object() {
            let width: f64 = first_arg.get("width")?;
            let height: f64 = first_arg.get("height")?;

            let size = try_size(width, height).into_js_result(ctx)?;

            return Ok((size.into(), rest));
        }

        Err(Exception::throw_message(ctx, "Invalid Size argument"))
    }

    /// @skip
    #[qjs(get, rename = "width")]
    #[must_use]
    pub fn get_width(&self) -> u32 {
        self.inner.width.into()
    }

    /// @skip
    #[qjs(set, rename = "width")]
    pub fn set_width(&mut self, width: u32) {
        self.inner.width = width.into();
    }

    /// @skip
    #[qjs(get, rename = "height")]
    #[must_use]
    pub fn get_height(&self) -> u32 {
        self.inner.height.into()
    }

    /// @skip
    #[qjs(set, rename = "height")]
    pub fn set_height(&mut self, height: u32) {
        self.inner.height = height.into();
    }

    /// Returns a JSON representation of this Size.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).unwrap()
    }

    /// Returns true if a Size equals another.
    #[must_use]
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    /// Adds two sizes and returns a new Size.
    #[must_use]
    pub fn add(&self, other: Self) -> Self {
        (self.inner + other.inner).into()
    }

    /// Subtracts two sizes and returns a new Size.
    #[must_use]
    pub fn subtract(&self, other: Self) -> Self {
        (self.inner - other.inner).into()
    }

    /// Scales this size by a factor and returns a new Size.
    pub fn scale<'js>(&self, ctx: Ctx<'js>, factor: f64) -> Result<Self> {
        self.inner
            .scaled(factor)
            .map(|value| value.into())
            .into_js_result(&ctx)
    }

    /// Returns a string representation of this Size.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        format!("({}, {})", self.inner.width, self.inner.height)
    }

    /// Clones this Size.
    #[qjs(rename = "clone")]
    #[must_use]
    pub const fn clone_js(&self) -> Self {
        *self
    }

    /// @skip
    #[must_use]
    #[qjs(skip)]
    pub const fn inner(&self) -> super::Size {
        self.inner
    }
}

impl<'js> Trace<'js> for JsSize {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl From<JsSize> for super::Size {
    fn from(value: JsSize) -> Self {
        value.inner
    }
}

impl From<super::Size> for JsSize {
    fn from(value: super::Size) -> Self {
        Self { inner: value }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::JsSize;
    use crate::{core::size::size, runtime::Runtime, scripting::Engine as ScriptEngine};

    async fn setup(script_engine: Arc<ScriptEngine>) {
        script_engine
            .eval::<()>(
                r#"
                let s1 = new Size({width: 1, height: 2});
                let s2 = new Size(2, 3);
                let s3 = new Size(s2);
            "#,
            )
            .await
            .unwrap();
    }

    #[test]
    fn test_size_equals() {
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

            let result = script_engine.eval::<bool>("s1 == s2").await.unwrap();
            assert!(!result);

            let result = script_engine.eval::<bool>("s1 != s2").await.unwrap();
            assert!(result);

            let result = script_engine.eval::<bool>("s2.equals(s3)").await.unwrap();
            assert!(result);
        });
    }

    #[test]
    fn test_size_attributes() {
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

            script_engine
                .eval::<()>(
                    r#"
                s1.width = 42;
                s1.height = 43;
            "#,
                )
                .await
                .unwrap();

            let result = script_engine.eval::<i64>("s1.width").await.unwrap();
            assert_eq!(result, 42);

            let result = script_engine.eval::<i64>("s1.height").await.unwrap();
            assert_eq!(result, 43);
        });
    }

    #[test]
    fn test_add_subtract_scale() {
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

            let result = script_engine
                .eval::<JsSize>("s1.add(new Size(1, 3))")
                .await
                .unwrap();
            assert_eq!(result, size(2, 5).into());

            let result = script_engine
                .eval::<JsSize>("s1.subtract(new Size(1, 3))")
                .await
                .unwrap();
            assert_eq!(result, size(0, 0).into());

            let result = script_engine.eval::<JsSize>("s1.scale(2)").await.unwrap();
            assert_eq!(result, size(2, 4).into());
        });
    }

    #[test]
    fn test_json() {
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

            let result = script_engine.eval::<String>("s1.toJson()").await.unwrap();
            assert_eq!(result, r#"{"width":1,"height":2}"#);
        });
    }

    #[test]
    fn test_clone() {
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

            script_engine
                .eval::<()>("let sc = s1.clone()")
                .await
                .unwrap();

            let result = script_engine.eval::<bool>("sc.equals(s1)").await.unwrap();
            assert!(result);

            let result = script_engine.eval::<bool>("sc == s1").await.unwrap();
            assert!(!result);
        });
    }
}
