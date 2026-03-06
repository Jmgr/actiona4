//! @verbatim /**
//! @verbatim  * PointLike
//! @verbatim  */
//! @verbatim type PointLike = Point | { x: number; y: number } | Match;

use macros::{js_class, js_methods};
use rquickjs::{
    Ctx, JsLifetime, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::{FromParam, ParamRequirement, ParamsAccessor},
};

use crate::{
    IntoJsResult,
    api::{ResultExt, image::js::JsMatch, js::classes::ValueClass, point::try_point},
    runtime::WithUserData,
    types::display::display_with_type,
};
pub struct JsPointLike(pub super::Point);

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

        // Also accept a JsPoint as a parameter.
        if let Ok(js_point) = value.get::<JsPoint>() {
            return Ok(Self(js_point.into()));
        }

        if let Ok(_match) = value.get::<JsMatch>() {
            return Ok(Self(_match.position().into()));
        }

        let object = value
            .as_object()
            .or_throw_message(params.ctx(), "Expected an object")?;

        let x: f64 = object.get("x")?;
        let y: f64 = object.get("y")?;
        let point = try_point(x, y).into_js_result(params.ctx())?;
        Ok(Self(point))
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
#[derive(Clone, Copy, Debug, Eq, JsLifetime, PartialEq)]
#[js_class]
pub struct JsPoint {
    inner: super::Point,
}

impl ValueClass<'_> for JsPoint {}

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

    /// Returns a string representation of this Point.
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
    use super::JsPoint;
    use crate::{api::point::point, runtime::Runtime, scripting::Engine as ScriptEngine};

    async fn setup(script_engine: ScriptEngine) {
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
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

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
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

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
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

            let result = script_engine
                .eval::<JsPoint>("p1.add(new Point(1, 3))")
                .await
                .unwrap();
            assert_eq!(result, point(2, 5).into());

            let result = script_engine
                .eval::<JsPoint>("p1.subtract(new Point(1, 3))")
                .await
                .unwrap();
            assert_eq!(result, point(0, -1).into());

            let result = script_engine.eval::<JsPoint>("p1.scaled(2)").await.unwrap();
            assert_eq!(result, point(2, 4).into());
        });
    }

    #[test]
    fn test_distance() {
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

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
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

            let result = script_engine.eval::<String>("p1.toJson()").await.unwrap();
            assert_eq!(result, r#"{"x":1,"y":2}"#);
        });
    }

    #[test]
    fn test_origin() {
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

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
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

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
        Runtime::test_with_script_engine(async |script_engine| {
            setup(script_engine.clone()).await;

            script_engine
                .eval::<JsPoint>("Point.randomInCircle(0, 0, 10)")
                .await
                .unwrap();
        });
    }
}
