//! @verbatim /**
//! @verbatim  * A size as a {@link Size} instance or a plain `{width, height}` object.
//! @verbatim  *
//! @verbatim  * ```ts
//! @verbatim  * image.resize(new Size(800, 600));           // Size instance
//! @verbatim  * image.resize({ width: 800, height: 600 }); // plain object
//! @verbatim  * ```
//! @verbatim  */
//! @verbatim type SizeLike = Size | { width: number; height: number };

use macros::{js_class, js_methods};
use rquickjs::{
    Ctx, JsLifetime, Result, Value,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::{FromParam, ParamRequirement, ParamsAccessor},
};
use types::{
    display::display_with_type,
    size::{Size, size},
};

use crate::{
    IntoJsResult,
    api::{
        ResultExt,
        js::{FromJsField, classes::ValueClass, has_registered_class_prototype},
    },
};

#[derive(Clone, Copy, Debug)]
pub struct JsSizeLike(pub Size);

fn size_from_value<'js>(ctx: &Ctx<'js>, value: &Value<'js>) -> Result<Size> {
    if let Some(object) = value.as_object() {
        if has_registered_class_prototype::<JsSize>(ctx, object)? {
            return value.clone().get::<JsSize>().map(Into::into);
        }

        let width: f64 = object.get("width")?;
        let height: f64 = object.get("height")?;
        return Ok(size(width, height));
    }

    Err(rquickjs::Error::new_from_js_message(
        value.type_name(),
        "Size",
        "Expected a Size or object with width/height",
    ))
}

impl<'js> FromJsField<'js> for JsSizeLike {
    fn from_js_field(ctx: &Ctx<'js>, value: Value<'js>) -> Result<Self> {
        size_from_value(ctx, &value).map(Self)
    }
}

impl<'js> FromParam<'js> for JsSizeLike {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::single() // 1 -> 1
            .combine(ParamRequirement::optional()) // 1 -> 2
            .combine(ParamRequirement::exhaustive())
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        let value = params.arg();

        if let Some(width) = value.as_number() {
            if params.is_empty() {
                return Err(rquickjs::Error::new_from_js_message(
                    "number",
                    "Size",
                    "Expected (width, height), got a single number",
                ));
            }
            let height = params
                .arg()
                .as_number()
                .or_throw_message(params.ctx(), "Expected height as a number")?;

            return Ok(Self(size(width, height)));
        }

        size_from_value(params.ctx(), &value).map(Self)
    }
}

/// A 2D size with width and height.
///
/// Sizes can be constructed from two numbers, an object with `width`/`height`, or another Size.
///
/// ```ts
/// const s1 = new Size(100, 50);
/// const s2 = new Size({ width: 100, height: 50 });
/// const s3 = new Size(s1);
/// ```
///
/// ```ts
/// const a = new Size(10, 20);
/// const b = new Size(5, 10);
/// println(a.add(b)); // "Size(15, 30)"
/// println(a.scale(2)); // "Size(20, 40)"
/// ```
///
/// @prop width: number // width
/// @prop height: number // height
#[derive(Clone, Copy, Debug, Eq, JsLifetime, PartialEq)]
#[js_class]
pub struct JsSize {
    inner: Size,
}

impl ValueClass<'_> for JsSize {}

#[js_methods]
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
    /// @constructorOnly
    /// Constructor with anything Size-like.
    /// @param s: SizeLike
    #[qjs(constructor)]
    #[must_use]
    pub fn new(size: JsSizeLike) -> Self {
        size.0.into()
    }

    /// @skip
    #[get("width")]
    #[must_use]
    pub fn get_width(&self) -> u32 {
        self.inner.width.into()
    }

    /// @skip
    #[set("width")]
    pub fn set_width(&mut self, width: u32) {
        self.inner.width = width.into();
    }

    /// @skip
    #[get("height")]
    #[must_use]
    pub fn get_height(&self) -> u32 {
        self.inner.height.into()
    }

    /// @skip
    #[set("height")]
    pub fn set_height(&mut self, height: u32) {
        self.inner.height = height.into();
    }

    /// Returns a JSON representation of this Size.
    ///
    /// ```ts
    /// const s = new Size(100, 50);
    /// println(s.toJson()); // '{"width":100,"height":50}'
    /// ```
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.inner).expect("Size should always serialize to JSON")
    }

    /// Returns true if a Size equals another.
    ///
    /// ```ts
    /// const a = new Size(10, 20);
    /// const b = new Size(10, 20);
    /// println(a.equals(b)); // true
    /// ```
    #[must_use]
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    /// Adds two sizes and returns a new Size.
    ///
    /// ```ts
    /// const sum = new Size(10, 20).add(new Size(5, 10));
    /// println(sum); // "Size(15, 30)"
    /// ```
    #[must_use]
    pub fn add(&self, other: Self) -> Self {
        (self.inner + other.inner).into()
    }

    /// Subtracts two sizes and returns a new Size.
    ///
    /// ```ts
    /// const diff = new Size(100, 50).subtract(new Size(30, 20));
    /// println(diff); // "Size(70, 30)"
    /// ```
    #[must_use]
    pub fn subtract(&self, other: Self) -> Self {
        (self.inner - other.inner).into()
    }

    /// Scales this size by a factor and returns a new Size.
    ///
    /// ```ts
    /// const s = new Size(10, 20).scale(3);
    /// println(s); // "Size(30, 60)"
    /// ```
    pub fn scale<'js>(&self, ctx: Ctx<'js>, factor: f64) -> Result<Self> {
        self.inner
            .scaled(factor)
            .map(|value| value.into())
            .into_js_result(&ctx)
    }

    /// Returns a string representation of this size.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        display_with_type("Size", self.inner)
    }

    /// Clones this Size.
    ///
    /// ```ts
    /// const original = new Size(100, 50);
    /// const copy = original.clone();
    /// ```
    #[qjs(rename = "clone")]
    #[must_use]
    pub const fn clone_js(&self) -> Self {
        *self
    }

    /// @skip
    #[must_use]
    #[qjs(skip)]
    pub const fn inner(&self) -> Size {
        self.inner
    }
}

impl<'js> Trace<'js> for JsSize {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl From<JsSize> for Size {
    fn from(value: JsSize) -> Self {
        value.inner
    }
}

impl From<Size> for JsSize {
    fn from(value: Size) -> Self {
        Self { inner: value }
    }
}
