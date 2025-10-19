//! Integer math in Rust has some characteristics that are not ideal for us:
//! * it's wrapping instead of saturating
//! * it panics when a division by 0 occurs
//!
//! One could use saturating_add and others but it's manual and tedious.
//!
//! std::num::Saturating is a good start but it still panics if a division by zero happens,
//! so this wraps it and adds the missing bits.

use std::num::Saturating;

use derive_more::{Add, AddAssign, Display, Mul, MulAssign, Neg, Sub, SubAssign};
use eyre::{Report, Result};
use serde::{Deserialize, Serialize};

use crate::types::su32::Su32;
pub use crate::types::try_traits::{TryDiv, TryDivAssign};

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
    Neg,
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
pub struct Si32(Saturating<i32>);

pub fn si32<T: Into<Si32>>(value: T) -> Si32 {
    value.into()
}

pub fn try_si32<T: TryInto<Si32, Error = Report>>(value: T) -> Result<Si32> {
    value.try_into()
}

impl Si32 {
    pub const ZERO: Self = Self(Saturating(0));

    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self(Saturating(value))
    }

    #[must_use]
    pub const fn into_inner(self) -> i32 {
        self.0.0
    }

    #[must_use]
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }

    #[must_use]
    pub fn unsigned_abs(self) -> Su32 {
        self.abs().into()
    }
}

#[cfg(test)]
#[allow(clippy::as_conversions)]
mod tests {
    use rstest::rstest;

    use crate::types::{
        si32::{Si32, si32, try_si32},
        su32::{Su32, su32},
    };

    // ---------------- Constructors / constants / accessors -------------------

    #[test]
    fn zero_const_and_inner() {
        let z = Si32::ZERO;
        assert_eq!(0, z.into_inner());
        assert_eq!(Si32::new(0), z);
    }

    #[rstest]
    #[case::zero(0, 0)]
    #[case::pos(123, 123)]
    #[case::neg(-456, -456)]
    #[case::min(i32::MIN, i32::MIN)]
    #[case::max(i32::MAX, i32::MAX)]
    fn new_and_inner_roundtrip(#[case] src: i32, #[case] want: i32) {
        let s = Si32::new(src);
        assert_eq!(want, s.into_inner());
    }

    // si32() generic constructor (uses Into<Si32> impls from `convert`)
    #[rstest]
    #[case::from_i32(5i32, Si32::new(5))]
    #[case::from_u32(7u32, Si32::new(7))]
    #[case::from_i16(-9i16, Si32::new(-9))]
    #[case::from_usize(11usize, Si32::new(11))]
    #[case::from_i64_high(i64::MAX, Si32::new(i32::MAX))] // clamped
    fn si32_ctor_into(#[case] src: impl Into<Si32>, #[case] want: Si32) {
        assert_eq!(want, si32(src));
    }

    // try_si32() (uses TryInto<Si32> impls from `convert`)
    #[rstest]
    #[case::f64_ok_rounded(1.6f64, Si32::new(2))]
    #[case::f64_low_clamped((i32::MIN as f64) - 1.0, Si32::new(i32::MIN))]
    #[case::f64_high_clamped((i32::MAX as f64) + 1.0, Si32::new(i32::MAX))]
    fn try_si32_ok(#[case] src: impl TryInto<Si32, Error = eyre::Report>, #[case] want: Si32) {
        let got = try_si32(src);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap());
    }

    #[rstest]
    #[case::nan(f64::NAN)]
    #[case::pos_inf(f64::INFINITY)]
    #[case::neg_inf(f64::NEG_INFINITY)]
    fn try_si32_err(#[case] src: f64) {
        let got = try_si32(src);
        assert!(got.is_err());
    }

    // ---------------- Ordering / Eq / Ord (derive) ---------------------------

    #[test]
    fn ordering_and_sort() {
        let mut v = vec![si32(5), si32(-1), si32(3)];
        v.sort(); // uses Ord derive on Saturating<i32>
        let got: Vec<i32> = v.into_iter().map(|x| x.into_inner()).collect();
        assert_eq!(vec![-1, 3, 5], got);
    }

    // ---------------- Arithmetic (Saturating<i32> via derive_more) -----------

    #[rstest]
    #[case::add_basic(si32(10), si32(5), si32(15))]
    #[case::add_overflow(si32(i32::MAX), si32(1), si32(i32::MAX))] // saturates
    fn add(#[case] a: Si32, #[case] b: Si32, #[case] want: Si32) {
        assert_eq!(want, a + b);
    }

    #[rstest]
    #[case::add_assign_basic(10, 5, 15)]
    #[case::add_assign_overflow(i32::MAX, 1, i32::MAX)]
    fn add_assign(#[case] a: i32, #[case] b: i32, #[case] want: i32) {
        let mut x = si32(a);
        x += si32(b);
        assert_eq!(want, x.into_inner());
    }

    #[rstest]
    #[case::sub_basic(si32(10), si32(5), si32(5))]
    #[case::sub_underflow(si32(i32::MIN), si32(1), si32(i32::MIN))] // saturates
    fn sub(#[case] a: Si32, #[case] b: Si32, #[case] want: Si32) {
        assert_eq!(want, a - b);
    }

    #[rstest]
    #[case::sub_assign_basic(10, 5, 5)]
    #[case::sub_assign_underflow(i32::MIN, 1, i32::MIN)]
    fn sub_assign(#[case] a: i32, #[case] b: i32, #[case] want: i32) {
        let mut x = si32(a);
        x -= si32(b);
        assert_eq!(want, x.into_inner());
    }

    #[rstest]
    #[case::mul_basic(si32(7), si32(3), si32(21))]
    #[case::mul_overflow_pos(si32(i32::MAX), si32(2), si32(i32::MAX))] // saturates
    #[case::mul_overflow_neg(si32(i32::MIN), si32(-2), si32(i32::MAX))] // saturates to MAX
    fn mul(#[case] a: Si32, #[case] b: Si32, #[case] want: Si32) {
        assert_eq!(want, a * b);
    }

    #[rstest]
    #[case::mul_assign_basic(7, 3, 21)]
    #[case::mul_assign_overflow(i32::MAX, 2, i32::MAX)]
    fn mul_assign(#[case] a: i32, #[case] b: i32, #[case] want: i32) {
        let mut x = si32(a);
        x *= si32(b);
        assert_eq!(want, x.into_inner());
    }

    // ---------------- Neg / abs / unsigned_abs -------------------------------

    #[rstest]
    #[case::neg_basic(si32(5), si32(-5))]
    #[case::neg_positive_to_negative(si32(1), si32(-1))]
    #[case::neg_min_to_max(si32(i32::MIN), si32(i32::MAX))] // saturating neg
    fn neg(#[case] x: Si32, #[case] want: Si32) {
        assert_eq!(want, -x);
    }

    #[rstest]
    #[case::abs_pos(si32(5), si32(5))]
    #[case::abs_neg(si32(-5), si32(5))]
    #[case::abs_min_to_max(si32(i32::MIN), si32(i32::MAX))] // saturating abs
    fn abs_(#[case] x: Si32, #[case] want: Si32) {
        assert_eq!(want, x.abs());
    }

    #[rstest]
    #[case::unsigned_abs_pos(si32(7), su32(7))]
    #[case::unsigned_abs_neg(si32(-7), su32(7))]
    #[case::unsigned_abs_min(si32(i32::MIN), su32(i32::MAX as u32))] // abs(MIN)=MAX
    fn unsigned_abs_(#[case] x: Si32, #[case] want: Su32) {
        assert_eq!(want, x.unsigned_abs());
    }

    // ---------------- Display (derive_more::Display) -------------------------

    #[rstest]
    #[case::zero(si32(0), "0")]
    #[case::pos(si32(123), "123")]
    #[case::neg(si32(-456), "-456")]
    fn display(#[case] v: Si32, #[case] want: &str) {
        assert_eq!(want, v.to_string());
    }
}
