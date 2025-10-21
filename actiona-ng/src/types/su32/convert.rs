use std::num::{NonZeroU32, NonZeroUsize, Saturating};

use eyre::{OptionExt, Report, Result, eyre};
use rquickjs::{FromJs, IntoJs};

use crate::{IntoJsResult, types::su32::Su32};

impl From<u32> for Su32 {
    fn from(value: u32) -> Self {
        Self(Saturating(value))
    }
}

impl From<Su32> for u32 {
    fn from(value: Su32) -> Self {
        value.into_inner()
    }
}

impl From<i32> for Su32 {
    #[allow(clippy::as_conversions)]
    fn from(value: i32) -> Self {
        let value = value.clamp(0, i32::MAX);

        Self::new(value as u32)
    }
}

impl From<Su32> for i32 {
    #[allow(clippy::as_conversions)]
    fn from(value: Su32) -> Self {
        let value = value.into_inner().clamp(0, Self::MAX as u32);

        value as Self
    }
}

impl From<usize> for Su32 {
    #[allow(clippy::as_conversions)]
    fn from(value: usize) -> Self {
        let value = value.clamp(0, u32::MAX as usize);

        Self::new(value as u32)
    }
}

impl From<i64> for Su32 {
    fn from(value: i64) -> Self {
        let value = value.clamp(u32::MIN.into(), u32::MAX.into());

        #[allow(clippy::as_conversions)]
        Self::new(value as u32)
    }
}

impl From<Su32> for i64 {
    fn from(value: Su32) -> Self {
        value.into_inner().into()
    }
}

impl From<u16> for Su32 {
    fn from(value: u16) -> Self {
        Self::new(value.into())
    }
}

impl From<Su32> for u16 {
    #[allow(clippy::as_conversions)]
    fn from(value: Su32) -> Self {
        let value = value.into_inner().clamp(Self::MIN.into(), Self::MAX.into());

        value as Self
    }
}

impl TryFrom<u64> for Su32 {
    type Error = Report;

    fn try_from(value: u64) -> Result<Self> {
        let value = value.clamp(u32::MIN.into(), u32::MAX.into());

        #[allow(clippy::as_conversions)]
        Ok(Self::new(value as u32))
    }
}

impl From<Su32> for u64 {
    fn from(value: Su32) -> Self {
        value.into_inner().into()
    }
}

impl TryFrom<f64> for Su32 {
    type Error = Report;

    fn try_from(value: f64) -> Result<Self> {
        if value.is_nan() {
            return Err(eyre!("value is not a number (NaN)"));
        }
        if value.is_infinite() {
            return Err(eyre!("value is infinite"));
        }

        let value = value.clamp(u32::MIN.into(), u32::MAX.into());

        #[allow(clippy::as_conversions)]
        Ok(Self::new(value as u32))
    }
}

impl From<Su32> for f64 {
    fn from(value: Su32) -> Self {
        value.into_inner().into()
    }
}

impl From<NonZeroU32> for Su32 {
    fn from(value: NonZeroU32) -> Self {
        Self::new(value.get())
    }
}

impl TryFrom<Su32> for NonZeroU32 {
    type Error = Report;

    fn try_from(value: Su32) -> Result<Self, Self::Error> {
        Self::new(value.into_inner()).ok_or_eyre("non-zero number expected")
    }
}

impl TryFrom<Su32> for usize {
    type Error = Report;

    fn try_from(value: Su32) -> Result<Self, Self::Error> {
        Ok(Self::try_from(value.into_inner())?)
    }
}

impl TryFrom<Su32> for NonZeroUsize {
    type Error = Report;

    fn try_from(value: Su32) -> Result<Self, Self::Error> {
        Self::new(value.try_into()?).ok_or_eyre("non-zero number expected")
    }
}

impl<'js> IntoJs<'js> for Su32 {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let value: f64 = self.into();

        value.into_js(ctx)
    }
}

impl<'js> FromJs<'js> for Su32 {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let value: f64 = value.get()?;

        IntoJsResult::into_js_result(Self::try_from(value), ctx)
    }
}

#[cfg(test)]
#[allow(clippy::as_conversions)]
mod tests {
    use std::num::{NonZeroU32, NonZeroUsize};

    use rquickjs::{Context, Ctx, FromJs, IntoJs, Null, Runtime, Undefined, Value};
    use rstest::rstest;

    use crate::types::su32::{Su32, su32};

    // Helper: run code inside a JS context
    fn with_ctx<F: FnOnce(Ctx) -> rquickjs::Result<()>>(f: F) {
        let rt = Runtime::new().expect("rt");
        let ctx = Context::full(&rt).expect("ctx");
        ctx.with(f).expect("js ok");
    }

    // ------------------------ From<u32> -> Su32 ------------------------------

    #[rstest]
    #[case::zero(0u32, su32(0))]
    #[case::small(123u32, su32(123))]
    #[case::max(u32::MAX, su32(u32::MAX))]
    fn from_u32(#[case] src: u32, #[case] want: Su32) {
        assert_eq!(want, Su32::from(src));
    }

    // ------------------------ From<Su32> -> u32 ------------------------------

    #[rstest]
    #[case::zero(su32(0), 0u32)]
    #[case::small(su32(123), 123u32)]
    #[case::max(su32(u32::MAX), u32::MAX)]
    fn into_u32(#[case] src: Su32, #[case] want: u32) {
        let got: u32 = src.into();
        assert_eq!(want, got);
    }

    // ------------------------ From<i32> -> Su32 (clamp to 0..=i32::MAX) ------

    #[rstest]
    #[case::neg_to_zero(-1, su32(0))]
    #[case::zero(0, su32(0))]
    #[case::small(123, su32(123))]
    #[case::at_i32_max(i32::MAX, su32(i32::MAX as u32))]
    fn from_i32(#[case] src: i32, #[case] want: Su32) {
        assert_eq!(want, Su32::from(src));
    }

    // ------------------------ From<Su32> -> i32 (clamp to 0..=i32::MAX) ------

    #[rstest]
    #[case::zero(su32(0), 0i32)]
    #[case::small(su32(123), 123i32)]
    #[case::at_i32_max(su32(i32::MAX as u32), i32::MAX)]
    #[case::over_i32_max(su32((i32::MAX as u32) + 1), i32::MAX)] // clamped
    #[case::u32_max_to_i32(su32(u32::MAX), i32::MAX)] // clamped
    fn into_i32(#[case] src: Su32, #[case] want: i32) {
        let got: i32 = src.into();
        assert_eq!(want, got);
    }

    // ------------------------ From<usize> -> Su32 (clamp to u32::MAX) --------

    #[rstest]
    #[case::zero(0usize, su32(0))]
    #[case::small(123usize, su32(123))]
    #[case::at_u32_max(u32::MAX as usize, su32(u32::MAX))]
    #[case::over_u32_max((u32::MAX as usize).saturating_add(1), su32(u32::MAX))] // clamped
    fn from_usize(#[case] src: usize, #[case] want: Su32) {
        assert_eq!(want, Su32::from(src));
    }

    // ------------------------ i64 <-> Su32 (clamp on narrowing) --------------

    #[rstest]
    #[case::zero(0i64, su32(0))]
    #[case::small(123i64, su32(123))]
    #[case::clamp_low(-1i64, su32(0))] // clamped to 0
    #[case::clamp_high(i64::MAX, su32(u32::MAX))] // clamped to u32::MAX
    fn from_i64(#[case] src: i64, #[case] want: Su32) {
        assert_eq!(want, Su32::from(src));
    }

    #[rstest]
    #[case::zero(su32(0), 0i64)]
    #[case::small(su32(123), 123i64)]
    #[case::max(su32(u32::MAX), u32::MAX as i64)]
    fn into_i64(#[case] src: Su32, #[case] want: i64) {
        let got: i64 = src.into();
        assert_eq!(want, got);
    }

    // ------------------------ u16 <-> Su32 (clamp on downcast) ---------------

    #[rstest]
    #[case::zero(0u16, su32(0))]
    #[case::small(123u16, su32(123))]
    #[case::max(u16::MAX, su32(u16::MAX as u32))]
    fn from_u16(#[case] src: u16, #[case] want: Su32) {
        assert_eq!(want, Su32::from(src));
    }

    #[rstest]
    #[case::zero(su32(0), 0u16)]
    #[case::small(su32(123), 123u16)]
    #[case::clamp_high(su32(u32::MAX), u16::MAX)] // clamped down
    fn into_u16(#[case] src: Su32, #[case] want: u16) {
        let got: u16 = src.into();
        assert_eq!(want, got);
    }

    // ------------------------ TryFrom<u64> -> Su32 (clamp, always Ok) --------

    #[rstest]
    #[case::zero(0u64, su32(0))]
    #[case::small(123u64, su32(123))]
    #[case::at_u32_max(u32::MAX as u64, su32(u32::MAX))]
    #[case::over_u32_max((u32::MAX as u64) + 1, su32(u32::MAX))] // clamped, still Ok
    #[case::u64_max(u64::MAX, su32(u32::MAX))] // clamped, still Ok
    fn try_from_u64_ok(#[case] src: u64, #[case] want: Su32) {
        let got = Su32::try_from(src);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap());
    }

    // ------------------------ From<Su32> -> u64 ------------------------------

    #[rstest]
    #[case::zero(su32(0), 0u64)]
    #[case::small(su32(123), 123u64)]
    #[case::max(su32(u32::MAX), u32::MAX as u64)]
    fn into_u64(#[case] src: Su32, #[case] want: u64) {
        let got: u64 = src.into();
        assert_eq!(want, got);
    }

    // ------------------------ TryFrom<f64> -> Su32 ---------------------------
    // Rounds, then clamps to u32 range; errors on NaN / +/-Inf.

    #[rstest]
    #[case::zero(0.0, su32(0))]
    #[case::pos_whole(123.0, su32(123))]
    #[case::round_up(1.6, su32(2))]
    #[case::clamp_low(-42.9, su32(0))]
    #[case::clamp_high((u32::MAX as f64) + 1234.5, su32(u32::MAX))]
    fn try_from_f64_ok(#[case] src: f64, #[case] want: Su32) {
        let got = Su32::try_from(src);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap());
    }

    #[rstest]
    #[case::nan(f64::NAN)]
    #[case::pos_inf(f64::INFINITY)]
    #[case::neg_inf(f64::NEG_INFINITY)]
    fn try_from_f64_err(#[case] src: f64) {
        let got = Su32::try_from(src);
        assert!(got.is_err());
    }

    // ------------------------ NonZeroU32 <-> Su32 ----------------------------

    #[rstest]
    #[case::one(NonZeroU32::new(1).unwrap(), su32(1))]
    #[case::max(NonZeroU32::new(u32::MAX).unwrap(), su32(u32::MAX))]
    fn from_nonzero_u32(#[case] src: NonZeroU32, #[case] want: Su32) {
        assert_eq!(want, Su32::from(src));
    }

    #[rstest]
    #[case::zero_err(su32(0))]
    fn try_into_nonzero_u32_err(#[case] src: Su32) {
        let got = <NonZeroU32 as TryFrom<Su32>>::try_from(src);
        assert!(got.is_err());
    }

    #[rstest]
    #[case::ok(su32(1))]
    fn try_into_nonzero_u32_ok(#[case] src: Su32) {
        let got = <NonZeroU32 as TryFrom<Su32>>::try_from(src);
        assert!(got.is_ok());
        assert_eq!(1, got.unwrap().get());
    }

    // ------------------------ TryFrom<Su32> -> usize -------------------------
    // Use small values to stay portable across targets.

    #[rstest]
    #[case::zero(su32(0), 0usize)]
    #[case::small(su32(123), 123usize)]
    fn try_into_usize_ok(#[case] src: Su32, #[case] want: usize) {
        let got = <usize as TryFrom<Su32>>::try_from(src);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap());
    }

    // ------------------------ TryFrom<Su32> -> NonZeroUsize ------------------

    #[rstest]
    #[case::ok(su32(1), 1usize)]
    fn try_into_nonzero_usize_ok(#[case] src: Su32, #[case] want: usize) {
        let got = <NonZeroUsize as TryFrom<Su32>>::try_from(src);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap().get());
    }

    #[rstest]
    #[case::zero_err(su32(0))]
    fn try_into_nonzero_usize_err(#[case] src: Su32) {
        let got = <NonZeroUsize as TryFrom<Su32>>::try_from(src);
        assert!(got.is_err());
    }

    // --- IntoJs (Su32 -> JS number) -----------------------------------------

    #[rstest]
    #[case::zero(su32(0), 0.0)]
    #[case::small(su32(123), 123.0)]
    #[case::max(su32(u32::MAX), u32::MAX as f64)]
    fn su32_into_js_number(#[case] src: Su32, #[case] want_f64: f64) {
        with_ctx(|ctx| {
            let v = src.into_js(&ctx)?;
            let got: f64 = v.get()?;
            assert_eq!(want_f64, got);
            Ok(())
        });
    }

    // --- FromJs (JS number -> Su32): OK paths (rounding + clamping) ----------

    #[rstest]
    #[case::zero(0.0, su32(0))]
    #[case::whole(123.0, su32(123))]
    #[case::round_up(1.6, su32(2))]
    #[case::clamp_low(-42.9, su32(0))]
    #[case::clamp_high((u32::MAX as f64) + 10_000.0, su32(u32::MAX))]
    fn su32_from_js_number_ok(#[case] num: f64, #[case] want: Su32) {
        with_ctx(|ctx| {
            let v = num.into_js(&ctx)?;
            let got = <Su32 as FromJs>::from_js(&ctx, v)?;
            assert_eq!(want, got);
            Ok(())
        });
    }

    // --- FromJs (JS number -> Su32): error on NaN / ±Infinity ----------------

    #[rstest]
    #[case::nan(f64::NAN)]
    #[case::pos_inf(f64::INFINITY)]
    #[case::neg_inf(f64::NEG_INFINITY)]
    fn su32_from_js_number_err_specials(#[case] num: f64) {
        with_ctx(|ctx| {
            let v = num.into_js(&ctx)?;
            let got = <Su32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }

    // --- FromJs (non-number values): must error ------------------------------

    #[rstest]
    fn su32_from_js_string_err() {
        with_ctx(|ctx| {
            let v = "123".into_js(&ctx)?; // string, not number
            let got = <Su32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }

    #[rstest]
    fn su32_from_js_bool_err() {
        with_ctx(|ctx| {
            let v = true.into_js(&ctx)?; // boolean
            let got = <Su32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }

    #[rstest]
    fn su32_from_js_null_err() {
        with_ctx(|ctx| {
            let v = Null.into_js(&ctx)?; // null
            let got = <Su32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }

    #[rstest]
    fn su32_from_js_undefined_err() {
        with_ctx(|ctx| {
            let v = Undefined.into_js(&ctx)?; // undefined
            let got = <Su32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }

    #[rstest]
    fn su32_from_js_object_err() {
        with_ctx(|ctx| {
            let v: Value = ctx.eval("{}")?; // plain object
            let got = <Su32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }

    #[rstest]
    fn su32_from_js_array_err() {
        with_ctx(|ctx| {
            let v: Value = ctx.eval("[1,2,3]")?; // array
            let got = <Su32 as FromJs>::from_js(&ctx, v);
            assert!(got.is_err());
            Ok(())
        });
    }
}
