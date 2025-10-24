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
        ParamRequirement::single()
            .combine(ParamRequirement::optional())
            .combine(ParamRequirement::optional())
            .combine(ParamRequirement::optional())
            .combine(ParamRequirement::exhaustive())
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
/// @const Red
/// @const Green
/// @const Blue
/// @const White
/// @const Black
/// @const Transparent
///
/// @const AliceBlue
/// @const AntiqueWhite
/// @const Aqua
/// @const Aquamarine
/// @const Azure
/// @const Beige
/// @const Bisque
/// @const BlanchedAlmond
/// @const BlueViolet
/// @const Brown
/// @const BurlyWood
/// @const CadetBlue
/// @const Chartreuse
/// @const Chocolate
/// @const Coral
/// @const CornflowerBlue
/// @const Cornsilk
/// @const Crimson
/// @const Cyan
/// @const DarkBlue
/// @const DarkCyan
/// @const DarkGoldenRod
/// @const DarkGray
/// @const DarkGreen
/// @const DarkKhaki
/// @const DarkMagenta
/// @const DarkOliveGreen
/// @const DarkOrange
/// @const DarkOrchid
/// @const DarkRed
/// @const DarkSalmon
/// @const DarkSeaGreen
/// @const DarkSlateBlue
/// @const DarkSlateGray
/// @const DarkTurquoise
/// @const DarkViolet
/// @const DeepPink
/// @const DeepSkyBlue
/// @const DimGray
/// @const DodgerBlue
/// @const Firebrick
/// @const FloralWhite
/// @const ForestGreen
/// @const Fuchsia
/// @const Gainsboro
/// @const GhostWhite
/// @const Gold
/// @const GoldenRod
/// @const Gray
/// @const GreenYellow
/// @const HoneyDew
/// @const HotPink
/// @const IndianRed
/// @const Indigo
/// @const Ivory
/// @const Khaki
/// @const Lavender
/// @const LavenderBlush
/// @const LawnGreen
/// @const LemonChiffon
/// @const LightBlue
/// @const LightCoral
/// @const LightCyan
/// @const LightGoldenRodYellow
/// @const LightGray
/// @const LightGreen
/// @const LightPink
/// @const LightSalmon
/// @const LightSeaGreen
/// @const LightSkyBlue
/// @const LightSlateGray
/// @const LightSteelBlue
/// @const LightYellow
/// @const Lime
/// @const LimeGreen
/// @const Linen
/// @const Magenta
/// @const Maroon
/// @const MediumAquaMarine
/// @const MediumBlue
/// @const MediumOrchid
/// @const MediumPurple
/// @const MediumSeaGreen
/// @const MediumSlateBlue
/// @const MediumSpringGreen
/// @const MediumTurquoise
/// @const MediumVioletRed
/// @const MidnightBlue
/// @const MintCream
/// @const MistyRose
/// @const Moccasin
/// @const NavajoWhite
/// @const Navy
/// @const OldLace
/// @const Olive
/// @const OliveDrab
/// @const Orange
/// @const OrangeRed
/// @const Orchid
/// @const PaleGoldenRod
/// @const PaleGreen
/// @const PaleTurquoise
/// @const PaleVioletRed
/// @const PapayaWhip
/// @const PeachPuff
/// @const Peru
/// @const Pink
/// @const Plum
/// @const PowderBlue
/// @const Purple
/// @const RebeccaPurple
/// @const RosyBrown
/// @const RoyalBlue
/// @const SaddleBrown
/// @const Salmon
/// @const SandyBrown
/// @const SeaGreen
/// @const SeaShell
/// @const Sienna
/// @const Silver
/// @const SkyBlue
/// @const SlateBlue
/// @const SlateGray
/// @const Snow
/// @const SpringGreen
/// @const SteelBlue
/// @const Tan
/// @const Teal
/// @const Thistle
/// @const Tomato
/// @const Turquoise
/// @const Violet
/// @const Wheat
/// @const WhiteSmoke
/// @const Yellow
/// @const YellowGreen
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
        object.prop("Red", Self::new(255, 0, 0, 255))?;
        object.prop("Green", Self::new(0, 128, 0, 255))?; // CSS green is #008000
        object.prop("Blue", Self::new(0, 0, 255, 255))?;
        object.prop("White", Self::new(255, 255, 255, 255))?;
        object.prop("Black", Self::new(0, 0, 0, 255))?;
        object.prop("Transparent", Self::new(0, 0, 0, 0))?;

        // CSS colors
        object.prop("AliceBlue", Self::new(240, 248, 255, 255))?;
        object.prop("AntiqueWhite", Self::new(250, 235, 215, 255))?;
        object.prop("Aqua", Self::new(0, 255, 255, 255))?;
        object.prop("Aquamarine", Self::new(127, 255, 212, 255))?;
        object.prop("Azure", Self::new(240, 255, 255, 255))?;
        object.prop("Beige", Self::new(245, 245, 220, 255))?;
        object.prop("Bisque", Self::new(255, 228, 196, 255))?;
        object.prop("BlanchedAlmond", Self::new(255, 235, 205, 255))?;
        object.prop("BlueViolet", Self::new(138, 43, 226, 255))?;
        object.prop("Brown", Self::new(165, 42, 42, 255))?;
        object.prop("BurlyWood", Self::new(222, 184, 135, 255))?;
        object.prop("CadetBlue", Self::new(95, 158, 160, 255))?;
        object.prop("Chartreuse", Self::new(127, 255, 0, 255))?;
        object.prop("Chocolate", Self::new(210, 105, 30, 255))?;
        object.prop("Coral", Self::new(255, 127, 80, 255))?;
        object.prop("CornflowerBlue", Self::new(100, 149, 237, 255))?;
        object.prop("Cornsilk", Self::new(255, 248, 220, 255))?;
        object.prop("Crimson", Self::new(220, 20, 60, 255))?;
        object.prop("Cyan", Self::new(0, 255, 255, 255))?;
        object.prop("DarkBlue", Self::new(0, 0, 139, 255))?;
        object.prop("DarkCyan", Self::new(0, 139, 139, 255))?;
        object.prop("DarkGoldenRod", Self::new(184, 134, 11, 255))?;
        object.prop("DarkGray", Self::new(169, 169, 169, 255))?;
        object.prop("DarkGreen", Self::new(0, 100, 0, 255))?;
        object.prop("DarkKhaki", Self::new(189, 183, 107, 255))?;
        object.prop("DarkMagenta", Self::new(139, 0, 139, 255))?;
        object.prop("DarkOliveGreen", Self::new(85, 107, 47, 255))?;
        object.prop("DarkOrange", Self::new(255, 140, 0, 255))?;
        object.prop("DarkOrchid", Self::new(153, 50, 204, 255))?;
        object.prop("DarkRed", Self::new(139, 0, 0, 255))?;
        object.prop("DarkSalmon", Self::new(233, 150, 122, 255))?;
        object.prop("DarkSeaGreen", Self::new(143, 188, 143, 255))?;
        object.prop("DarkSlateBlue", Self::new(72, 61, 139, 255))?;
        object.prop("DarkSlateGray", Self::new(47, 79, 79, 255))?;
        object.prop("DarkTurquoise", Self::new(0, 206, 209, 255))?;
        object.prop("DarkViolet", Self::new(148, 0, 211, 255))?;
        object.prop("DeepPink", Self::new(255, 20, 147, 255))?;
        object.prop("DeepSkyBlue", Self::new(0, 191, 255, 255))?;
        object.prop("DimGray", Self::new(105, 105, 105, 255))?;
        object.prop("DodgerBlue", Self::new(30, 144, 255, 255))?;
        object.prop("Firebrick", Self::new(178, 34, 34, 255))?;
        object.prop("FloralWhite", Self::new(255, 250, 240, 255))?;
        object.prop("ForestGreen", Self::new(34, 139, 34, 255))?;
        object.prop("Fuchsia", Self::new(255, 0, 255, 255))?;
        object.prop("Gainsboro", Self::new(220, 220, 220, 255))?;
        object.prop("GhostWhite", Self::new(248, 248, 255, 255))?;
        object.prop("Gold", Self::new(255, 215, 0, 255))?;
        object.prop("GoldenRod", Self::new(218, 165, 32, 255))?;
        object.prop("Gray", Self::new(128, 128, 128, 255))?;
        object.prop("GreenYellow", Self::new(173, 255, 47, 255))?;
        object.prop("HoneyDew", Self::new(240, 255, 240, 255))?;
        object.prop("HotPink", Self::new(255, 105, 180, 255))?;
        object.prop("IndianRed", Self::new(205, 92, 92, 255))?;
        object.prop("Indigo", Self::new(75, 0, 130, 255))?;
        object.prop("Ivory", Self::new(255, 255, 240, 255))?;
        object.prop("Khaki", Self::new(240, 230, 140, 255))?;
        object.prop("Lavender", Self::new(230, 230, 250, 255))?;
        object.prop("LavenderBlush", Self::new(255, 240, 245, 255))?;
        object.prop("LawnGreen", Self::new(124, 252, 0, 255))?;
        object.prop("LemonChiffon", Self::new(255, 250, 205, 255))?;
        object.prop("LightBlue", Self::new(173, 216, 230, 255))?;
        object.prop("LightCoral", Self::new(240, 128, 128, 255))?;
        object.prop("LightCyan", Self::new(224, 255, 255, 255))?;
        object.prop("LightGoldenRodYellow", Self::new(250, 250, 210, 255))?;
        object.prop("LightGray", Self::new(211, 211, 211, 255))?;
        object.prop("LightGreen", Self::new(144, 238, 144, 255))?;
        object.prop("LightPink", Self::new(255, 182, 193, 255))?;
        object.prop("LightSalmon", Self::new(255, 160, 122, 255))?;
        object.prop("LightSeaGreen", Self::new(32, 178, 170, 255))?;
        object.prop("LightSkyBlue", Self::new(135, 206, 250, 255))?;
        object.prop("LightSlateGray", Self::new(119, 136, 153, 255))?;
        object.prop("LightSteelBlue", Self::new(176, 196, 222, 255))?;
        object.prop("LightYellow", Self::new(255, 255, 224, 255))?;
        object.prop("Lime", Self::new(0, 255, 0, 255))?;
        object.prop("LimeGreen", Self::new(50, 205, 50, 255))?;
        object.prop("Linen", Self::new(250, 240, 230, 255))?;
        object.prop("Magenta", Self::new(255, 0, 255, 255))?;
        object.prop("Maroon", Self::new(128, 0, 0, 255))?;
        object.prop("MediumAquaMarine", Self::new(102, 205, 170, 255))?;
        object.prop("MediumBlue", Self::new(0, 0, 205, 255))?;
        object.prop("MediumOrchid", Self::new(186, 85, 211, 255))?;
        object.prop("MediumPurple", Self::new(147, 112, 219, 255))?;
        object.prop("MediumSeaGreen", Self::new(60, 179, 113, 255))?;
        object.prop("MediumSlateBlue", Self::new(123, 104, 238, 255))?;
        object.prop("MediumSpringGreen", Self::new(0, 250, 154, 255))?;
        object.prop("MediumTurquoise", Self::new(72, 209, 204, 255))?;
        object.prop("MediumVioletRed", Self::new(199, 21, 133, 255))?;
        object.prop("MidnightBlue", Self::new(25, 25, 112, 255))?;
        object.prop("MintCream", Self::new(245, 255, 250, 255))?;
        object.prop("MistyRose", Self::new(255, 228, 225, 255))?;
        object.prop("Moccasin", Self::new(255, 228, 181, 255))?;
        object.prop("NavajoWhite", Self::new(255, 222, 173, 255))?;
        object.prop("Navy", Self::new(0, 0, 128, 255))?;
        object.prop("OldLace", Self::new(253, 245, 230, 255))?;
        object.prop("Olive", Self::new(128, 128, 0, 255))?;
        object.prop("OliveDrab", Self::new(107, 142, 35, 255))?;
        object.prop("Orange", Self::new(255, 165, 0, 255))?;
        object.prop("OrangeRed", Self::new(255, 69, 0, 255))?;
        object.prop("Orchid", Self::new(218, 112, 214, 255))?;
        object.prop("PaleGoldenRod", Self::new(238, 232, 170, 255))?;
        object.prop("PaleGreen", Self::new(152, 251, 152, 255))?;
        object.prop("PaleTurquoise", Self::new(175, 238, 238, 255))?;
        object.prop("PaleVioletRed", Self::new(219, 112, 147, 255))?;
        object.prop("PapayaWhip", Self::new(255, 239, 213, 255))?;
        object.prop("PeachPuff", Self::new(255, 218, 185, 255))?;
        object.prop("Peru", Self::new(205, 133, 63, 255))?;
        object.prop("Pink", Self::new(255, 192, 203, 255))?;
        object.prop("Plum", Self::new(221, 160, 221, 255))?;
        object.prop("PowderBlue", Self::new(176, 224, 230, 255))?;
        object.prop("Purple", Self::new(128, 0, 128, 255))?;
        object.prop("RebeccaPurple", Self::new(102, 51, 153, 255))?;
        object.prop("RosyBrown", Self::new(188, 143, 143, 255))?;
        object.prop("RoyalBlue", Self::new(65, 105, 225, 255))?;
        object.prop("SaddleBrown", Self::new(139, 69, 19, 255))?;
        object.prop("Salmon", Self::new(250, 128, 114, 255))?;
        object.prop("SandyBrown", Self::new(244, 164, 96, 255))?;
        object.prop("SeaGreen", Self::new(46, 139, 87, 255))?;
        object.prop("SeaShell", Self::new(255, 245, 238, 255))?;
        object.prop("Sienna", Self::new(160, 82, 45, 255))?;
        object.prop("Silver", Self::new(192, 192, 192, 255))?;
        object.prop("SkyBlue", Self::new(135, 206, 235, 255))?;
        object.prop("SlateBlue", Self::new(106, 90, 205, 255))?;
        object.prop("SlateGray", Self::new(112, 128, 144, 255))?;
        object.prop("Snow", Self::new(255, 250, 250, 255))?;
        object.prop("SpringGreen", Self::new(0, 255, 127, 255))?;
        object.prop("SteelBlue", Self::new(70, 130, 180, 255))?;
        object.prop("Tan", Self::new(210, 180, 140, 255))?;
        object.prop("Teal", Self::new(0, 128, 128, 255))?;
        object.prop("Thistle", Self::new(216, 191, 216, 255))?;
        object.prop("Tomato", Self::new(255, 99, 71, 255))?;
        object.prop("Turquoise", Self::new(64, 224, 208, 255))?;
        object.prop("Violet", Self::new(238, 130, 238, 255))?;
        object.prop("Wheat", Self::new(245, 222, 179, 255))?;
        object.prop("WhiteSmoke", Self::new(245, 245, 245, 255))?;
        object.prop("Yellow", Self::new(255, 255, 0, 255))?;
        object.prop("YellowGreen", Self::new(154, 205, 50, 255))?;

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
    use tracing_test::traced_test;

    use crate::{core::color::js::JsColor, runtime::Runtime};

    #[test]
    #[traced_test]
    fn test_button() {
        Runtime::test_with_script_engine(async |script_engine| {
            let color = script_engine.eval::<JsColor>("Color.Red").await.unwrap();
            println!("Red: {color:?}");
        });
    }
}
