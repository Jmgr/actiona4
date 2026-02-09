use std::{
    cmp::Ordering,
    num::Saturating,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
};

use color_eyre::{Report, Result, eyre::eyre};

use crate::types::{
    si32::Si32,
    try_traits::{TryDiv, TryDivAssign},
};

impl PartialEq<i32> for Si32 {
    fn eq(&self, rhs: &i32) -> bool {
        self.into_inner() == *rhs
    }
}

impl PartialOrd<i32> for Si32 {
    fn partial_cmp(&self, rhs: &i32) -> Option<Ordering> {
        self.into_inner().partial_cmp(rhs)
    }
}

impl Add<i32> for Si32 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: i32) -> Self {
        Self(self.0 + Saturating(rhs))
    }
}

impl AddAssign<i32> for Si32 {
    #[inline]
    fn add_assign(&mut self, rhs: i32) {
        self.0 = self.0 + Saturating(rhs);
    }
}

impl Sub<i32> for Si32 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: i32) -> Self {
        Self(self.0 - Saturating(rhs))
    }
}

impl SubAssign<i32> for Si32 {
    #[inline]
    fn sub_assign(&mut self, rhs: i32) {
        self.0 = self.0 - Saturating(rhs);
    }
}

impl Mul<i32> for Si32 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: i32) -> Self {
        Self(self.0 * Saturating(rhs))
    }
}

impl MulAssign<i32> for Si32 {
    #[inline]
    fn mul_assign(&mut self, rhs: i32) {
        self.0 = self.0 * Saturating(rhs);
    }
}

impl TryDiv for Si32 {
    type Output = Self;
    type Error = Report;

    fn try_div(self, rhs: Self) -> Result<Self::Output, Self::Error> {
        if rhs.into_inner() == 0 {
            return Err(eyre!("cannot divide by zero"));
        }
        if self.into_inner() == i32::MIN && rhs.into_inner() == -1 {
            return Err(eyre!("cannot divide {} by -1", i32::MIN));
        }

        Ok(Self(self.0 / rhs.0))
    }
}

impl TryDivAssign for Si32 {
    type Error = Report;

    fn try_div_assign(&mut self, rhs: Self) -> Result<(), Self::Error> {
        if rhs.into_inner() == 0 {
            return Err(eyre!("cannot divide by zero"));
        }
        if self.into_inner() == i32::MIN && rhs.into_inner() == -1 {
            return Err(eyre!("cannot divide {} by -1", i32::MIN));
        }

        self.0 /= rhs.0;

        Ok(())
    }
}

impl TryDiv<i32> for Si32 {
    type Output = Self;
    type Error = Report;

    fn try_div(self, rhs: i32) -> Result<Self::Output, Self::Error> {
        if rhs == 0 {
            return Err(eyre!("cannot divide by zero"));
        }
        if self.into_inner() == i32::MIN && rhs == -1 {
            return Err(eyre!("cannot divide {} by -1", i32::MIN));
        }

        Ok(Self(self.0 / Saturating(rhs)))
    }
}

impl TryDivAssign<i32> for Si32 {
    type Error = Report;

    fn try_div_assign(&mut self, rhs: i32) -> Result<(), Self::Error> {
        if rhs == 0 {
            return Err(eyre!("cannot divide by zero"));
        }
        if self.into_inner() == i32::MIN && rhs == -1 {
            return Err(eyre!("cannot divide {} by -1", i32::MIN));
        }

        self.0 /= rhs;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::types::{
        si32::{Si32, si32},
        try_traits::{TryDiv, TryDivAssign},
    };

    // --- Eq/Ord --------------------------------------------------------------

    #[rstest]
    #[case::eq_basic(si32(42) == 42, true)]
    #[case::ge_equal(si32(42) >= 42, true)]
    #[case::gt_false(si32(42) > 42, false)]
    #[case::ne_false(si32(42) != 42, false)]
    #[case::lt_neg_true(si32(-1) < 0, true)]
    #[case::le_equal(si32(-1) <= -1, true)]
    #[case::lt_false(si32(0) < -1, false)]
    fn si32_ops_eq_ord(#[case] input: bool, #[case] expected: bool) {
        assert_eq!(expected, input)
    }

    // --- Add/Sub (including saturation) -------------------------------------

    #[rstest]
    #[case::add_basic(si32(42) + 1, si32(43))]
    #[case::sub_basic(si32(42) - 1, si32(41))]
    #[case::add_assign_basic({let mut a = si32(42); a += 1; a}, si32(43))]
    #[case::sub_assign_basic({let mut a = si32(42); a -= 1; a}, si32(41))]
    #[case::add_overflow(si32(i32::MAX) + 1, si32(i32::MAX))]
    #[case::sub_underflow(si32(i32::MIN) - 1, si32(i32::MIN))]
    fn si32_ops_add_sub(#[case] input: Si32, #[case] expected: Si32) {
        assert_eq!(expected, input)
    }

    // --- Mul (including saturation) -----------------------------------------

    #[rstest]
    #[case::mul_basic(si32(10) * 2, si32(20))]
    #[case::mul_assign_basic({let mut a = si32(7); a *= 3; a}, si32(21))]
    #[case::mul_overflow(si32(i32::MAX) * 2, si32(i32::MAX))]
    #[case::mul_underflow(si32(i32::MIN) * 2, si32(i32::MIN))]
    #[case::mul_neg_overflow(si32(-1) * i32::MIN, -si32(i32::MIN))]
    fn si32_ops_mul(#[case] input: Si32, #[case] expected: Si32) {
        assert_eq!(expected, input)
    }

    // --- TryDiv with Si32 RHS -----------------------------------------------

    #[rstest]
    #[case::div_simple(si32(6), si32(3), si32(2))]
    #[case::div_negative(si32(-6), si32(3), si32(-2))]
    #[case::div_trunc(si32(7), si32(3), si32(2))]
    fn si32_try_div_self_ok(#[case] lhs: Si32, #[case] rhs: Si32, #[case] want: Si32) {
        let got = lhs.try_div(rhs);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap());
    }

    #[rstest]
    #[case::div_zero_err(si32(1), si32(0))]
    #[case::div_min_neg1_err(si32(i32::MIN), si32(-1))]
    fn si32_try_div_self_err(#[case] lhs: Si32, #[case] rhs: Si32) {
        let got = lhs.try_div(rhs);
        assert!(got.is_err());
    }

    // --- TryDivAssign with Si32 RHS -----------------------------------------

    #[rstest]
    #[case::assign_div_simple(si32(6), si32(3), si32(2))]
    #[case::assign_div_negative(si32(-6), si32(3), si32(-2))]
    #[case::assign_div_trunc(si32(7), si32(3), si32(2))]
    fn si32_try_div_assign_self_ok(#[case] start: Si32, #[case] rhs: Si32, #[case] want: Si32) {
        let mut x = start;
        let res = x.try_div_assign(rhs);
        assert!(res.is_ok());
        assert_eq!(want, x);
    }

    #[rstest]
    #[case::assign_div_zero_err(si32(1), si32(0))]
    #[case::assign_div_min_neg1_err(si32(i32::MIN), si32(-1))]
    fn si32_try_div_assign_self_err(#[case] start: Si32, #[case] rhs: Si32) {
        let mut x = start;
        let res = x.try_div_assign(rhs);
        assert!(res.is_err());
    }

    // --- TryDiv with i32 RHS ------------------------------------------------

    #[rstest]
    #[case::div_i32_simple(si32(6), 3, si32(2))]
    #[case::div_i32_negative(si32(-6), 3, si32(-2))]
    #[case::div_i32_trunc(si32(7), 3, si32(2))]
    fn si32_try_div_i32_ok(#[case] lhs: Si32, #[case] rhs: i32, #[case] want: Si32) {
        let got = lhs.try_div(rhs);
        assert!(got.is_ok());
        assert_eq!(want, got.unwrap());
    }

    #[rstest]
    #[case::div_i32_zero_err(si32(1), 0)]
    #[case::div_i32_min_neg1_err(si32(i32::MIN), -1)]
    fn si32_try_div_i32_err(#[case] lhs: Si32, #[case] rhs: i32) {
        let got = lhs.try_div(rhs);
        assert!(got.is_err());
    }

    // --- TryDivAssign with i32 RHS ------------------------------------------

    #[rstest]
    #[case::assign_div_i32_simple(si32(6), 3, si32(2))]
    #[case::assign_div_i32_negative(si32(-6), 3, si32(-2))]
    #[case::assign_div_i32_trunc(si32(7), 3, si32(2))]
    fn si32_try_div_assign_i32_ok(#[case] start: Si32, #[case] rhs: i32, #[case] want: Si32) {
        let mut x = start;
        let res = x.try_div_assign(rhs);
        assert!(res.is_ok());
        assert_eq!(want, x);
    }

    #[rstest]
    #[case::assign_div_i32_zero_err(si32(1), 0)]
    #[case::assign_div_i32_min_neg1_err(si32(i32::MIN), -1)]
    fn si32_try_div_assign_i32_err(#[case] start: Si32, #[case] rhs: i32) {
        let mut x = start;
        let res = x.try_div_assign(rhs);
        assert!(res.is_err());
    }
}
