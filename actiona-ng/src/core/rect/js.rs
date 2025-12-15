//! @verbatim /**
//! @verbatim  * RectLike
//! @verbatim  */
//! @verbatim type RectLike = Rect | { x: number; y: number; width: number; height: number };

use rquickjs::{
    Ctx, Exception, FromJs, JsLifetime, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::{FromParam, ParamRequirement, ParamsAccessor},
};

use super::rect;
use crate::{
    core::{
        ResultExt,
        js::classes::ValueClass,
        point::{js::JsPoint, point},
        size::{js::JsSize, size},
    },
    types::{si32::Si32, su32::Su32},
};

pub struct JsRectLike(pub super::Rect);

impl<'js> FromParam<'js> for JsRectLike {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::single()
            .combine(ParamRequirement::optional())
            .combine(ParamRequirement::optional())
            .combine(ParamRequirement::optional())
            .combine(ParamRequirement::exhaustive())
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        Ok(Self(match params.len() {
            n if n >= 4 => {
                let param = params.arg();
                let x = Si32::from_js(params.ctx(), param)?;

                let param = params.arg();
                let y = Si32::from_js(params.ctx(), param)?;

                let param = params.arg();
                let w = Su32::from_js(params.ctx(), param)?;

                let param = params.arg();
                let h = Su32::from_js(params.ctx(), param)?;

                super::Rect::new(point(x, y), size(w, h))
            }
            n if n >= 1 => {
                let value = params.arg();

                // Also accept a js::Color as a parameter
                if let Ok(js_rect) = value.get::<JsRect>() {
                    return Ok(Self(js_rect.into()));
                }

                let object = value
                    .as_object()
                    .or_throw_message(params.ctx(), "Expected an object")?;

                super::Rect::new(
                    point(
                        Si32::from_js(params.ctx(), object.get("x")?)?,
                        Si32::from_js(params.ctx(), object.get("y")?)?,
                    ),
                    size(
                        Su32::from_js(params.ctx(), object.get("width")?)?,
                        Su32::from_js(params.ctx(), object.get("height")?)?,
                    ),
                )
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

// TODO: property for point + size
/// A 2D Rectangle.
///
/// @prop x: number // X coordinate
/// @prop y: number // Y coordinate
/// @prop width: number // Width
/// @prop height: number // Height
/// @prop topLeft: Point // Top-left origin
/// @prop size: Size // Size
///
/// ```js
/// let r = new Rect(1, 2, 50, 100);
/// ```
#[derive(Clone, Copy, Debug, Eq, JsLifetime, PartialEq)]
#[rquickjs::class(rename = "Rect")]
pub struct JsRect {
    inner: super::Rect,
}

impl ValueClass<'_> for JsRect {}

#[rquickjs::methods(rename_all = "camelCase")]
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
    #[qjs(get, rename = "x")]
    #[must_use]
    pub fn get_x(&self) -> i32 {
        self.inner.top_left.x.into()
    }

    /// @skip
    #[qjs(set, rename = "x")]
    pub fn set_x(&mut self, x: i32) {
        self.inner.top_left.x = x.into();
    }

    /// @skip
    #[qjs(get, rename = "y")]
    #[must_use]
    pub fn get_y(&self) -> i32 {
        self.inner.top_left.y.into()
    }

    /// @skip
    #[qjs(set, rename = "y")]
    pub fn set_y(&mut self, y: i32) {
        self.inner.top_left.y = y.into();
    }

    /// @skip
    #[qjs(get, rename = "width")]
    #[must_use]
    pub fn get_width(&self) -> u32 {
        self.inner.size.width.into()
    }

    /// @skip
    #[qjs(set, rename = "width")]
    pub fn set_width(&mut self, width: u32) {
        self.inner.size.width = width.into();
    }

    /// @skip
    #[qjs(get, rename = "height")]
    #[must_use]
    pub fn get_height(&self) -> u32 {
        self.inner.size.height.into()
    }

    /// @skip
    #[qjs(set, rename = "height")]
    pub fn set_height(&mut self, height: u32) {
        self.inner.size.height = height.into();
    }

    /// @skip
    #[qjs(get, rename = "topLeft")]
    #[must_use]
    pub fn get_top_left(&self) -> JsPoint {
        self.inner.top_left.into()
    }

    /// @skip
    #[qjs(set, rename = "topLeft")]
    pub fn set_top_left(&mut self, top_left: JsPoint) {
        self.inner.top_left = top_left.into();
    }

    /// @skip
    #[qjs(get, rename = "size")]
    #[must_use]
    pub fn get_size(&self) -> JsSize {
        self.inner.size.into()
    }

    /// @skip
    #[qjs(set, rename = "size")]
    pub fn set_size(&mut self, size: JsSize) {
        self.inner.size = size.into();
    }

    #[must_use]
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    #[must_use]
    pub fn contains(&self, point: JsPoint) -> bool {
        self.inner.contains(point.into())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        format!(
            "({}, {}, {}, {})",
            self.inner.top_left.x,
            self.inner.top_left.y,
            self.inner.size.width,
            self.inner.size.height
        )
    }

    #[qjs(rename = "clone")]
    #[must_use]
    pub const fn clone_js(&self) -> Self {
        *self
    }

    #[must_use]
    pub fn intersects(&self, other: Self) -> bool {
        self.inner.intersects(other.into())
    }

    #[must_use]
    pub fn intersection(&self, other: Self) -> Option<Self> {
        self.inner
            .intersection(other.into())
            .map(|result| result.into())
    }

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
