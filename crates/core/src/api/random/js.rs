use std::{borrow::Cow, fmt::Debug};

use macros::{FromJsObject, js_class, js_methods, options};
use rquickjs::{
    Array, Ctx, Exception, JsLifetime, Result, Value,
    atom::PredefinedAtom,
    class::{Trace, Tracer},
    prelude::{Opt, Rest},
};
use unicode_segmentation::UnicodeSegmentation;
use uuid::Builder as UuidBuilder;

use crate::{
    IntoJsResult,
    api::{color::js::JsColor, js::classes::SingletonClass, point::js::JsPoint},
    runtime::WithUserData,
};

const ASCII_NUMBER_CHARACTERS: &str = "0123456789";
const ASCII_LETTER_CHARACTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const ASCII_SPECIAL_CHARACTERS: &str = r##" !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"##;

/// Options for generating random strings.
///
/// ```ts
/// const token = random.string(32);
/// const pin = random.string(6, { characters: "0123456789" });
/// ```
#[options]
#[derive(Clone, Debug, FromJsObject)]
pub struct JsRandomStringOptions {
    /// Possible characters to pick from.
    /// Can contain any Unicode grapheme cluster.
    /// When `characters` is specified, `allowNumbers`, `allowLetters` and `allowSpecialCharacters` are ignored.
    /// Defaults to all printable ASCII characters.
    pub characters: Option<String>,

    /// Include digits `0-9` in the default character set.
    /// Ignored when `characters` is specified.
    #[default(true)]
    pub allow_numbers: bool,

    /// Include letters `A-Z` and `a-z` in the default character set.
    /// Ignored when `characters` is specified.
    #[default(true)]
    pub allow_letters: bool,

    /// Include printable ASCII non-alphanumeric characters in the default character set.
    /// Ignored when `characters` is specified.
    #[default(true)]
    pub allow_special_characters: bool,
}

/// Random number generator.
///
/// Provides methods for generating random numbers, integers, positions, and choices.
/// The generator is deterministic when seeded.
///
/// ```ts
/// const n = random.number(); // 0..1
/// const i = random.integer(1, 10); // 1..10
/// const item = random.choice(["a", "b", "c"]);
/// ```
///
/// ```ts
/// random.setSeed(42);
/// println(random.number()); // always the same value
/// random.resetSeed();
/// ```
///
/// @singleton
#[derive(Debug, Default, JsLifetime)]
#[js_class]
pub struct JsRandom {}

impl<'js> Trace<'js> for JsRandom {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> SingletonClass<'js> for JsRandom {}

#[js_methods]
impl JsRandom {
    /// Returns a random floating-point number.
    ///
    /// ```ts
    /// const a = random.number();        // 0..1
    /// const b = random.number(10);      // 0..10
    /// const c = random.number(5, 10);   // 5..10
    /// ```
    ///
    /// @overload
    /// Returns a number between 0 (inclusive) and 1 (exclusive)
    ///
    /// @overload
    /// Returns a number between 0 (inclusive) and max (exclusive)
    /// @param max: number
    ///
    /// @overload
    /// Returns a number between min (inclusive) and max (exclusive)
    /// @param min: number
    /// @param max: number
    pub fn number(&self, ctx: Ctx<'_>, args: Rest<f64>) -> Result<f64> {
        Ok(match args.as_slice() {
            [min, max, ..] => {
                if min >= max {
                    return Err(Exception::throw_message(
                        &ctx,
                        "min should be less than max",
                    ));
                }
                ctx.user_data().rng().random_range(*min..*max)
            }
            [max] => {
                if *max <= 0.0 {
                    return Err(Exception::throw_message(&ctx, "max must be greater than 0"));
                }
                ctx.user_data().rng().random_range(0.0..*max)
            }
            [] => ctx.user_data().rng().random(),
        })
    }

    /// Returns a random integer.
    ///
    /// ```ts
    /// const a = random.integer(10);     // 0..10
    /// const b = random.integer(5, 10);  // 5..10
    /// ```
    ///
    /// @overload
    /// Returns an integer between 0 (inclusive) and max (inclusive)
    /// @param max: number
    ///
    /// @overload
    /// Returns an integer between min (inclusive) and max (inclusive)
    /// @param min: number
    /// @param max: number
    pub fn integer(&self, ctx: Ctx<'_>, args: Rest<i64>) -> Result<i64> {
        if args.is_empty() {
            return Err(Exception::throw_message(
                &ctx,
                "expected at least 1 argument",
            ));
        }

        Ok(match args.as_slice() {
            [min, max, ..] => {
                if min > max {
                    return Err(Exception::throw_message(
                        &ctx,
                        "min should be less or equal than max",
                    ));
                }
                ctx.user_data().rng().random_range(*min..=*max)
            }
            [max] => {
                if *max < 0 {
                    return Err(Exception::throw_message(
                        &ctx,
                        "max must be greater or equal than 0",
                    ));
                }
                ctx.user_data().rng().random_range(0..=*max)
            }
            [] => ctx.user_data().rng().random(),
        })
    }

    /// Sets the seed to a value.
    /// This seed is used for all random number generation. Since the random number generator is
    /// deterministic that means that setting it to a particular number will always generate the same
    /// random numbers.
    ///
    /// ```ts
    /// random.setSeed(42);
    /// ```
    pub fn set_seed(&self, ctx: Ctx<'_>, seed: u64) {
        ctx.user_data().rng().set_seed(seed);
    }

    /// Resets the seed to be a random one.
    ///
    /// ```ts
    /// random.resetSeed();
    /// ```
    pub fn reset_seed(&self, ctx: Ctx<'_>) {
        ctx.user_data().rng().reset_seed();
    }

    /// Returns a random position on any display, or `undefined` if there is no display.
    ///
    /// ```ts
    /// const pos = await random.position();
    /// if (pos) {
    ///   println(pos);
    /// }
    /// ```
    /// @readonly
    pub async fn position(&mut self, ctx: Ctx<'_>) -> Result<Option<JsPoint>> {
        let user_data = ctx.user_data();

        let point = user_data
            .displays()
            .random_point(user_data.rng())
            .await
            .into_js_result(&ctx)?
            .map(Into::into);

        Ok(point)
    }

    /// Returns a random color with full opacity.
    ///
    /// ```ts
    /// const c = random.color();
    /// println(c); // Color(r: ?, g: ?, b: ?, a: 255)
    /// ```
    /// @readonly
    #[must_use]
    pub fn color(&self, ctx: Ctx<'_>) -> JsColor {
        let rng = ctx.user_data().rng();

        JsColor::new(rng.random(), rng.random(), rng.random(), 255)
    }

    /// Returns a random color including a random alpha channel.
    ///
    /// ```ts
    /// const c = random.colorWithAlpha();
    /// println(c); // Color(r: ?, g: ?, b: ?, a: ?)
    /// ```
    /// @readonly
    #[must_use]
    pub fn color_with_alpha(&self, ctx: Ctx<'_>) -> JsColor {
        let rng = ctx.user_data().rng();

        JsColor::new(rng.random(), rng.random(), rng.random(), rng.random())
    }

    /// Returns a random string of the given length.
    ///
    /// ```ts
    /// const token = random.string(16);
    /// ```
    ///
    /// ```ts
    /// const code = random.string(8, { characters: "ABCDEF0123456789" });
    /// ```
    pub fn string(
        &self,
        ctx: Ctx<'_>,
        length: u32,
        options: Opt<JsRandomStringOptions>,
    ) -> Result<String> {
        let length = usize::try_from(length)
            .map_err(|_| Exception::throw_message(&ctx, "length is too large"))?;
        let options = options.0.unwrap_or_default();
        let explicit_characters = options.characters.as_deref();

        let characters: Cow<'_, str> = if let Some(characters) = explicit_characters {
            Cow::Borrowed(characters)
        } else {
            let mut generated_characters = String::new();

            if options.allow_numbers {
                generated_characters.push_str(ASCII_NUMBER_CHARACTERS);
            }

            if options.allow_letters {
                generated_characters.push_str(ASCII_LETTER_CHARACTERS);
            }

            if options.allow_special_characters {
                generated_characters.push_str(ASCII_SPECIAL_CHARACTERS);
            }

            Cow::Owned(generated_characters)
        };

        if characters.is_empty() {
            if explicit_characters.is_some() {
                return Err(Exception::throw_message(
                    &ctx,
                    "options.characters must not be empty",
                ));
            }

            return Err(Exception::throw_message(
                &ctx,
                "at least one of options.allowNumbers, options.allowLetters, options.allowSpecialCharacters must be true",
            ));
        }

        let candidates = characters.graphemes(true).collect::<Vec<_>>();
        if candidates.is_empty() {
            return Err(Exception::throw_message(
                &ctx,
                "options.characters must contain at least 1 grapheme cluster",
            ));
        }

        let rng = ctx.user_data().rng();
        let mut output = String::new();

        for _ in 0..length {
            let index = rng.random_range(0..candidates.len());
            output.push_str(candidates[index]);
        }

        Ok(output)
    }

    /// Returns a random UUID (v4).
    ///
    /// ```ts
    /// const id = random.uuid();
    /// println(id); // e.g. "f47ac10b-58cc-4372-a567-0e02b2c3d479"
    /// ```
    #[must_use]
    pub fn uuid(&self, ctx: Ctx<'_>) -> String {
        let rng = ctx.user_data().rng();
        let mut bytes = [0_u8; 16];

        for byte in &mut bytes {
            *byte = rng.random();
        }

        UuidBuilder::from_random_bytes(bytes)
            .into_uuid()
            .to_string()
    }

    /// Chooses one random entry in an array.
    /// A fallback can be provided, in case the array is empty.
    ///
    /// ```ts
    /// const item = random.choice(["apple", "banana", "cherry"]);
    /// ```
    ///
    /// ```ts
    /// const item = random.choice([], "default");
    /// println(item); // "default"
    /// ```
    ///
    /// @generic
    /// @param array: Array<T>
    /// @param fallback?: T
    /// @returns T
    pub fn choice<'js>(
        &self,
        ctx: Ctx<'js>,
        array: Array<'js>,
        fallback: Opt<Value<'js>>,
    ) -> Result<Value<'js>> {
        if array.is_empty() {
            if let Some(fallback) = fallback.0 {
                return Ok(fallback);
            } else {
                return Err(Exception::throw_message(
                    &ctx,
                    "empty array and no fallback set",
                ));
            }
        }

        let index = ctx.user_data().rng().random_range(0..array.len());
        let value = array.get(index)?;

        Ok(value)
    }

    /// Returns a string representation of the `random` singleton.
    #[qjs(rename = PredefinedAtom::ToString)]
    #[must_use]
    pub fn to_string_js(&self) -> String {
        "Random".to_string()
    }
}

#[cfg(test)]
mod tests {
    use unicode_segmentation::UnicodeSegmentation;

    use crate::runtime::Runtime;

    #[test]
    fn test_random_string_accepts_unicode_character_set() {
        Runtime::test_with_script_engine(async |script_engine| {
            let value = script_engine
                .eval::<String>(r#"random.string(128, { characters: "a\u0302👍🏽🇬🇧" })"#)
                .await
                .unwrap();

            let allowed = "a\u{0302}👍🏽🇬🇧".graphemes(true).collect::<Vec<_>>();
            let generated = value.graphemes(true).collect::<Vec<_>>();
            assert_eq!(generated.len(), 128);
            assert!(generated.iter().all(|g| allowed.contains(g)));
        });
    }
}
