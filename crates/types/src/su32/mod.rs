//! Integer math in Rust has some characteristics that are not ideal for us:
//! * it's wrapping instead of saturating
//! * it panics when a division by 0 occurs
//!
//! One could use saturating_add and others but it's manual and tedious.
//!
//! std::num::Saturating is a good start but it still panics if a division by zero happens,
//! so this wraps it and adds the missing bits.

use std::num::Saturating;

use color_eyre::{Report, Result};
use derive_more::{Add, AddAssign, Display, Mul, MulAssign, Sub, SubAssign};
use serde::{Deserialize, Serialize};

pub use crate::try_traits::{TryDiv, TryDivAssign};

pub mod convert;
pub mod ops;

/// Safe saturating and non-panicking integer
#[derive(
    Add,
    AddAssign,
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    Eq,
    Hash,
    Mul,
    MulAssign,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
    Sub,
    SubAssign,
)]
#[repr(transparent)]
#[mul(forward)]
#[mul_assign(forward)]
pub struct Su32(Saturating<u32>);

pub fn su32<T: Into<Su32>>(value: T) -> Su32 {
    value.into()
}

pub fn try_su32<T: TryInto<Su32, Error = Report>>(value: T) -> Result<Su32> {
    value.try_into()
}

impl Su32 {
    pub const ZERO: Self = Self(Saturating(0));

    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(Saturating(value))
    }

    #[must_use]
    pub const fn into_inner(self) -> u32 {
        self.0.0
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::su32::{Su32, su32, try_su32};

    // ---------- Constructors / constants / accessors -------------------------

    #[test]
    fn zero_const_and_inner() {
        let z = Su32::ZERO;
        assert_eq!(0, z.into_inner());
        assert_eq!(Su32::new(0), z);
    }

    #[rstest]
    #[case::zero(0, 0)]
    #[case::small(123, 123)]
    #[case::max(u32::MAX, u32::MAX)]
    fn new_and_inner_roundtrip(#[case] src: u32, #[case] want: u32) {
        let s = Su32::new(src);
        assert_eq!(want, s.into_inner());
    }

    // su32() generic constructor (uses Into<Su32> impls from `convert`)
    #[rstest]
    #[case::from_u32(123u32, Su32::new(123))]
    #[case::from_i32_neg(-5i32, Su32::new(0))] // clamped
    #[case::from_i32_pos(5i32, Su32::new(5))]
    #[case::from_u16(42u16, Su32::new(42))]
    #[case::from_usize(77usize, Su32::new(77))]
    #[case::from_i64_high(i64::MAX, Su32::new(u32::MAX))] // clamped
    fn su32_ctor_into(#[case] src: impl Into<Su32>, #[case] want: Su32) {
        assert_eq!(want, su32(src));
    }

    // try_su32() (uses TryInto<Su32> impls from `convert`)
    #[rstest]
    #[case::u64_ok(123u64, Su32::new(123))]
    #[case::u64_clamp(u64::MAX, Su32::new(u32::MAX))] // Ok + clamped
    #[case::f64_ok_clamped(1.6f64, Su32::new(1))]
    #[case::f64_low_clamped(-42.9f64, Su32::new(0))]
    fn try_su32_ok(
        #[case] src: impl TryInto<Su32, Error = color_eyre::Report>,
        #[case] want: Su32,
    ) {
        let got = try_su32(src);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap());
    }

    #[rstest]
    #[case::nan(f64::NAN)]
    #[case::pos_inf(f64::INFINITY)]
    #[case::neg_inf(f64::NEG_INFINITY)]
    fn try_su32_err(#[case] src: f64) {
        let got = try_su32(src);
        assert!(got.is_err());
    }

    // ---------- Ordering / Eq / Ord (derive) --------------------------------

    #[test]
    fn ordering_and_sort() {
        let mut v = vec![su32(5u32), su32(1u32), su32(3u32)];
        v.sort(); // uses Ord derive
        let got: Vec<u32> = v.into_iter().map(|x| x.into_inner()).collect();
        assert_eq!(vec![1, 3, 5], got);
    }

    // ---------- Arithmetic (derive_more + Saturating<T>) ---------------------
    // NOTE: inner is Saturating<u32>, so ops are saturating (no panic).

    #[rstest]
    #[case::add_basic(su32(10), su32(5), su32(15))]
    #[case::add_overflow(su32(u32::MAX), su32(1), su32(u32::MAX))] // saturates
    fn add(#[case] a: Su32, #[case] b: Su32, #[case] want: Su32) {
        assert_eq!(want, a + b);
    }

    #[rstest]
    #[case::add_assign_basic(10u32, 5u32, 15u32)]
    #[case::add_assign_overflow(u32::MAX, 1u32, u32::MAX)]
    fn add_assign(#[case] a: u32, #[case] b: u32, #[case] want: u32) {
        let mut x = su32(a);
        x += su32(b);
        assert_eq!(want, x.into_inner());
    }

    #[rstest]
    #[case::sub_basic(su32(10), su32(5), su32(5))]
    #[case::sub_underflow(su32(0), su32(1), su32(0))] // saturates to 0
    fn sub(#[case] a: Su32, #[case] b: Su32, #[case] want: Su32) {
        assert_eq!(want, a - b);
    }

    #[rstest]
    #[case::sub_assign_basic(10u32, 5u32, 5u32)]
    #[case::sub_assign_underflow(0u32, 1u32, 0u32)]
    fn sub_assign(#[case] a: u32, #[case] b: u32, #[case] want: u32) {
        let mut x = su32(a);
        x -= su32(b);
        assert_eq!(want, x.into_inner());
    }

    #[rstest]
    #[case::mul_basic(su32(7), su32(3), su32(21))]
    #[case::mul_overflow(su32(u32::MAX), su32(2), su32(u32::MAX))] // saturates
    fn mul(#[case] a: Su32, #[case] b: Su32, #[case] want: Su32) {
        assert_eq!(want, a * b);
    }

    #[rstest]
    #[case::mul_assign_basic(7u32, 3u32, 21u32)]
    #[case::mul_assign_overflow(u32::MAX, 2u32, u32::MAX)]
    fn mul_assign(#[case] a: u32, #[case] b: u32, #[case] want: u32) {
        let mut x = su32(a);
        x *= su32(b);
        assert_eq!(want, x.into_inner());
    }

    // ---------- Display (derive_more::Display) -------------------------------

    #[rstest]
    #[case::zero(su32(0), "0")]
    #[case::small(su32(123), "123")]
    #[case::max(su32(u32::MAX), &u32::MAX.to_string())]
    fn display(#[case] v: Su32, #[case] want: &str) {
        assert_eq!(want, v.to_string());
    }
}
