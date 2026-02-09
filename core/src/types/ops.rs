use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::types::{si32::Si32, su32::Su32};

impl Add<Su32> for Si32 {
    type Output = Self;

    fn add(self, rhs: Su32) -> Self::Output {
        let lhs: i64 = self.into();
        let rhs: i64 = rhs.into();
        lhs.saturating_add(rhs).into()
    }
}

impl AddAssign<Su32> for Si32 {
    fn add_assign(&mut self, rhs: Su32) {
        let lhs: i64 = (*self).into();
        let rhs: i64 = rhs.into();
        *self = lhs.saturating_add(rhs).into();
    }
}

impl Sub<Su32> for Si32 {
    type Output = Self;

    fn sub(self, rhs: Su32) -> Self::Output {
        let lhs: i64 = self.into();
        let rhs: i64 = rhs.into();
        lhs.saturating_sub(rhs).into()
    }
}

impl SubAssign<Su32> for Si32 {
    fn sub_assign(&mut self, rhs: Su32) {
        let lhs: i64 = (*self).into();
        let rhs: i64 = rhs.into();
        *self = lhs.saturating_sub(rhs).into();
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::types::{
        si32::{Si32, si32},
        su32::{Su32, su32},
    };

    // ------------------------ Add (Si32 + Su32) ------------------------------

    #[rstest]
    #[case::basic_pos(si32(10), su32(5), si32(15))]
    #[case::neg_plus_pos_stays_neg(si32(-5), su32(3), si32(-2))]
    #[case::cross_zero(si32(-2), su32(5), si32(3))]
    #[case::saturate_high_edge(si32(i32::MAX), su32(1), si32(i32::MAX))] // clamp to i32::MAX
    #[case::saturate_high_far(si32(i32::MAX), su32(u32::MAX), si32(i32::MAX))]
    fn add_mixed(#[case] a: Si32, #[case] b: Su32, #[case] want: Si32) {
        assert_eq!(want, a + b);
    }

    #[rstest]
    #[case::assign_basic_pos(10, 5, 15)]
    #[case::assign_neg_plus_pos_stays_neg(-5, 3, -2)]
    #[case::assign_cross_zero(-2, 5, 3)]
    #[case::assign_saturate_high_edge(i32::MAX, 1, i32::MAX)]
    #[case::assign_saturate_high_far(i32::MAX, u32::MAX, i32::MAX)]
    fn add_assign_mixed(#[case] a: i32, #[case] b: u32, #[case] want: i32) {
        let mut x = si32(a);
        x += su32(b);
        assert_eq!(want, x.into_inner());
    }

    // ------------------------ Sub (Si32 - Su32) ------------------------------

    #[rstest]
    #[case::basic_pos(si32(10), su32(5), si32(5))]
    #[case::neg_minus_pos_more_neg(si32(-5), su32(3), si32(-8))]
    #[case::cross_zero(si32(2), su32(5), si32(-3))]
    #[case::saturate_low_edge(si32(i32::MIN), su32(1), si32(i32::MIN))] // clamp to i32::MIN
    #[case::saturate_low_far(si32(-10), su32(u32::MAX), si32(i32::MIN))]
    fn sub_mixed(#[case] a: Si32, #[case] b: Su32, #[case] want: Si32) {
        assert_eq!(want, a - b);
    }

    #[rstest]
    #[case::assign_basic_pos(10, 5, 5)]
    #[case::assign_neg_minus_pos_more_neg(-5, 3, -8)]
    #[case::assign_cross_zero(2, 5, -3)]
    #[case::assign_saturate_low_edge(i32::MIN, 1, i32::MIN)]
    #[case::assign_saturate_low_far(-10, u32::MAX, i32::MIN)]
    fn sub_assign_mixed(#[case] a: i32, #[case] b: u32, #[case] want: i32) {
        let mut x = si32(a);
        x -= su32(b);
        assert_eq!(want, x.into_inner());
    }
}
