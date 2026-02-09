use std::fmt::Debug;

use rquickjs::{
    Array, Ctx, Exception, JsLifetime, Result, Value,
    class::{Trace, Tracer},
    prelude::{Opt, Rest},
};

use crate::{
    IntoJsResult,
    api::{displays, js::classes::SingletonClass, point::js::JsPoint},
    runtime::WithUserData,
};

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
/// console.log(random.number()); // always the same value
/// random.resetSeed();
/// ```
///
/// @singleton
#[derive(Debug, Default, JsLifetime)]
#[rquickjs::class(rename = "Random")]
pub struct JsRandom {}

impl<'js> Trace<'js> for JsRandom {
    fn trace<'a>(&self, _tracer: Tracer<'a, 'js>) {}
}

impl<'js> SingletonClass<'js> for JsRandom {}

#[rquickjs::methods(rename_all = "camelCase")]
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

    /// Returns a random position on any display.
    ///
    /// ```ts
    /// const pos = await random.position();
    /// console.log(pos.toString());
    /// ```
    /// @readonly
    pub async fn position(&mut self, ctx: Ctx<'_>) -> Result<JsPoint> {
        let user_data = ctx.user_data();

        let point: displays::Result<JsPoint> = user_data
            .displays()
            .random_point(user_data.rng())
            .await
            .map(|point| point.into());

        point.into_js_result(&ctx)
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
    /// console.log(item); // "default"
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
}

#[cfg(test)]
mod tests {
    use crate::runtime::Runtime;

    // TODO: add tests
    #[test]
    fn test_random() {
        Runtime::test_with_script_engine(|script_engine| async move {
            let result = script_engine
                .eval::<i64>("random.integer(10)")
                .await
                .unwrap();
            println!("{result}");
        });
    }
}
