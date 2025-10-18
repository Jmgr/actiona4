use image::Rgba;
use rquickjs::{
    Ctx, Exception, JsLifetime, Object, Result, Value,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::{FromParam, ParamRequirement, ParamsAccessor},
    prelude::*,
};

use crate::core::{ResultExt, js::classes::ValueClass};

pub struct JsColorParam(pub super::Color);

impl<'js> FromParam<'js> for JsColorParam {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::exhaustive()
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        Ok(Self(match params.len() {
            n if n >= 4 => super::Color::new(
                params.arg().get()?,
                params.arg().get()?,
                params.arg().get()?,
                params.arg().get()?,
            ),
            n if n >= 3 => super::Color::new(
                params.arg().get()?,
                params.arg().get()?,
                params.arg().get()?,
                255,
            ),
            n if n >= 1 => {
                let value = params.arg();

                // Also accept a js::Color as a parameter
                if let Ok(js_color) = value.get::<JsColor>() {
                    return Ok(Self(js_color.into()));
                }

                let object = value
                    .as_object()
                    .or_throw_message(params.ctx(), "Expected an object")?;

                let a = object.get("a").unwrap_or(255);

                super::Color::new(object.get("r")?, object.get("g")?, object.get("b")?, a)
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

/// A Color.
///
/// @prop r: number // Red (should be between 0-255)
/// @prop g: number // Green (should be between 0-255)
/// @prop b: number // Blue (should be between 0-255)
/// @prop a: number // Alpha (should be between 0-255)
///
/// @const RED
/// @const GREEN
/// @const BLUE
/// @const WHITE
/// @const BLACK
/// @const TRANSPARENT
///
/// @const ALICE_BLUE
/// @const ANTIQUE_WHITE
/// @const AQUA
/// @const AQUAMARINE
/// @const AZURE
/// @const BEIGE
/// @const BISQUE
/// @const BLANCHED_ALMOND
/// @const BLUE_VIOLET
/// @const BROWN
/// @const BURLY_WOOD
/// @const CADET_BLUE
/// @const CHARTREUSE
/// @const CHOCOLATE
/// @const CORAL
/// @const CORNFLOWER_BLUE
/// @const CORNSILK
/// @const CRIMSON
/// @const CYAN
/// @const DARK_BLUE
/// @const DARK_CYAN
/// @const DARK_GOLDEN_ROD
/// @const DARK_GRAY
/// @const DARK_GREEN
/// @const DARK_KHAKI
/// @const DARK_MAGENTA
/// @const DARK_OLIVE_GREEN
/// @const DARK_ORANGE
/// @const DARK_ORCHID
/// @const DARK_RED
/// @const DARK_SALMON
/// @const DARK_SEA_GREEN
/// @const DARK_SLATE_BLUE
/// @const DARK_SLATE_GRAY
/// @const DARK_TURQUOISE
/// @const DARK_VIOLET
/// @const DEEP_PINK
/// @const DEEP_SKY_BLUE
/// @const DIM_GRAY
/// @const DODGER_BLUE
/// @const FIREBRICK
/// @const FLORAL_WHITE
/// @const FOREST_GREEN
/// @const FUCHSIA
/// @const GAINSBORO
/// @const GHOST_WHITE
/// @const GOLD
/// @const GOLDEN_ROD
/// @const GRAY
/// @const GREEN_YELLOW
/// @const HONEY_DEW
/// @const HOT_PINK
/// @const INDIAN_RED
/// @const INDIGO
/// @const IVORY
/// @const KHAKI
/// @const LAVENDER
/// @const LAVENDER_BLUSH
/// @const LAWN_GREEN
/// @const LEMON_CHIFFON
/// @const LIGHT_BLUE
/// @const LIGHT_CORAL
/// @const LIGHT_CYAN
/// @const LIGHT_GOLDEN_ROD_YELLOW
/// @const LIGHT_GRAY
/// @const LIGHT_GREEN
/// @const LIGHT_PINK
/// @const LIGHT_SALMON
/// @const LIGHT_SEA_GREEN
/// @const LIGHT_SKY_BLUE
/// @const LIGHT_SLATE_GRAY
/// @const LIGHT_STEEL_BLUE
/// @const LIGHT_YELLOW
/// @const LIME
/// @const LIME_GREEN
/// @const LINEN
/// @const MAGENTA
/// @const MAROON
/// @const MEDIUM_AQUA_MARINE
/// @const MEDIUM_BLUE
/// @const MEDIUM_ORCHID
/// @const MEDIUM_PURPLE
/// @const MEDIUM_SEA_GREEN
/// @const MEDIUM_SLATE_BLUE
/// @const MEDIUM_SPRING_GREEN
/// @const MEDIUM_TURQUOISE
/// @const MEDIUM_VIOLET_RED
/// @const MIDNIGHT_BLUE
/// @const MINT_CREAM
/// @const MISTY_ROSE
/// @const MOCCASIN
/// @const NAVAJO_WHITE
/// @const NAVY
/// @const OLD_LACE
/// @const OLIVE
/// @const OLIVE_DRAB
/// @const ORANGE
/// @const ORANGE_RED
/// @const ORCHID
/// @const PALE_GOLDEN_ROD
/// @const PALE_GREEN
/// @const PALE_TURQUOISE
/// @const PALE_VIOLET_RED
/// @const PAPAYA_WHIP
/// @const PEACH_PUFF
/// @const PERU
/// @const PINK
/// @const PLUM
/// @const POWDER_BLUE
/// @const PURPLE
/// @const REBECCA_PURPLE
/// @const ROSY_BROWN
/// @const ROYAL_BLUE
/// @const SADDLE_BROWN
/// @const SALMON
/// @const SANDY_BROWN
/// @const SEA_GREEN
/// @const SEA_SHELL
/// @const SIENNA
/// @const SILVER
/// @const SKY_BLUE
/// @const SLATE_BLUE
/// @const SLATE_GRAY
/// @const SNOW
/// @const SPRING_GREEN
/// @const STEEL_BLUE
/// @const TAN
/// @const TEAL
/// @const THISTLE
/// @const TOMATO
/// @const TURQUOISE
/// @const VIOLET
/// @const WHEAT
/// @const WHITE_SMOKE
/// @const YELLOW
/// @const YELLOW_GREEN
///
/// ```js
/// let c = new Color(128, 255, 255, 255);
/// ```
#[derive(Clone, Copy, Debug, Eq, JsLifetime, PartialEq)]
#[rquickjs::class(rename = "Color")]
pub struct JsColor {
    inner: super::Color,
}

impl<'js> ValueClass<'js> for JsColor {
    fn extra_registration(object: &Object<'js>) -> rquickjs::Result<()> {
        // Basic colors
        object.prop("RED", Self::new(255, 0, 0, 255))?;
        object.prop("GREEN", Self::new(0, 128, 0, 255))?; // CSS green is #008000
        object.prop("BLUE", Self::new(0, 0, 255, 255))?;
        object.prop("WHITE", Self::new(255, 255, 255, 255))?;
        object.prop("BLACK", Self::new(0, 0, 0, 255))?;
        object.prop("TRANSPARENT", Self::new(0, 0, 0, 0))?;

        // CSS colors
        object.prop("ALICE_BLUE", Self::new(240, 248, 255, 255))?;
        object.prop("ANTIQUE_WHITE", Self::new(250, 235, 215, 255))?;
        object.prop("AQUA", Self::new(0, 255, 255, 255))?;
        object.prop("AQUAMARINE", Self::new(127, 255, 212, 255))?;
        object.prop("AZURE", Self::new(240, 255, 255, 255))?;
        object.prop("BEIGE", Self::new(245, 245, 220, 255))?;
        object.prop("BISQUE", Self::new(255, 228, 196, 255))?;
        object.prop("BLANCHED_ALMOND", Self::new(255, 235, 205, 255))?;
        object.prop("BLUE_VIOLET", Self::new(138, 43, 226, 255))?;
        object.prop("BROWN", Self::new(165, 42, 42, 255))?;
        object.prop("BURLY_WOOD", Self::new(222, 184, 135, 255))?;
        object.prop("CADET_BLUE", Self::new(95, 158, 160, 255))?;
        object.prop("CHARTREUSE", Self::new(127, 255, 0, 255))?;
        object.prop("CHOCOLATE", Self::new(210, 105, 30, 255))?;
        object.prop("CORAL", Self::new(255, 127, 80, 255))?;
        object.prop("CORNFLOWER_BLUE", Self::new(100, 149, 237, 255))?;
        object.prop("CORNSILK", Self::new(255, 248, 220, 255))?;
        object.prop("CRIMSON", Self::new(220, 20, 60, 255))?;
        object.prop("CYAN", Self::new(0, 255, 255, 255))?;
        object.prop("DARK_BLUE", Self::new(0, 0, 139, 255))?;
        object.prop("DARK_CYAN", Self::new(0, 139, 139, 255))?;
        object.prop("DARK_GOLDEN_ROD", Self::new(184, 134, 11, 255))?;
        object.prop("DARK_GRAY", Self::new(169, 169, 169, 255))?;
        object.prop("DARK_GREEN", Self::new(0, 100, 0, 255))?;
        object.prop("DARK_KHAKI", Self::new(189, 183, 107, 255))?;
        object.prop("DARK_MAGENTA", Self::new(139, 0, 139, 255))?;
        object.prop("DARK_OLIVE_GREEN", Self::new(85, 107, 47, 255))?;
        object.prop("DARK_ORANGE", Self::new(255, 140, 0, 255))?;
        object.prop("DARK_ORCHID", Self::new(153, 50, 204, 255))?;
        object.prop("DARK_RED", Self::new(139, 0, 0, 255))?;
        object.prop("DARK_SALMON", Self::new(233, 150, 122, 255))?;
        object.prop("DARK_SEA_GREEN", Self::new(143, 188, 143, 255))?;
        object.prop("DARK_SLATE_BLUE", Self::new(72, 61, 139, 255))?;
        object.prop("DARK_SLATE_GRAY", Self::new(47, 79, 79, 255))?;
        object.prop("DARK_TURQUOISE", Self::new(0, 206, 209, 255))?;
        object.prop("DARK_VIOLET", Self::new(148, 0, 211, 255))?;
        object.prop("DEEP_PINK", Self::new(255, 20, 147, 255))?;
        object.prop("DEEP_SKY_BLUE", Self::new(0, 191, 255, 255))?;
        object.prop("DIM_GRAY", Self::new(105, 105, 105, 255))?;
        object.prop("DODGER_BLUE", Self::new(30, 144, 255, 255))?;
        object.prop("FIREBRICK", Self::new(178, 34, 34, 255))?;
        object.prop("FLORAL_WHITE", Self::new(255, 250, 240, 255))?;
        object.prop("FOREST_GREEN", Self::new(34, 139, 34, 255))?;
        object.prop("FUCHSIA", Self::new(255, 0, 255, 255))?;
        object.prop("GAINSBORO", Self::new(220, 220, 220, 255))?;
        object.prop("GHOST_WHITE", Self::new(248, 248, 255, 255))?;
        object.prop("GOLD", Self::new(255, 215, 0, 255))?;
        object.prop("GOLDEN_ROD", Self::new(218, 165, 32, 255))?;
        object.prop("GRAY", Self::new(128, 128, 128, 255))?;
        object.prop("GREEN_YELLOW", Self::new(173, 255, 47, 255))?;
        object.prop("HONEY_DEW", Self::new(240, 255, 240, 255))?;
        object.prop("HOT_PINK", Self::new(255, 105, 180, 255))?;
        object.prop("INDIAN_RED", Self::new(205, 92, 92, 255))?;
        object.prop("INDIGO", Self::new(75, 0, 130, 255))?;
        object.prop("IVORY", Self::new(255, 255, 240, 255))?;
        object.prop("KHAKI", Self::new(240, 230, 140, 255))?;
        object.prop("LAVENDER", Self::new(230, 230, 250, 255))?;
        object.prop("LAVENDER_BLUSH", Self::new(255, 240, 245, 255))?;
        object.prop("LAWN_GREEN", Self::new(124, 252, 0, 255))?;
        object.prop("LEMON_CHIFFON", Self::new(255, 250, 205, 255))?;
        object.prop("LIGHT_BLUE", Self::new(173, 216, 230, 255))?;
        object.prop("LIGHT_CORAL", Self::new(240, 128, 128, 255))?;
        object.prop("LIGHT_CYAN", Self::new(224, 255, 255, 255))?;
        object.prop("LIGHT_GOLDEN_ROD_YELLOW", Self::new(250, 250, 210, 255))?;
        object.prop("LIGHT_GRAY", Self::new(211, 211, 211, 255))?;
        object.prop("LIGHT_GREEN", Self::new(144, 238, 144, 255))?;
        object.prop("LIGHT_PINK", Self::new(255, 182, 193, 255))?;
        object.prop("LIGHT_SALMON", Self::new(255, 160, 122, 255))?;
        object.prop("LIGHT_SEA_GREEN", Self::new(32, 178, 170, 255))?;
        object.prop("LIGHT_SKY_BLUE", Self::new(135, 206, 250, 255))?;
        object.prop("LIGHT_SLATE_GRAY", Self::new(119, 136, 153, 255))?;
        object.prop("LIGHT_STEEL_BLUE", Self::new(176, 196, 222, 255))?;
        object.prop("LIGHT_YELLOW", Self::new(255, 255, 224, 255))?;
        object.prop("LIME", Self::new(0, 255, 0, 255))?;
        object.prop("LIME_GREEN", Self::new(50, 205, 50, 255))?;
        object.prop("LINEN", Self::new(250, 240, 230, 255))?;
        object.prop("MAGENTA", Self::new(255, 0, 255, 255))?;
        object.prop("MAROON", Self::new(128, 0, 0, 255))?;
        object.prop("MEDIUM_AQUA_MARINE", Self::new(102, 205, 170, 255))?;
        object.prop("MEDIUM_BLUE", Self::new(0, 0, 205, 255))?;
        object.prop("MEDIUM_ORCHID", Self::new(186, 85, 211, 255))?;
        object.prop("MEDIUM_PURPLE", Self::new(147, 112, 219, 255))?;
        object.prop("MEDIUM_SEA_GREEN", Self::new(60, 179, 113, 255))?;
        object.prop("MEDIUM_SLATE_BLUE", Self::new(123, 104, 238, 255))?;
        object.prop("MEDIUM_SPRING_GREEN", Self::new(0, 250, 154, 255))?;
        object.prop("MEDIUM_TURQUOISE", Self::new(72, 209, 204, 255))?;
        object.prop("MEDIUM_VIOLET_RED", Self::new(199, 21, 133, 255))?;
        object.prop("MIDNIGHT_BLUE", Self::new(25, 25, 112, 255))?;
        object.prop("MINT_CREAM", Self::new(245, 255, 250, 255))?;
        object.prop("MISTY_ROSE", Self::new(255, 228, 225, 255))?;
        object.prop("MOCCASIN", Self::new(255, 228, 181, 255))?;
        object.prop("NAVAJO_WHITE", Self::new(255, 222, 173, 255))?;
        object.prop("NAVY", Self::new(0, 0, 128, 255))?;
        object.prop("OLD_LACE", Self::new(253, 245, 230, 255))?;
        object.prop("OLIVE", Self::new(128, 128, 0, 255))?;
        object.prop("OLIVE_DRAB", Self::new(107, 142, 35, 255))?;
        object.prop("ORANGE", Self::new(255, 165, 0, 255))?;
        object.prop("ORANGE_RED", Self::new(255, 69, 0, 255))?;
        object.prop("ORCHID", Self::new(218, 112, 214, 255))?;
        object.prop("PALE_GOLDEN_ROD", Self::new(238, 232, 170, 255))?;
        object.prop("PALE_GREEN", Self::new(152, 251, 152, 255))?;
        object.prop("PALE_TURQUOISE", Self::new(175, 238, 238, 255))?;
        object.prop("PALE_VIOLET_RED", Self::new(219, 112, 147, 255))?;
        object.prop("PAPAYA_WHIP", Self::new(255, 239, 213, 255))?;
        object.prop("PEACH_PUFF", Self::new(255, 218, 185, 255))?;
        object.prop("PERU", Self::new(205, 133, 63, 255))?;
        object.prop("PINK", Self::new(255, 192, 203, 255))?;
        object.prop("PLUM", Self::new(221, 160, 221, 255))?;
        object.prop("POWDER_BLUE", Self::new(176, 224, 230, 255))?;
        object.prop("PURPLE", Self::new(128, 0, 128, 255))?;
        object.prop("REBECCA_PURPLE", Self::new(102, 51, 153, 255))?;
        object.prop("ROSY_BROWN", Self::new(188, 143, 143, 255))?;
        object.prop("ROYAL_BLUE", Self::new(65, 105, 225, 255))?;
        object.prop("SADDLE_BROWN", Self::new(139, 69, 19, 255))?;
        object.prop("SALMON", Self::new(250, 128, 114, 255))?;
        object.prop("SANDY_BROWN", Self::new(244, 164, 96, 255))?;
        object.prop("SEA_GREEN", Self::new(46, 139, 87, 255))?;
        object.prop("SEA_SHELL", Self::new(255, 245, 238, 255))?;
        object.prop("SIENNA", Self::new(160, 82, 45, 255))?;
        object.prop("SILVER", Self::new(192, 192, 192, 255))?;
        object.prop("SKY_BLUE", Self::new(135, 206, 235, 255))?;
        object.prop("SLATE_BLUE", Self::new(106, 90, 205, 255))?;
        object.prop("SLATE_GRAY", Self::new(112, 128, 144, 255))?;
        object.prop("SNOW", Self::new(255, 250, 250, 255))?;
        object.prop("SPRING_GREEN", Self::new(0, 255, 127, 255))?;
        object.prop("STEEL_BLUE", Self::new(70, 130, 180, 255))?;
        object.prop("TAN", Self::new(210, 180, 140, 255))?;
        object.prop("TEAL", Self::new(0, 128, 128, 255))?;
        object.prop("THISTLE", Self::new(216, 191, 216, 255))?;
        object.prop("TOMATO", Self::new(255, 99, 71, 255))?;
        object.prop("TURQUOISE", Self::new(64, 224, 208, 255))?;
        object.prop("VIOLET", Self::new(238, 130, 238, 255))?;
        object.prop("WHEAT", Self::new(245, 222, 179, 255))?;
        object.prop("WHITE_SMOKE", Self::new(245, 245, 245, 255))?;
        object.prop("YELLOW", Self::new(255, 255, 0, 255))?;
        object.prop("YELLOW_GREEN", Self::new(154, 205, 50, 255))?;

        Ok(())
    }
}

impl JsColor {
    /// @skip
    #[must_use]
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            inner: super::Color::new(r, g, b, a),
        }
    }
}

#[rquickjs::methods(rename_all = "camelCase")]
impl JsColor {
    /// Creates a new color.
    ///
    /// Example
    /// ```js
    /// let color = new Color(4, 5, 6);
    /// ```
    ///
    /// @constructor
    ///
    /// @overload
    /// Constructor with three color channels and an alpha channel.
    /// @param r: number // Red (should be between 0-255)
    /// @param g: number // Green (should be between 0-255)
    /// @param b: number // Blue (should be between 0-255)
    /// @param a: number // Alpha (should be between 0-255)
    ///
    /// @overload
    /// Constructor with three color channels.
    /// @param r: number // Red (should be between 0-255)
    /// @param g: number // Green (should be between 0-255)
    /// @param b: number // Blue (should be between 0-255)
    ///
    /// @overload
    /// Constructor with an object.
    /// @param o: {r: number, g: number, b: number}
    ///
    /// @overload
    /// Constructor with an object.
    /// @param o: {r: number, g: number, b: number, a: number}
    ///
    /// @overload
    /// Constructor with another Color.
    /// @param c: Color
    #[qjs(constructor)]
    pub fn new_js(_ctx: Ctx<'_>, _args: Rest<Value<'_>>) -> Result<Self> {
        //let (point, _) = Self::from_args(&ctx, &args.0)?;
        //Ok(point)
        Ok(JsColor {
            inner: Rgba([0, 0, 0, 0]).into(), // TMP // TODO
        })
    }

    /// @skip
    #[qjs(get, rename = "r")]
    #[must_use]
    pub fn get_r(&self) -> u8 {
        self.inner.0[0]
    }

    /// @skip
    #[qjs(set, rename = "r")]
    pub fn set_r(&mut self, r: u8) {
        self.inner.0[0] = r;
    }

    /// @skip
    #[qjs(get, rename = "g")]
    #[must_use]
    pub fn get_g(&self) -> u8 {
        self.inner.0[1]
    }

    /// @skip
    #[qjs(set, rename = "g")]
    pub fn set_g(&mut self, g: u8) {
        self.inner.0[1] = g;
    }

    /// @skip
    #[qjs(get, rename = "b")]
    #[must_use]
    pub fn get_b(&self) -> u8 {
        self.inner.0[2]
    }

    /// @skip
    #[qjs(set, rename = "b")]
    pub fn set_b(&mut self, b: u8) {
        self.inner.0[2] = b;
    }

    /// @skip
    #[qjs(get, rename = "a")]
    #[must_use]
    pub fn get_a(&self) -> u8 {
        self.inner.0[3]
    }

    /// @skip
    #[qjs(set, rename = "a")]
    pub fn set_a(&mut self, a: u8) {
        self.inner.0[3] = a;
    }

    #[must_use]
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        format!(
            "({}, {}, {}, {})",
            self.get_r(),
            self.get_g(),
            self.get_b(),
            self.get_a()
        )
    }

    #[qjs(rename = "clone")]
    #[must_use]
    pub const fn clone_js(&self) -> Self {
        *self
    }
}

impl From<super::Color> for JsColor {
    fn from(value: super::Color) -> Self {
        Self { inner: value }
    }
}

impl From<JsColor> for super::Color {
    fn from(value: JsColor) -> Self {
        value.inner
    }
}

impl From<Rgba<u8>> for JsColor {
    fn from(value: Rgba<u8>) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl From<JsColor> for Rgba<u8> {
    fn from(value: JsColor) -> Self {
        value.inner.into()
    }
}

impl<'js> Trace<'js> for JsColor {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

#[cfg(test)]
mod tests {
    use image::Rgba;
    use rquickjs::Object;
    use tracing_test::traced_test;

    use crate::{core::color::js::JsColor, runtime::Runtime};

    #[test]
    #[traced_test]
    fn test_button() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .with(|ctx| {
                    let color = ctx.globals().get::<_, Object>("Color").unwrap(); // TODO: add a macro? Or a helper function
                    let val = JsColor {
                        inner: Rgba([255, 0, 0, 255]).into(),
                    };
                    color.prop("RED", val).unwrap();
                })
                .await;

            let color = script_engine.eval::<JsColor>("Color.RED2").await.unwrap();
            println!("RED2: {color:?}");
        });
    }
}
