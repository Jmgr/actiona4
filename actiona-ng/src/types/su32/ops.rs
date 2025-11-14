use std::{
    cmp::Ordering,
    num::Saturating,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use color_eyre::{Report, Result, eyre::eyre};

use crate::types::{
    su32::Su32,
    try_traits::{TryDiv, TryDivAssign},
};

impl PartialEq<u32> for Su32 {
    fn eq(&self, rhs: &u32) -> bool {
        self.into_inner() == *rhs
    }
}

impl PartialOrd<u32> for Su32 {
    fn partial_cmp(&self, rhs: &u32) -> Option<Ordering> {
        self.into_inner().partial_cmp(rhs)
    }
}

impl Add<u32> for Su32 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: u32) -> Self {
        Self(self.0 + Saturating(rhs))
    }
}

impl AddAssign<u32> for Su32 {
    #[inline]
    fn add_assign(&mut self, rhs: u32) {
        self.0 = self.0 + Saturating(rhs);
    }
}

impl Sub<u32> for Su32 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: u32) -> Self {
        Self(self.0 - Saturating(rhs))
    }
}

impl SubAssign<u32> for Su32 {
    #[inline]
    fn sub_assign(&mut self, rhs: u32) {
        self.0 = self.0 - Saturating(rhs);
    }
}

impl Mul<u32> for Su32 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: u32) -> Self {
        Self(self.0 * Saturating(rhs))
    }
}

impl MulAssign<u32> for Su32 {
    #[inline]
    fn mul_assign(&mut self, rhs: u32) {
        self.0 = self.0 * Saturating(rhs);
    }
}

impl TryDiv for Su32 {
    type Output = Self;
    type Error = Report;

    fn try_div(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        if rhs.into_inner() == 0 {
            return Err(eyre!("cannot divide by zero"));
        }

        Ok(Self(self.0 / rhs.0))
    }
}

impl TryDivAssign for Su32 {
    type Error = Report;

    fn try_div_assign(&mut self, rhs: Self) -> Result<(), Self::Error> {
        if rhs.into_inner() == 0 {
            return Err(eyre!("cannot divide by zero"));
        }

        self.0 /= rhs.0;

        Ok(())
    }
}

impl TryDiv<u32> for Su32 {
    type Output = Self;
    type Error = Report;

    fn try_div(self, rhs: u32) -> Result<Self::Output, Self::Error> {
        if rhs == 0 {
            return Err(eyre!("cannot divide by zero"));
        }

        Ok(Self(self.0 / Saturating(rhs)))
    }
}

impl TryDivAssign<u32> for Su32 {
    type Error = Report;

    fn try_div_assign(&mut self, rhs: u32) -> Result<(), Self::Error> {
        if rhs == 0 {
            return Err(eyre!("cannot divide by zero"));
        }

        self.0 /= rhs;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::as_conversions)]
mod tests {
    use rstest::rstest;

    use crate::types::{
        su32::{Su32, su32},
        try_traits::{TryDiv, TryDivAssign},
    };

    // --- Eq/Ord --------------------------------------------------------------

    #[rstest]
    #[case::eq_basic(su32(42) == 42, true)]
    #[case::ge_equal(su32(42) >= 42, true)]
    #[case::gt_false(su32(42) > 42, false)]
    #[case::ne_false(su32(42) != 42, false)]
    #[case::lt_true(su32(1) < 2, true)]
    #[case::le_equal(su32(5) <= 5, true)]
    #[case::lt_false(su32(10) < 5, false)]
    fn su32_ops_eq_ord(#[case] input: bool, #[case] expected: bool) {
        assert_eq!(expected, input)
    }

    // --- Add/Sub (including saturation) -------------------------------------

    #[rstest]
    #[case::add_basic(su32(42) + 1, su32(43))]
    #[case::sub_basic(su32(42) - 1, su32(41))]
    #[case::add_assign_basic({let mut a = su32(42); a += 1; a}, su32(43))]
    #[case::sub_assign_basic({let mut a = su32(42); a -= 1; a}, su32(41))]
    #[case::add_overflow(su32(u32::MAX) + 1, su32(u32::MAX))]
    #[case::sub_underflow(su32(0) - 1, su32(0))]
    fn su32_ops_add_sub(#[case] input: Su32, #[case] expected: Su32) {
        assert_eq!(expected, input)
    }

    // --- Mul (including saturation) -----------------------------------------

    #[rstest]
    #[case::mul_basic(su32(10) * 2, su32(20))]
    #[case::mul_assign_basic({let mut a = su32(7); a *= 3; a}, su32(21))]
    #[case::mul_overflow(su32(u32::MAX) * 2, su32(u32::MAX))]
    #[case::mul_large(su32(100_000) * 100_000, su32(10_000_000_000u64.min(u32::MAX as u64) as u32))]
    fn su32_ops_mul(#[case] input: Su32, #[case] expected: Su32) {
        assert_eq!(expected, input)
    }

    // --- TryDiv with Su32 RHS -----------------------------------------------

    #[rstest]
    #[case::div_simple(su32(6), su32(3), su32(2))]
    #[case::div_trunc(su32(7), su32(3), su32(2))]
    #[case::div_equal(su32(9), su32(9), su32(1))]
    fn su32_try_div_self_ok(#[case] lhs: Su32, #[case] rhs: Su32, #[case] want: Su32) {
        let got = lhs.try_div(rhs);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap());
    }

    #[rstest]
    #[case::div_zero_err(su32(1), su32(0))]
    fn su32_try_div_self_err(#[case] lhs: Su32, #[case] rhs: Su32) {
        let got = lhs.try_div(rhs);
        assert!(got.is_err());
    }

    // --- TryDivAssign with Su32 RHS -----------------------------------------

    #[rstest]
    #[case::assign_div_simple(su32(6), su32(3), su32(2))]
    #[case::assign_div_trunc(su32(7), su32(3), su32(2))]
    #[case::assign_div_equal(su32(9), su32(9), su32(1))]
    fn su32_try_div_assign_self_ok(#[case] start: Su32, #[case] rhs: Su32, #[case] want: Su32) {
        let mut x = start;
        let res = x.try_div_assign(rhs);
        assert!(res.is_ok());
        assert_eq!(want, x);
    }

    #[rstest]
    #[case::assign_div_zero_err(su32(1), su32(0))]
    fn su32_try_div_assign_self_err(#[case] start: Su32, #[case] rhs: Su32) {
        let mut x = start;
        let res = x.try_div_assign(rhs);
        assert!(res.is_err());
    }

    // --- TryDiv with u32 RHS ------------------------------------------------

    #[rstest]
    #[case::div_u32_simple(su32(6), 3, su32(2))]
    #[case::div_u32_trunc(su32(7), 3, su32(2))]
    #[case::div_u32_equal(su32(9), 9, su32(1))]
    fn su32_try_div_u32_ok(#[case] lhs: Su32, #[case] rhs: u32, #[case] want: Su32) {
        let got = lhs.try_div(rhs);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap());
    }

    #[rstest]
    #[case::div_u32_zero_err(su32(1), 0)]
    fn su32_try_div_u32_err(#[case] lhs: Su32, #[case] rhs: u32) {
        let got = lhs.try_div(rhs);
        assert!(got.is_err());
    }

    // --- TryDivAssign with u32 RHS ------------------------------------------

    #[rstest]
    #[case::assign_div_u32_simple(su32(6), 3, su32(2))]
    #[case::assign_div_u32_trunc(su32(7), 3, su32(2))]
    #[case::assign_div_u32_equal(su32(9), 9, su32(1))]
    fn su32_try_div_assign_u32_ok(#[case] start: Su32, #[case] rhs: u32, #[case] want: Su32) {
        let mut x = start;
        let res = x.try_div_assign(rhs);
        assert!(res.is_ok());
        assert_eq!(want, x);
    }

    #[rstest]
    #[case::assign_div_u32_zero_err(su32(1), 0)]
    fn su32_try_div_assign_u32_err(#[case] start: Su32, #[case] rhs: u32) {
        let mut x = start;
        let res = x.try_div_assign(rhs);
        assert!(res.is_err());
    }
}
