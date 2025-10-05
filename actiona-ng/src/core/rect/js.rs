use rquickjs::{
    Ctx, Exception, JsLifetime, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::{FromParam, ParamRequirement, ParamsAccessor},
};

use super::rect;
use crate::core::{ResultExt, js::classes::ValueClass, point::js::JsPoint};

pub struct JsRectParam(pub super::Rect);

impl<'js> FromParam<'js> for JsRectParam {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::exhaustive()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        Ok(Self(match params.len() {
            n if n >= 4 => super::Rect::new(
                params.arg().get()?,
                params.arg().get()?,
                params.arg().get()?,
                params.arg().get()?,
            ),
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
                    object.get("x")?,
                    object.get("y")?,
                    object.get("width")?,
                    object.get("height")?,
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
    /// Constructor with an object.
    /// @param o: {x: number, y: number, width: number, height: number}
    ///
    /// @overload
    /// Constructor with another Rect.
    /// @param r: Rect
    #[qjs(constructor)]
    pub fn new(_ctx: Ctx<'_>, x: i32, y: i32, width: u32, height: u32) -> Result<Self> {
        // TODO: accept an object as arg

        Ok(Self {
            inner: rect(x, y, width, height),
        })
    }

    /// @skip
    #[qjs(get, rename = "x")]
    pub const fn get_x(&self) -> i32 {
        self.inner.x
    }

    /// @skip
    #[qjs(set, rename = "x")]
    pub const fn set_x(&mut self, x: i32) {
        self.inner.x = x;
    }

    /// @skip
    #[qjs(get, rename = "y")]
    pub const fn get_y(&self) -> i32 {
        self.inner.y
    }

    /// @skip
    #[qjs(set, rename = "y")]
    pub const fn set_y(&mut self, y: i32) {
        self.inner.y = y;
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

    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    pub fn contains(&self, point: JsPoint) -> bool {
        self.inner.contains(point.into())
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    pub fn to_string_js(&self) -> String {
        format!(
            "({}, {}, {}, {})",
            self.inner.x, self.inner.y, self.inner.width, self.inner.height
        )
    }

    #[qjs(rename = "clone")]
    pub const fn clone_js(&self) -> Self {
        *self
    }

    pub fn intersects(&self, other: Self) -> bool {
        self.inner.intersects(other.into())
    }

    pub fn intersection(&self, other: Self) -> Option<Self> {
        self.inner
            .intersection(other.into())
            .map(|result| result.into())
    }

    pub fn union(&self, other: Self) -> Self {
        self.inner.union(other.into()).into()
    }

    /// @skip
    #[qjs(skip)]
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
