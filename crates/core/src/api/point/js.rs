//! @verbatim /**
//! @verbatim  * A point as a {@link Point} instance, a plain `{x, y}` object, or a {@link Match}.
//! @verbatim  *
//! @verbatim  * ```ts
//! @verbatim  * mouse.move(new Point(100, 200)); // Point instance
//! @verbatim  * mouse.move({ x: 100, y: 200 });  // plain object
//! @verbatim  * mouse.move(match);               // Match from findImage
//! @verbatim  * ```
//! @verbatim  */
//! @verbatim type PointLike = Point | { x: number; y: number } | Match;

use macros::{js_class, js_methods};
use rquickjs::{
    Ctx, JsLifetime, Object, Result, Value,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::{FromParam, ParamRequirement, ParamsAccessor},
};

use crate::{
    IntoJsResult,
    api::{
        ResultExt,
        image::js::JsMatch,
        js::{FromJsField, classes::ValueClass, has_registered_class_prototype},
        point::try_point,
    },
    runtime::WithUserData,
    types::display::display_with_type,
};

#[derive(Clone, Copy, Debug)]
pub struct JsPointLike(pub super::Point);

fn point_from_value<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> Result<super::Point> {
    if let Some(object) = value.as_object() {
        if has_registered_class_prototype::<JsPoint>(ctx, object)? {
            return value.clone().get::<JsPoint>().map(Into::into);
        }

        if has_registered_class_prototype::<JsMatch>(ctx, object)? {
            return value
                .clone()
                .get::<JsMatch>()
                .map(|matched_point| matched_point.position().into());
        }

        let x: f64 = object.get("x")?;
        let y: f64 = object.get("y")?;
        return try_point(x, y).into_js_result(ctx);
    }

    Err(rquickjs::Error::new_from_js_message(
        value.type_name(),
        "Point",
        "Expected a Point, Match, or object with x/y",
    ))
}

impl<'js> FromJsField<'js> for JsPointLike {
    fn from_js_field(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        point_from_value(ctx, &value).map(Self)
    }
}

impl From<super::Point> for JsPointLike {
    fn from(value: super::Point) -> Self {
        Self(value)
    }
}

impl<'js> FromParam<'js> for JsPointLike {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::single() // 1 -> 1
            .combine(ParamRequirement::optional()) // 1 -> 2
            .combine(ParamRequirement::exhaustive())
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        // Prefer explicit numeric overloads when the first argument is a number.
        // Otherwise accept a Point instance or an object with x/y.
        let value = params.arg();
        if let Some(x) = value.as_number() {
            if params.is_empty() {
                return Err(rquickjs::Error::new_from_js_message(
                    "number",
                    "Point",
                    "Expected (x, y) coordinates, got a single number",
                ));
            }
            let second_arg = params.arg();
            let y = second_arg
                .as_number()
                .or_throw_message(params.ctx(), "Expected second argument to be a number")?;

            let point = try_point(x, y).into_js_result(params.ctx())?;
            return Ok(Self(point));
        }

        point_from_value(params.ctx(), &value).map(Self)
    }
}

/// A 2D point with integer coordinates.
///
/// Points can be constructed from two numbers, an object with `x`/`y`, or another Point.
///
/// ```ts
/// const p1 = new Point(10, 20);
/// const p2 = new Point({ x: 10, y: 20 });
/// const p3 = new Point(p1);
/// ```
///
/// ```ts
/// const a = new Point(1, 2);
/// const b = new Point(4, 6);
/// println(a.distanceTo(b)); // 5
/// println(a.add(b)); // "Point(5, 8)"
/// ```
///
/// @prop x: number // X coordinate
/// @prop y: number // Y coordinate
///
/// @const Zero // Point(0, 0)
#[derive(Clone, Copy, Debug, Eq, JsLifetime, PartialEq)]
#[js_class]
pub struct JsPoint {
    inner: super::Point,
}

impl<'js> ValueClass<'js> for JsPoint {
    fn extra_registration(object: &Object<'js>) -> rquickjs::Result<()> {
        object.prop("Zero", Self::from(super::Point::default()))?;
        Ok(())
    }
}

#[js_methods]
impl JsPoint {
    /// Constructor.
    ///
    /// @constructor
    ///
    /// @overload
    /// Constructor with two numbers.
    /// @param x: number // X coordinate
    /// @param y: number // Y coordinate
    ///
    /// @overload
    /// @constructorOnly
    /// Constructor with anything Point-like.
    /// @param p: PointLike
    #[qjs(constructor)]
    #[must_use]
    pub fn new(point: JsPointLike) -> Self {
        point.0.into()
    }

    /// @skip
    #[get("x")]
    #[must_use]
    pub fn get_x(&self) -> i32 {
        self.inner.x.into()
    }

    /// @skip
    #[set("x")]
    pub fn set_x(&mut self, x: i32) {
        self.inner.x = x.into();
    }

    /// @skip
    #[get("y")]
    #[must_use]
    pub fn get_y(&self) -> i32 {
        self.inner.y.into()
    }

    /// @skip
    #[set("y")]
    pub fn set_y(&mut self, y: i32) {
        self.inner.y = y.into();
    }

    /// Length of this point (distance from origin).
    ///
    /// ```ts
    /// const p = new Point(3, 4);
    /// println(p.length()); // 5
    /// ```
    #[must_use]
    pub fn length(&self) -> f64 {
        self.inner.length()
    }

    /// Returns a random point within a circle of the given radius around a center point.
    ///
    /// ```ts
    /// const p = Point.randomInCircle(100, 100, 50);
    /// ```
    #[qjs(static)]
    pub fn random_in_circle(ctx: Ctx<'_>, center: JsPointLike, radius: f64) -> Result<Self> {
        let user_data = ctx.user_data();

        let point = super::Point::random_in_circle(center.0, radius, user_data.rng())
            .into_js_result(&ctx)?;
        Ok(point.into())
    }

    /// Calculates the distance between this point and another.
    ///
    /// ```ts
    /// const a = new Point(0, 0);
    /// const b = new Point(3, 4);
    /// println(a.distanceTo(b)); // 5
    /// ```
    #[must_use]
    pub fn distance_to(&self, other: Self) -> f64 {
        self.inner.distance_to(other.into())
    }

    /// Returns a JSON representation of this Point.
    ///
    /// ```ts
    /// const p = new Point(1, 2);
    /// println(p.toJson()); // '{"x":1,"y":2}'
    /// ```
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("Point should always serialize to JSON")
    }

    /// Returns true if this Point is at the origin, (0, 0).
    ///
    /// ```ts
    /// println(new Point(0, 0).isOrigin()); // true
    /// println(new Point(1, 0).isOrigin()); // false
    /// ```
    #[must_use]
    pub fn is_origin(&self) -> bool {
        self.inner.is_origin()
    }

    /// Computes the distance between two points.
    ///
    /// ```ts
    /// const d = Point.distance(new Point(0, 0), new Point(3, 4));
    /// println(d); // 5
    /// ```
    #[qjs(static)]
    #[must_use]
    pub fn distance(a: Self, b: Self) -> f64 {
        a.distance_to(b)
    }

    /// Returns true if a Point equals another.
    ///
    /// ```ts
    /// const a = new Point(1, 2);
    /// const b = new Point(1, 2);
    /// println(a.equals(b)); // true
    /// ```
    #[must_use]
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    /// Adds two points and returns a new Point.
    ///
    /// ```ts
    /// const sum = new Point(1, 2).add(new Point(3, 4));
    /// println(sum); // "Point(4, 6)"
    /// ```
    #[must_use]
    pub fn add(&self, other: Self) -> Self {
        (self.inner + other.inner).into()
    }

    /// Subtracts two points and returns a new Point.
    ///
    /// ```ts
    /// const diff = new Point(5, 7).subtract(new Point(2, 3));
    /// println(diff); // "Point(3, 4)"
    /// ```
    #[must_use]
    pub fn subtract(&self, other: Self) -> Self {
        (self.inner - other.inner).into()
    }

    /// Scales this point by a factor and returns a new Point.
    ///
    /// ```ts
    /// const p = new Point(3, 4).scaled(2);
    /// println(p); // "Point(6, 8)"
    /// ```
    pub fn scaled<'js>(&self, ctx: Ctx<'js>, factor: f64) -> Result<Self> {
        self.inner
            .scaled(factor)
            .map(Into::into)
            .into_js_result(&ctx)
    }

    /// Returns a string representation of this point.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Point", self.inner)
    }

    /// Clones this Point.
    ///
    /// ```ts
    /// const original = new Point(1, 2);
    /// const copy = original.clone();
    /// ```
    #[qjs(rename = "clone")]
    #[must_use]
    pub const fn clone_js(&self) -> Self {
        *self
    }

    /// @skip
    #[qjs(skip)]
    #[must_use]
    pub const fn inner(&self) -> super::Point {
        self.inner
    }
}

impl<'js> Trace<'js> for JsPoint {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl From<JsPoint> for super::Point {
    fn from(value: JsPoint) -> Self {
        value.inner
    }
}

impl From<super::Point> for JsPoint {
    fn from(value: super::Point) -> Self {
        Self { inner: value }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{api::test_helpers::js_path, runtime::Runtime};

    fn test_data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-data")
    }

    #[test]
    fn test_point_like_accepts_match() {
        Runtime::test_with_script_engine(async |script_engine| {
            let test_data_dir = test_data_dir();
            let source_path = test_data_dir.join("input.png");
            let template_path = test_data_dir.join("Crown_icon_transparent.png");

            let result = script_engine
                .eval_async::<bool>(&format!(
                    r#"
                    const source = await Image.load({});
                    const template = await Image.load({});
                    const match = await source.find(template, {{ useColors: true }});
                    if (!match) {{
                        throw new Error("Expected a match but got undefined");
                    }}
                    new Point(match).equals(match.position)
                    "#,
                    js_path(&source_path),
                    js_path(&template_path)
                ))
                .await
                .unwrap();

            assert!(result);
        });
    }
}
