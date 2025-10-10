use rquickjs::{
    Ctx, Exception, JsLifetime, Result, Value,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::{FromParam, ParamRequirement, ParamsAccessor},
    prelude::*,
};

use crate::{
    core::{ResultExt, js::classes::ValueClass},
    runtime::WithUserData,
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

                // Also accept a js::Point as a parameter
                if let Ok(js_point) = value.get::<JsSize>() {
                    return Ok(Self(js_point.into()));
                }

                let object = value
                    .as_object()
                    .or_throw_message(params.ctx(), "Expected an object")?;

                super::Size::new(object.get("width")?, object.get("y")?)
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

/// A 2D Point.
///
/// @prop x: number // X coordinate
/// @prop y: number // Y coordinate
///
/// ```js
/// let p = new Point(1, 2);
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
    /// Constructor with two number.
    /// @param x: number // X coordinate
    /// @param y: number // Y coordinate
    ///
    /// @overload
    /// Constructor with an object.
    /// @param o: {x: number, y: number} // Object containing the x and y coordinates
    ///
    /// @overload
    /// Constructor with another Point.
    /// @param p: Point // Other point
    #[qjs(constructor)]
    pub fn new<'js>(ctx: Ctx<'js>, args: Rest<Value<'js>>) -> Result<Self> {
        let (point, _) = Self::from_args(&ctx, &args.0)?;
        Ok(point)
    }

    /// Constructs a Point from an argument slice.
    /// Accepted forms:
    /// new Point(other_point)
    /// new Point({x: 0, y: 1})
    /// new Point(0, 1)
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

            return Ok((super::size(first_arg, second_arg).into(), rest));
        }

        // If it's a Point then get a copy
        if let Ok(other_point) = first_arg.get::<JsSize>() {
            return Ok((other_point, rest));
        }

        // If it's an object, then get its x and y properties
        if let Some(first_arg) = first_arg.as_object() {
            let width: f64 = first_arg.get("width")?;
            let y: f64 = first_arg.get("y")?;

            return Ok((super::size(width, y).into(), rest));
        }

        Err(Exception::throw_message(ctx, "Invalid Point argument"))
    }

    /// @skip
    #[qjs(get, rename = "width")]
    pub const fn get_width(&self) -> u32 {
        self.inner.width
    }

    /// @skip
    #[qjs(set, rename = "width")]
    pub const fn set_width(&mut self, width: u32) {
        self.inner.width = width;
    }

    /// @skip
    #[qjs(get, rename = "height")]
    pub const fn get_height(&self) -> u32 {
        self.inner.height
    }

    /// @skip
    #[qjs(set, rename = "height")]
    pub const fn set_height(&mut self, height: u32) {
        self.inner.height = height;
    }

    /// Length of this point.
    pub fn length(&self) -> f32 {
        self.inner.length()
    }

    /// Normalize the point.
    pub fn normalize(self) -> Self {
        self.inner.normalize().into()
    }

    /// Calculates the distance between this point and another.
    pub fn distance_to(&self, other: Self) -> f32 {
        self.inner.distance_to(other.into())
    }

    /// Returns a JSON representation of this Point.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).unwrap()
    }

    /// Returns true if this Point is at the origin, (0, 0).
    pub const fn is_origin(&self) -> bool {
        self.inner.is_origin()
    }

    /// Computes the distance between two points.
    #[qjs(static)]
    pub fn distance(a: Self, b: Self) -> f32 {
        a.distance_to(b)
    }

    /// Returns true if a Point equals another.
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    /// Adds two points and returns a new Point.
    pub fn add(&self, other: Self) -> Self {
        (self.inner + other.inner).into()
    }

    /// Subtracts two points and returns a new Point.
    pub fn subtract(&self, other: Self) -> Self {
        (self.inner - other.inner).into()
    }

    /// Scales this point by a factor and returns a new Point.
    pub fn scale(&self, factor: f32) -> Self {
        self.inner.scale(factor).into()
    }

    /// Returns a string representation of this Point.
    #[qjs(rename = PredefinedAtom::ToString)]
    pub fn to_string_js(&self) -> String {
        format!("({}, {})", self.inner.width, self.inner.height)
    }

    /// Clones this Point.
    #[qjs(rename = "clone")]
    pub const fn clone_js(&self) -> Self {
        *self
    }

    /// @skip
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
    use super::JsSize;
    use crate::{core::size::size, runtime::Runtime, scripting::Engine as ScriptEngine};

    async fn setup(script_engine: &mut ScriptEngine) {
        script_engine
            .eval::<()>(
                r#"
                let p1 = new Point({x: 1, y: 2});
                let p2 = new Point(2, 3);
                let p3 = new Point(p2);
            "#,
            )
            .await
            .unwrap();
    }

    #[test]
    fn test_point_equals() {
        Runtime::test_with_script_engine(async |mut script_engine| {
            setup(&mut script_engine).await;

            let result = script_engine.eval::<bool>("p1 == p2").await.unwrap();
            assert!(!result);

            let result = script_engine.eval::<bool>("p1 != p2").await.unwrap();
            assert!(result);

            let result = script_engine.eval::<bool>("p2.equals(p3)").await.unwrap();
            assert!(result);
        });
    }

    #[test]
    fn test_point_attributes() {
        Runtime::test_with_script_engine(async |mut script_engine| {
            setup(&mut script_engine).await;

            script_engine
                .eval::<()>(
                    r#"
                p1.x = 42;
                p1.y = 43;
            "#,
                )
                .await
                .unwrap();

            let result = script_engine.eval::<i64>("p1.x").await.unwrap();
            assert_eq!(result, 42);

            let result = script_engine.eval::<i64>("p1.y").await.unwrap();
            assert_eq!(result, 43);
        });
    }

    #[test]
    fn test_add_subtract_scale() {
        Runtime::test_with_script_engine(async |mut script_engine| {
            setup(&mut script_engine).await;

            let result = script_engine
                .eval::<JsSize>("p1.add(new Point(1, 3))")
                .await
                .unwrap();
            assert_eq!(result, size(2, 5).into());

            let result = script_engine
                .eval::<JsSize>("p1.subtract(new Point(1, 3))")
                .await
                .unwrap();
            assert_eq!(result, size(0, -1).into());

            let result = script_engine.eval::<JsSize>("p1.scale(2)").await.unwrap();
            assert_eq!(result, size(2, 4).into());
        });
    }

    #[test]
    fn test_distance() {
        Runtime::test_with_script_engine(async |mut script_engine| {
            setup(&mut script_engine).await;

            let result = script_engine
                .eval::<f32>("p1.distanceTo(new Point(4, 6))")
                .await
                .unwrap();
            assert_eq!(result, 5.);

            let result = script_engine
                .eval::<f32>("Point.distance(p1, new Point(4, 6))")
                .await
                .unwrap();
            assert_eq!(result, 5.);
        });
    }

    #[test]
    fn test_json() {
        Runtime::test_with_script_engine(async |mut script_engine| {
            setup(&mut script_engine).await;

            let result = script_engine.eval::<String>("p1.toJson()").await.unwrap();
            assert_eq!(result, r#"{"x":1,"y":2}"#);
        });
    }

    #[test]
    fn test_origin() {
        Runtime::test_with_script_engine(async |mut script_engine| {
            setup(&mut script_engine).await;

            let result = script_engine.eval::<bool>("p1.isOrigin()").await.unwrap();
            assert!(!result);

            let result = script_engine
                .eval::<bool>("new Point(0, 0).isOrigin()")
                .await
                .unwrap();
            assert!(result);
        });
    }

    #[test]
    fn test_clone() {
        Runtime::test_with_script_engine(async |mut script_engine| {
            setup(&mut script_engine).await;

            script_engine
                .eval::<()>("let pc = p1.clone()")
                .await
                .unwrap();

            let result = script_engine.eval::<bool>("pc.equals(p1)").await.unwrap();
            assert!(result);

            let result = script_engine.eval::<bool>("pc == p1").await.unwrap();
            assert!(!result);
        });
    }

    #[test]
    fn test_random() {
        Runtime::test_with_script_engine(async |mut script_engine| {
            setup(&mut script_engine).await;

            script_engine
                .eval::<JsSize>("Point.random()")
                .await
                .unwrap();
        });
    }
}
