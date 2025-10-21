use std::num::Saturating;

use eyre::{Report, Result, eyre};
use rquickjs::{FromJs, IntoJs};

use crate::{IntoJsResult, types::si32::Si32};

impl From<i32> for Si32 {
    fn from(value: i32) -> Self {
        Self(Saturating(value))
    }
}

impl From<Si32> for i32 {
    fn from(value: Si32) -> Self {
        value.into_inner()
    }
}

impl From<u32> for Si32 {
    #[allow(clippy::as_conversions)]
    fn from(value: u32) -> Self {
        let value = value.clamp(0, i32::MAX as u32);

        Self::new(value as i32)
    }
}

impl From<Si32> for u32 {
    #[allow(clippy::as_conversions)]
    fn from(value: Si32) -> Self {
        let v = value.into_inner();
        if v < 0 { 0 } else { v as Self }
    }
}

impl From<i16> for Si32 {
    fn from(value: i16) -> Self {
        Self::new(value.into())
    }
}

impl From<Si32> for i16 {
    #[allow(clippy::as_conversions)]
    fn from(value: Si32) -> Self {
        let value = value.into_inner().clamp(Self::MIN.into(), Self::MAX.into());

        value as Self
    }
}

impl From<usize> for Si32 {
    #[allow(clippy::as_conversions)]
    fn from(value: usize) -> Self {
        let value = value.clamp(0, i32::MAX as usize);

        Self::new(value as i32)
    }
}

impl From<i64> for Si32 {
    fn from(value: i64) -> Self {
        let value = value.clamp(i32::MIN.into(), i32::MAX.into());

        #[allow(clippy::as_conversions)]
        Self::new(value as i32)
    }
}

impl From<Si32> for i64 {
    fn from(value: Si32) -> Self {
        value.into_inner().into()
    }
}

impl TryFrom<f64> for Si32 {
    type Error = Report;

    fn try_from(value: f64) -> Result<Self> {
        if value.is_nan() {
            return Err(eyre!("value is not a number (NaN)"));
        }
        if value.is_infinite() {
            return Err(eyre!("value is infinite"));
        }

        let value = value.clamp(i32::MIN.into(), i32::MAX.into());

        #[allow(clippy::as_conversions)]
        Ok(Self::new(value as i32))
    }
}

impl From<Si32> for f64 {
    fn from(value: Si32) -> Self {
        value.into_inner().into()
    }
}

impl From<Si32> for f32 {
    #[allow(clippy::as_conversions)]
    fn from(value: Si32) -> Self {
        value.into_inner() as Self
    }
}

impl<'js> IntoJs<'js> for Si32 {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let value: f64 = self.into();

        value.into_js(ctx)
    }
}

impl<'js> FromJs<'js> for Si32 {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let value: f64 = value.get()?;

        IntoJsResult::into_js_result(Self::try_from(value), ctx)
    }
}

#[cfg(test)]
#[allow(clippy::as_conversions)]
mod tests {
    use rquickjs::{Context, Ctx, FromJs, IntoJs, Null, Runtime, Undefined, Value};
    use rstest::rstest;

    use crate::types::si32::{Si32, si32};

    // Helper: run code inside a JS context
    fn with_ctx<F: FnOnce(Ctx) -> rquickjs::Result<()>>(f: F) {
        let rt = Runtime::new().expect("rt");
        let ctx = Context::full(&rt).expect("ctx");
        ctx.with(f).expect("js ok");
    }

    // ------------------------ From<i32> -> Si32  -----------------------------

    #[rstest]
    #[case::zero(0, si32(0))]
    #[case::pos(123, si32(123))]
    #[case::neg(-456, si32(-456))]
    #[case::min(i32::MIN, si32(i32::MIN))]
    #[case::max(i32::MAX, si32(i32::MAX))]
    fn from_i32(#[case] src: i32, #[case] want: Si32) {
        assert_eq!(want, Si32::from(src));
    }

    // ------------------------ From<Si32> -> i32  -----------------------------

    #[rstest]
    #[case::zero(si32(0), 0)]
    #[case::pos(si32(123), 123)]
    #[case::neg(si32(-456), -456)]
    #[case::min(si32(i32::MIN), i32::MIN)]
    #[case::max(si32(i32::MAX), i32::MAX)]
    fn into_i32(#[case] src: Si32, #[case] want: i32) {
        let got: i32 = src.into();
        assert_eq!(want, got);
    }

    // ------------------------ From<u32> -> Si32  (clamp to i32::MAX) ---------

    #[rstest]
    #[case::zero(0u32, si32(0))]
    #[case::small(123u32, si32(123))]
    #[case::at_i32_max(i32::MAX as u32, si32(i32::MAX))]
    #[case::over_i32_max((i32::MAX as u32) + 1, si32(i32::MAX))] // clamped
    #[case::u32_max(u32::MAX, si32(i32::MAX))] // clamped
    fn from_u32(#[case] src: u32, #[case] want: Si32) {
        assert_eq!(want, Si32::from(src));
    }

    // ------------------------ From<Si32> -> u32  (clamp 0..=u32::MAX) --------

    #[rstest]
    #[case::neg_to_zero(si32(-1), 0u32)] // clamped to 0
    #[case::zero(si32(0), 0u32)]
    #[case::small(si32(123), 123u32)]
    #[case::i32_max_to_u32(si32(i32::MAX), i32::MAX as u32)] // still <= u32::MAX
    fn into_u32(#[case] src: Si32, #[case] want: u32) {
        let got: u32 = src.into();
        assert_eq!(want, got);
    }

    // ------------------------ i16 <-> Si32 (clamping on downcast) ------------

    #[rstest]
    #[case::zero(0i16, si32(0))]
    #[case::pos(123i16, si32(123))]
    #[case::neg(-456i16, si32(-456))]
    #[case::min(i16::MIN, si32(i16::MIN as i32))]
    #[case::max(i16::MAX, si32(i16::MAX as i32))]
    fn from_i16(#[case] src: i16, #[case] want: Si32) {
        assert_eq!(want, Si32::from(src));
    }

    #[rstest]
    #[case::in_range(si32(123), 123i16)]
    #[case::neg_in_range(si32(-456), -456i16)]
    #[case::clamp_low(si32(i32::MIN), i16::MIN)] // clamped
    #[case::clamp_high(si32(i32::MAX), i16::MAX)] // clamped
    fn into_i16(#[case] src: Si32, #[case] want: i16) {
        let got: i16 = src.into();
        assert_eq!(want, got);
    }

    // ------------------------ From<usize> -> Si32 (clamp to i32::MAX) --------

    #[rstest]
    #[case::zero(0usize, si32(0))]
    #[case::small(123usize, si32(123))]
    #[case::at_i32_max(i32::MAX as usize, si32(i32::MAX))]
    #[case::over_i32_max((i32::MAX as usize).saturating_add(1), si32(i32::MAX))] // clamped
    fn from_usize(#[case] src: usize, #[case] want: Si32) {
        assert_eq!(want, Si32::from(src));
    }

    // ------------------------ i64 <-> Si32 (clamp on narrowing) --------------

    #[rstest]
    #[case::zero(0i64, si32(0))]
    #[case::pos(123i64, si32(123))]
    #[case::neg(-456i64, si32(-456))]
    #[case::clamp_low(i64::MIN, si32(i32::MIN))] // clamped
    #[case::clamp_high(i64::MAX, si32(i32::MAX))] // clamped
    fn from_i64(#[case] src: i64, #[case] want: Si32) {
        assert_eq!(want, Si32::from(src));
    }

    #[rstest]
    #[case::zero(si32(0), 0i64)]
    #[case::pos(si32(123), 123i64)]
    #[case::neg(si32(-456), -456i64)]
    #[case::min(si32(i32::MIN), i32::MIN as i64)]
    #[case::max(si32(i32::MAX), i32::MAX as i64)]
    fn into_i64(#[case] src: Si32, #[case] want: i64) {
        let got: i64 = src.into();
        assert_eq!(want, got);
    }

    // ------------------------ TryFrom<f64> -> Si32 ---------------------------
    // Rounds, then clamps to i32 range; errors on NaN / +/-Inf.

    #[rstest]
    #[case::zero(0.0, si32(0))]
    #[case::pos_whole(123.0, si32(123))]
    #[case::neg_whole(-456.0, si32(-456))]
    #[case::round_up(1.6, si32(2))]
    #[case::round_down(-1.4, si32(-1))]
    #[case::clamp_high((i32::MAX as f64) + 10_000.5, si32(i32::MAX))]
    #[case::clamp_low((i32::MIN as f64) - 10_000.5, si32(i32::MIN))]
    fn try_from_f64_ok(#[case] src: f64, #[case] want: Si32) {
        let got = Si32::try_from(src);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap());
    }

    #[rstest]
    #[case::nan(f64::NAN)]
    #[case::pos_inf(f64::INFINITY)]
    #[case::neg_inf(f64::NEG_INFINITY)]
    fn try_from_f64_err(#[case] src: f64) {
        let got = Si32::try_from(src);
        assert!(got.is_err());
    }

    // ------------------------ Si32 -> f64 / f32 ------------------------------

    #[rstest]
    #[case::zero(si32(0), 0.0f64)]
    #[case::pos(si32(123), 123.0f64)]
    #[case::neg(si32(-456), -456.0f64)]
    #[case::min(si32(i32::MIN), i32::MIN as f64)]
    #[case::max(si32(i32::MAX), i32::MAX as f64)]
    fn into_f64(#[case] src: Si32, #[case] want: f64) {
        let got: f64 = src.into();
        assert_eq!(want, got);
    }

    #[rstest]
    #[case::zero(si32(0), 0.0f32)]
    #[case::pos(si32(123), 123.0f32)]
    #[case::neg(si32(-456), -456.0f32)]
    fn into_f32(#[case] src: Si32, #[case] want: f32) {
        let got: f32 = src.into();
        assert_eq!(want, got);
    }

    // --- IntoJs (Si32 -> JS number) -----------------------------------------

    #[rstest]
    #[case::zero(si32(0), 0.0)]
    #[case::pos(si32(123), 123.0)]
    #[case::neg(si32(-456), -456.0)]
    #[case::min(si32(i32::MIN), i32::MIN as f64)]
    #[case::max(si32(i32::MAX), i32::MAX as f64)]
    fn si32_into_js_number(#[case] src: Si32, #[case] want_f64: f64) {
        with_ctx(|ctx| {
            // Si32 -> JS Value (number)
            let v = src.into_js(&ctx).expect("into_js");
            // Read back as f64 to verify numeric content
            let got: f64 = v.get()?;
            assert_eq!(want_f64, got);
            Ok(())
        });
    }

    // --- FromJs (JS number -> Si32) : OK paths (rounding + clamping) --------

    #[rstest]
    #[case::zero(0.0, si32(0))]
    #[case::pos_whole(123.0, si32(123))]
    #[case::neg_whole(-456.0, si32(-456))]
    #[case::round_up(1.6, si32(2))]
    #[case::round_down(-1.4, si32(-1))]
    #[case::clamp_high((i32::MAX as f64) + 42.9, si32(i32::MAX))]
    #[case::clamp_low((i32::MIN as f64) - 42.9, si32(i32::MIN))]
    fn si32_from_js_number_ok(#[case] num: f64, #[case] want: Si32) {
        with_ctx(|ctx| {
            // Create a JS number Value
            let v = num.into_js(&ctx)?;
            // FromJs -> Si32
            let got = <Si32 as FromJs>::from_js(&ctx, v)?;
            assert_eq!(want, got);
            Ok(())
        });
    }

    // --- FromJs (JS number -> Si32) : error on NaN / ±Infinity ---------------

    #[rstest]
    #[case::nan(f64::NAN)]
    #[case::pos_inf(f64::INFINITY)]
    #[case::neg_inf(f64::NEG_INFINITY)]
    fn si32_from_js_number_err_specials(#[case] num: f64) {
        with_ctx(|ctx| {
            let v = num.into_js(&ctx)?;
            let got = <Si32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }

    // --- FromJs (non-number values) : must error -----------------------------

    #[rstest]
    fn si32_from_js_string_err() {
        with_ctx(|ctx| {
            let v = "123".into_js(&ctx)?; // JS string, not number
            let got = <Si32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }

    #[rstest]
    fn si32_from_js_bool_err() {
        with_ctx(|ctx| {
            let v = true.into_js(&ctx)?; // JS boolean
            let got = <Si32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }

    #[rstest]
    fn si32_from_js_null_err() {
        with_ctx(|ctx| {
            let v = Null.into_js(&ctx)?; // JS null
            let got = <Si32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }

    #[rstest]
    fn si32_from_js_undefined_err() {
        with_ctx(|ctx| {
            let v = Undefined.into_js(&ctx)?; // JS undefined
            let got = <Si32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }

    #[rstest]
    fn si32_from_js_object_err() {
        with_ctx(|ctx| {
            // Easiest way: evaluate a JS expression to get an object Value.
            let v: Value = ctx.eval("{}")?;
            let got = <Si32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }

    #[rstest]
    fn si32_from_js_array_err() {
        with_ctx(|ctx| {
            let v: Value = ctx.eval("[1,2,3]")?;
            let got = <Si32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }
}
