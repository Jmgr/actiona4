//! @verbatim /**
//! @verbatim  * RectLike
//! @verbatim  */
//! @verbatim type RectLike = Rect | { x: number; y: number; width: number; height: number };

use macros::{js_class, js_methods};
use rquickjs::{
    Ctx, JsLifetime, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::{FromParam, ParamRequirement, ParamsAccessor},
};

use super::rect;
use crate::{
    IntoJsResult,
    api::{
        ResultExt,
        js::classes::ValueClass,
        point::{js::JsPoint, point, try_point},
        size::{js::JsSize, size, try_size},
    },
    types::display::display_with_type,
};
pub struct JsRectLike(pub super::Rect);

impl<'js> FromParam<'js> for JsRectLike {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::single() // 1 -> 1
            .combine(ParamRequirement::optional()) // 1 -> 2
            .combine(ParamRequirement::optional()) // 1 -> 3
            .combine(ParamRequirement::optional()) // 1 -> 4
            .combine(ParamRequirement::exhaustive())
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        let value = params.arg();

        if let Some(x) = value.as_number() {
            if params.len() < 3 {
                return Err(rquickjs::Error::new_from_js_message(
                    "number",
                    "Rect",
                    "Expected (x, y, width, height), got too few arguments",
                ));
            }
            let y = params
                .arg()
                .as_number()
                .or_throw_message(params.ctx(), "Expected y as a number")?;
            let width = params
                .arg()
                .as_number()
                .or_throw_message(params.ctx(), "Expected width as a number")?;
            let height = params
                .arg()
                .as_number()
                .or_throw_message(params.ctx(), "Expected height as a number")?;

            let origin = try_point(x, y).into_js_result(params.ctx())?;
            let size = try_size(width, height).into_js_result(params.ctx())?;
            return Ok(Self(super::Rect::new(origin, size)));
        }

        // Also accept a js::Rect as a parameter
        if let Ok(js_rect) = value.get::<JsRect>() {
            return Ok(Self(js_rect.into()));
        }

        let object = value
            .as_object()
            .or_throw_message(params.ctx(), "Expected an object")?;

        let x: f64 = object.get("x")?;
        let y: f64 = object.get("y")?;
        let width: f64 = object.get("width")?;
        let height: f64 = object.get("height")?;

        let origin = try_point(x, y).into_js_result(params.ctx())?;
        let size = try_size(width, height).into_js_result(params.ctx())?;
        Ok(Self(super::Rect::new(origin, size)))
    }
}

// TODO: property for point + size
/// A 2D rectangle with position and size.
///
/// Rects can be constructed from four numbers, an object with `x`/`y`/`width`/`height`, or another Rect.
///
/// ```ts
/// const r1 = new Rect(0, 0, 100, 50);
/// const r2 = new Rect({ x: 0, y: 0, width: 100, height: 50 });
/// ```
///
/// ```ts
/// const a = new Rect(0, 0, 100, 100);
/// const b = new Rect(50, 50, 100, 100);
/// println(a.intersects(b)); // true
/// const inter = a.intersection(b); // Rect(50, 50, 50, 50)
/// ```
///
/// @prop x: number // X coordinate
/// @prop y: number // Y coordinate
/// @prop width: number // Width
/// @prop height: number // Height
/// @prop topLeft: Point // Top-left origin
/// @prop size: Size // Size
#[derive(Clone, Copy, Debug, Eq, JsLifetime, PartialEq)]
#[js_class]
pub struct JsRect {
    inner: super::Rect,
}

impl ValueClass<'_> for JsRect {}

#[js_methods]
impl JsRect {
    /// Creates a new rectangle.
    ///
    /// Example
    /// ```js
    /// let rect = new Rect(0, 0, 32, 32);
    /// ```
    ///
    /// @constructor
    ///
    /// @overload
    /// Constructor with a position and a size.
    /// @param x: number
    /// @param y: number
    /// @param width: number
    /// @param height: number
    ///
    /// @overload
    /// @constructorOnly
    /// Constructor with anything Rect-like.
    /// @param r: RectLike
    #[qjs(constructor)]
    pub fn new(_ctx: Ctx<'_>, x: i32, y: i32, width: u32, height: u32) -> Result<Self> {
        // TODO: accept an object as arg

        Ok(Self {
            inner: rect(point(x, y), size(width, height)),
        })
    }

    /// @skip
    #[get("x")]
    #[must_use]
    pub fn get_x(&self) -> i32 {
        self.inner.top_left.x.into()
    }

    /// @skip
    #[set("x")]
    pub fn set_x(&mut self, x: i32) {
        self.inner.top_left.x = x.into();
    }

    /// @skip
    #[get("y")]
    #[must_use]
    pub fn get_y(&self) -> i32 {
        self.inner.top_left.y.into()
    }

    /// @skip
    #[set("y")]
    pub fn set_y(&mut self, y: i32) {
        self.inner.top_left.y = y.into();
    }

    /// @skip
    #[get("width")]
    #[must_use]
    pub fn get_width(&self) -> u32 {
        self.inner.size.width.into()
    }

    /// @skip
    #[set("width")]
    pub fn set_width(&mut self, width: u32) {
        self.inner.size.width = width.into();
    }

    /// @skip
    #[get("height")]
    #[must_use]
    pub fn get_height(&self) -> u32 {
        self.inner.size.height.into()
    }

    /// @skip
    #[set("height")]
    pub fn set_height(&mut self, height: u32) {
        self.inner.size.height = height.into();
    }

    /// @skip
    #[get("topLeft")]
    #[must_use]
    pub fn get_top_left(&self) -> JsPoint {
        self.inner.top_left.into()
    }

    /// @skip
    #[set("topLeft")]
    pub fn set_top_left(&mut self, top_left: JsPoint) {
        self.inner.top_left = top_left.into();
    }

    /// @skip
    #[get("size")]
    #[must_use]
    pub fn get_size(&self) -> JsSize {
        self.inner.size.into()
    }

    /// @skip
    #[set("size")]
    pub fn set_size(&mut self, size: JsSize) {
        self.inner.size = size.into();
    }

    /// Returns true if this Rect equals another.
    ///
    /// ```ts
    /// const a = new Rect(0, 0, 10, 10);
    /// const b = new Rect(0, 0, 10, 10);
    /// println(a.equals(b)); // true
    /// ```
    #[must_use]
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    /// Returns true if this Rect contains the given point.
    ///
    /// ```ts
    /// const r = new Rect(0, 0, 100, 100);
    /// println(r.contains(new Point(50, 50)));  // true
    /// println(r.contains(new Point(150, 50))); // false
    /// ```
    #[must_use]
    pub fn contains(&self, point: JsPoint) -> bool {
        self.inner.contains(point.into())
    }

    /// Returns a string representation of this Rect.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Rect", self.inner)
    }

    /// Clones this Rect.
    ///
    /// ```ts
    /// const original = new Rect(0, 0, 100, 100);
    /// const copy = original.clone();
    /// ```
    #[qjs(rename = "clone")]
    #[must_use]
    pub const fn clone_js(&self) -> Self {
        *self
    }

    /// Returns true if this Rect intersects with another.
    ///
    /// ```ts
    /// const a = new Rect(0, 0, 100, 100);
    /// const b = new Rect(50, 50, 100, 100);
    /// println(a.intersects(b)); // true
    /// ```
    #[must_use]
    pub fn intersects(&self, other: Self) -> bool {
        self.inner.intersects(other.into())
    }

    /// Returns the intersection of two Rects, or undefined if they don't overlap.
    ///
    /// ```ts
    /// const a = new Rect(0, 0, 100, 100);
    /// const b = new Rect(50, 50, 100, 100);
    /// const inter = a.intersection(b); // Rect(50, 50, 50, 50)
    /// ```
    #[must_use]
    pub fn intersection(&self, other: Self) -> Option<Self> {
        self.inner
            .intersection(other.into())
            .map(|result| result.into())
    }

    /// Returns the smallest Rect containing both this and another Rect.
    ///
    /// ```ts
    /// const a = new Rect(0, 0, 50, 50);
    /// const b = new Rect(25, 25, 50, 50);
    /// const u = a.union(b); // Rect(0, 0, 75, 75)
    /// ```
    #[must_use]
    pub fn union(&self, other: Self) -> Self {
        self.inner.union(other.into()).into()
    }

    /// @skip
    #[qjs(skip)]
    #[must_use]
    pub const fn inner(&self) -> super::Rect {
        self.inner
    }
}

impl<'js> Trace<'js> for JsRect {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl From<JsRect> for super::Rect {
    fn from(value: JsRect) -> Self {
        value.inner
    }
}

impl From<super::Rect> for JsRect {
    fn from(value: super::Rect) -> Self {
        Self { inner: value }
    }
}
