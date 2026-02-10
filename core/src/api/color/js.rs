//! @verbatim /**
//! @verbatim  * ColorLike
//! @verbatim  */
//! @verbatim type ColorLike = Color | { r: number; g: number; b: number; a?: number };

use image::Rgba;
use rquickjs::{
    JsLifetime, Object, Result,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    function::{FromParam, ParamRequirement, ParamsAccessor},
};

use crate::api::{ResultExt, js::classes::ValueClass};

pub struct JsColorLike(pub super::Color);

impl<'js> FromParam<'js> for JsColorLike {
    fn param_requirement() -> ParamRequirement {
        ParamRequirement::single() // 1 -> 1
            .combine(ParamRequirement::optional()) // 1 -> 2
            .combine(ParamRequirement::optional()) // 1 -> 3
            .combine(ParamRequirement::optional()) // 1 -> 4
            .combine(ParamRequirement::optional()) // 1 -> 5
            .combine(ParamRequirement::exhaustive())
    }

    fn from_param<'a>(params: &mut ParamsAccessor<'a, 'js>) -> Result<Self> {
        // Numeric overloads: r, g, b[, a]
        let arg_count = params.len();
        let value = params.arg();
        if value.is_number() {
            let r: u8 = value.get()?;
            let g: u8 = params.arg().get()?;
            let b: u8 = params.arg().get()?;
            let a: u8 = if arg_count >= 4 {
                params.arg().get()?
            } else {
                255
            };

            return Ok(Self(super::Color::new(r, g, b, a)));
        }

        // Also accept a js::Color as a parameter
        if let Ok(js_color) = value.get::<JsColor>() {
            return Ok(Self(js_color.into()));
        }

        // Or an object with r/g/b/a.
        let object = value
            .as_object()
            .or_throw_message(params.ctx(), "Expected an object")?;

        let r: u8 = object.get("r")?;
        let g: u8 = object.get("g")?;
        let b: u8 = object.get("b")?;
        let a: u8 = object.get("a").unwrap_or(255);

        Ok(Self(super::Color::new(r, g, b, a)))
    }
}

/// An RGBA color with 8-bit channels.
///
/// Can be constructed from individual r/g/b/a values, or by using one of the
/// many named color constants (CSS colors).
///
/// ```ts
/// // Create from RGB (alpha defaults to 255)
/// const red = new Color(255, 0, 0);
///
/// // Create from RGBA
/// const semiTransparent = new Color(255, 0, 0, 128);
///
/// // Use a named constant
/// const blue = Color.Blue;
///
/// // Read and modify channels
/// const c = new Color(10, 20, 30);
/// c.r = 100;
/// println(c.toString()); // "(100, 20, 30, 255)"
///
/// // Compare colors
/// Color.Red.equals(new Color(255, 0, 0)); // true
///
/// // Clone a color
/// const copy = Color.Red.clone();
/// ```
///
/// @prop r: number // Red (should be between 0-255)
/// @prop g: number // Green (should be between 0-255)
/// @prop b: number // Blue (should be between 0-255)
/// @prop a: number // Alpha (should be between 0-255)
///
/// @const Red // #FF0000FF
/// @const Green // #008000FF
/// @const Blue // #0000FFFF
/// @const White // #FFFFFFFF
/// @const Black // #000000FF
/// @const Transparent // #00000000
///
/// @const AliceBlue // #F0F8FFFF
/// @const AntiqueWhite // #FAEBD7FF
/// @const Aqua // #00FFFFFF
/// @const Aquamarine // #7FFFD4FF
/// @const Azure // #F0FFFFFF
/// @const Beige // #F5F5DCFF
/// @const Bisque // #FFE4C4FF
/// @const BlanchedAlmond // #FFEBCDFF
/// @const BlueViolet // #8A2BE2FF
/// @const Brown // #A52A2AFF
/// @const BurlyWood // #DEB887FF
/// @const CadetBlue // #5F9EA0FF
/// @const Chartreuse // #7FFF00FF
/// @const Chocolate // #D2691EFF
/// @const Coral // #FF7F50FF
/// @const CornflowerBlue // #6495EDFF
/// @const Cornsilk // #FFF8DCFF
/// @const Crimson // #DC143CFF
/// @const Cyan // #00FFFFFF
/// @const DarkBlue // #00008BFF
/// @const DarkCyan // #008B8BFF
/// @const DarkGoldenRod // #B8860BFF
/// @const DarkGray // #A9A9A9FF
/// @const DarkGreen // #006400FF
/// @const DarkKhaki // #BDB76BFF
/// @const DarkMagenta // #8B008BFF
/// @const DarkOliveGreen // #556B2FFF
/// @const DarkOrange // #FF8C00FF
/// @const DarkOrchid // #9932CCFF
/// @const DarkRed // #8B0000FF
/// @const DarkSalmon // #E9967AFF
/// @const DarkSeaGreen // #8FBC8FFF
/// @const DarkSlateBlue // #483D8BFF
/// @const DarkSlateGray // #2F4F4FFF
/// @const DarkTurquoise // #00CED1FF
/// @const DarkViolet // #9400D3FF
/// @const DeepPink // #FF1493FF
/// @const DeepSkyBlue // #00BFFFFF
/// @const DimGray // #696969FF
/// @const DodgerBlue // #1E90FFFF
/// @const Firebrick // #B22222FF
/// @const FloralWhite // #FFFAF0FF
/// @const ForestGreen // #228B22FF
/// @const Fuchsia // #FF00FFFF
/// @const Gainsboro // #DCDCDCFF
/// @const GhostWhite // #F8F8FFFF
/// @const Gold // #FFD700FF
/// @const GoldenRod // #DAA520FF
/// @const Gray // #808080FF
/// @const GreenYellow // #ADFF2FFF
/// @const HoneyDew // #F0FFF0FF
/// @const HotPink // #FF69B4FF
/// @const IndianRed // #CD5C5CFF
/// @const Indigo // #4B0082FF
/// @const Ivory // #FFFFF0FF
/// @const Khaki // #F0E68CFF
/// @const Lavender // #E6E6FAFF
/// @const LavenderBlush // #FFF0F5FF
/// @const LawnGreen // #7CFC00FF
/// @const LemonChiffon // #FFFACDFF
/// @const LightBlue // #ADD8E6FF
/// @const LightCoral // #F08080FF
/// @const LightCyan // #E0FFFFFF
/// @const LightGoldenRodYellow // #FAFAD2FF
/// @const LightGray // #D3D3D3FF
/// @const LightGreen // #90EE90FF
/// @const LightPink // #FFB6C1FF
/// @const LightSalmon // #FFA07AFF
/// @const LightSeaGreen // #20B2AAFF
/// @const LightSkyBlue // #87CEFAFF
/// @const LightSlateGray // #778899FF
/// @const LightSteelBlue // #B0C4DEFF
/// @const LightYellow // #FFFFE0FF
/// @const Lime // #00FF00FF
/// @const LimeGreen // #32CD32FF
/// @const Linen // #FAF0E6FF
/// @const Magenta // #FF00FFFF
/// @const Maroon // #800000FF
/// @const MediumAquaMarine // #66CDAAFF
/// @const MediumBlue // #0000CDFF
/// @const MediumOrchid // #BA55D3FF
/// @const MediumPurple // #9370DBFF
/// @const MediumSeaGreen // #3CB371FF
/// @const MediumSlateBlue // #7B68EEFF
/// @const MediumSpringGreen // #00FA9AFF
/// @const MediumTurquoise // #48D1CCFF
/// @const MediumVioletRed // #C71585FF
/// @const MidnightBlue // #191970FF
/// @const MintCream // #F5FFFAFF
/// @const MistyRose // #FFE4E1FF
/// @const Moccasin // #FFE4B5FF
/// @const NavajoWhite // #FFDEADFF
/// @const Navy // #000080FF
/// @const OldLace // #FDF5E6FF
/// @const Olive // #808000FF
/// @const OliveDrab // #6B8E23FF
/// @const Orange // #FFA500FF
/// @const OrangeRed // #FF4500FF
/// @const Orchid // #DA70D6FF
/// @const PaleGoldenRod // #EEE8AAFF
/// @const PaleGreen // #98FB98FF
/// @const PaleTurquoise // #AFEEEEFF
/// @const PaleVioletRed // #DB7093FF
/// @const PapayaWhip // #FFEFD5FF
/// @const PeachPuff // #FFDAB9FF
/// @const Peru // #CD853FFF
/// @const Pink // #FFC0CBFF
/// @const Plum // #DDA0DDFF
/// @const PowderBlue // #B0E0E6FF
/// @const Purple // #800080FF
/// @const RebeccaPurple // #663399FF
/// @const RosyBrown // #BC8F8FFF
/// @const RoyalBlue // #4169E1FF
/// @const SaddleBrown // #8B4513FF
/// @const Salmon // #FA8072FF
/// @const SandyBrown // #F4A460FF
/// @const SeaGreen // #2E8B57FF
/// @const SeaShell // #FFF5EEFF
/// @const Sienna // #A0522DFF
/// @const Silver // #C0C0C0FF
/// @const SkyBlue // #87CEEBFF
/// @const SlateBlue // #6A5ACDFF
/// @const SlateGray // #708090FF
/// @const Snow // #FFFAFAFF
/// @const SpringGreen // #00FF7FFF
/// @const SteelBlue // #4682B4FF
/// @const Tan // #D2B48CFF
/// @const Teal // #008080FF
/// @const Thistle // #D8BFD8FF
/// @const Tomato // #FF6347FF
/// @const Turquoise // #40E0D0FF
/// @const Violet // #EE82EEFF
/// @const Wheat // #F5DEB3FF
/// @const WhiteSmoke // #F5F5F5FF
/// @const Yellow // #FFFF00FF
/// @const YellowGreen // #9ACD32FF
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
    /// @param a?: number // Alpha (should be between 0-255)
    ///
    /// @overload
    /// @constructorOnly
    /// Constructor with anything Color-like.
    /// @param c: ColorLike
    #[qjs(constructor)]
    #[must_use]
    pub fn new_js(color: JsColorLike) -> Self {
        color.0.into()
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

    /// Returns `true` if both colors have the same r, g, b, and a values.
    ///
    /// ```ts
    /// Color.Red.equals(new Color(255, 0, 0)); // true
    /// ```
    #[must_use]
    pub fn equals(&self, other: Self) -> bool {
        *self == other
    }

    /// Returns a string representation of the color: `"(r, g, b, a)"`.
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

    /// Returns a copy of this color.
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

    use super::JsColor;
    use crate::{api::color::Color, runtime::Runtime};

    #[test]
    fn test_color_constants() {
        Runtime::test_with_script_engine(async |script_engine| {
            let color = script_engine.eval::<JsColor>("Color.Red").await.unwrap();
            assert_eq!(color, JsColor::new(255, 0, 0, 255));

            let color = script_engine
                .eval::<JsColor>("Color.Transparent")
                .await
                .unwrap();
            assert_eq!(color, JsColor::new(0, 0, 0, 0));
        });
    }

    #[test]
    fn test_color_alias_constants() {
        Runtime::test_with_script_engine(async |script_engine| {
            let result = script_engine
                .eval::<bool>("Color.Aqua.equals(Color.Cyan)")
                .await
                .unwrap();
            assert!(result);

            let result = script_engine
                .eval::<bool>("Color.Fuchsia.equals(Color.Magenta)")
                .await
                .unwrap();
            assert!(result);
        });
    }

    #[test]
    fn test_color_constructor_overloads() {
        Runtime::test_with_script_engine(async |script_engine| {
            let color = script_engine
                .eval::<JsColor>("new Color(1, 2, 3)")
                .await
                .unwrap();
            assert_eq!(color, JsColor::new(1, 2, 3, 255));

            let color = script_engine
                .eval::<JsColor>("new Color(4, 5, 6, 7)")
                .await
                .unwrap();
            assert_eq!(color, JsColor::new(4, 5, 6, 7));

            let color = script_engine
                .eval::<JsColor>("new Color({r: 8, g: 9, b: 10})")
                .await
                .unwrap();
            assert_eq!(color, JsColor::new(8, 9, 10, 255));

            let color = script_engine
                .eval::<JsColor>("new Color(Color.Red)")
                .await
                .unwrap();
            assert_eq!(color, JsColor::new(255, 0, 0, 255));
        });
    }

    #[test]
    fn test_color_js_attributes_and_methods() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval::<()>(
                    r#"
                    let c = Color.Red.clone();
                    c.g = 10;
                    c.b = 11;
                    c.a = 12;
                "#,
                )
                .await
                .unwrap();

            let result = script_engine.eval::<u8>("c.r").await.unwrap();
            assert_eq!(result, 255);

            let result = script_engine.eval::<u8>("c.g").await.unwrap();
            assert_eq!(result, 10);

            let result = script_engine.eval::<u8>("c.b").await.unwrap();
            assert_eq!(result, 11);

            let result = script_engine.eval::<u8>("c.a").await.unwrap();
            assert_eq!(result, 12);

            let result = script_engine
                .eval::<bool>("c.equals(Color.Red)")
                .await
                .unwrap();
            assert!(!result);

            let result = script_engine.eval::<String>("c.toString()").await.unwrap();
            assert_eq!(result, "(255, 10, 11, 12)");

            let result = script_engine
                .eval::<bool>("c.clone().equals(c)")
                .await
                .unwrap();
            assert!(result);
        });
    }

    #[test]
    fn test_color_rust_methods() {
        let mut color = JsColor::new(1, 2, 3, 4);

        assert_eq!(color.get_r(), 1);
        assert_eq!(color.get_g(), 2);
        assert_eq!(color.get_b(), 3);
        assert_eq!(color.get_a(), 4);

        color.set_r(5);
        color.set_g(6);
        color.set_b(7);
        color.set_a(8);

        assert!(color.equals(JsColor::new(5, 6, 7, 8)));
        assert_eq!(color.to_string_js(), "(5, 6, 7, 8)");
        assert_eq!(color.clone_js(), color);
    }

    #[test]
    fn test_color_conversions() {
        let color = Color::new(9, 8, 7, 6);
        let js_color = JsColor::from(color);
        assert_eq!(js_color, JsColor::new(9, 8, 7, 6));
        assert_eq!(Color::from(js_color), color);

        let rgba = Rgba([10, 20, 30, 40]);
        let js_color = JsColor::from(rgba);
        assert_eq!(Rgba::<u8>::from(js_color), rgba);
        assert_eq!(Color::from(js_color), Color::new(10, 20, 30, 40));
    }

    #[test]
    fn test_color_clone_is_not_strictly_equal_in_js() {
        Runtime::test_with_script_engine(async |script_engine| {
            script_engine
                .eval::<()>("let c = Color.Red.clone()")
                .await
                .unwrap();

            let result = script_engine
                .eval::<bool>("c.equals(Color.Red)")
                .await
                .unwrap();
            assert!(result);

            let result = script_engine.eval::<bool>("c == Color.Red").await.unwrap();
            assert!(!result);
        });
    }
}
