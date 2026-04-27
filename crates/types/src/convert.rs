use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::{
    point::{Point, point},
    si32::Si32,
    size::Size,
    su32::Su32,
};

impl From<Su32> for Si32 {
    fn from(value: Su32) -> Self {
        value.into_inner().into()
    }
}

impl From<Si32> for Su32 {
    fn from(value: Si32) -> Self {
        value.into_inner().into()
    }
}

impl Add<Size> for Point {
    type Output = Self;

    fn add(self, rhs: Size) -> Self::Output {
        point(self.x + rhs.width, self.y + rhs.height)
    }
}

impl AddAssign<Size> for Point {
    fn add_assign(&mut self, rhs: Size) {
        *self = point(self.x + rhs.width, self.y + rhs.height);
    }
}

impl Sub<Size> for Point {
    type Output = Self;

    fn sub(self, rhs: Size) -> Self::Output {
        point(self.x - rhs.width, self.y - rhs.height)
    }
}

impl SubAssign<Size> for Point {
    fn sub_assign(&mut self, rhs: Size) {
        *self = point(self.x - rhs.width, self.y - rhs.height);
    }
}

#[cfg(test)]
#[allow(clippy::as_conversions)]
mod tests {
    use rstest::rstest;

    use crate::{
        si32::{Si32, si32},
        su32::{Su32, su32},
    };

    // ---------------- Su32 -> Si32 (via u32 -> Si32) -------------------------
    // Expect: clamp to i32::MAX if Su32 value exceeds i32 range.

    #[rstest]
    #[case::zero(su32(0), si32(0))]
    #[case::small(su32(123), si32(123))]
    #[case::at_i32_max(su32(i32::MAX as u32), si32(i32::MAX))]
    #[case::over_i32_max(su32((i32::MAX as u32) + 1), si32(i32::MAX))] // clamped
    #[case::u32_max(su32(u32::MAX), si32(i32::MAX))] // clamped
    fn from_su32_to_si32(#[case] src: Su32, #[case] want: Si32) {
        let got: Si32 = src.into();
        assert_eq!(want, got);
    }

    // ---------------- Si32 -> Su32 (via i32 -> Su32) -------------------------
    // Expect: negatives clamp to 0; others unchanged (up to i32::MAX).

    #[rstest]
    #[case::neg_to_zero(si32(-1), su32(0))] // clamped
    #[case::min_to_zero(si32(i32::MIN), su32(0))] // clamped
    #[case::zero(si32(0), su32(0))]
    #[case::small(si32(123), su32(123))]
    #[case::at_i32_max(si32(i32::MAX), su32(i32::MAX as u32))]
    fn from_si32_to_su32(#[case] src: Si32, #[case] want: Su32) {
        let got: Su32 = src.into();
        assert_eq!(want, got);
    }

    // ---------------- Round-trip sanity checks -------------------------------

    // For all u32 in range [0, i32::MAX], Su32 -> Si32 -> Su32 should be identity.
    #[rstest]
    #[case::zero(0u32)]
    #[case::one(1u32)]
    #[case::mid(123456u32)]
    #[case::edge(i32::MAX as u32)]
    fn roundtrip_su32_si32_su32_in_range(#[case] x: u32) {
        let a = su32(x);
        let b: Si32 = a.into();
        let c: Su32 = b.into();
        assert_eq!(a, c);
    }

    // For negative Si32, Si32 -> Su32 -> Si32 yields 0.
    #[rstest]
    #[case::neg_small(-1)]
    #[case::neg_large(-123456)]
    #[case::min(i32::MIN)]
    fn roundtrip_si32_su32_si32_negative(#[case] x: i32) {
        let a = si32(x);
        let b: Su32 = a.into(); // clamped to 0
        let c: Si32 = b.into(); // 0 stays 0
        assert_eq!(si32(0), c);
    }

    // For non-negative Si32 within [0, i32::MAX], round-trip preserves value.
    #[rstest]
    #[case::zero(0)]
    #[case::small(123)]
    #[case::edge(i32::MAX)]
    fn roundtrip_si32_su32_si32_nonneg(#[case] x: i32) {
        let a = si32(x);
        let b: Su32 = a.into();
        let c: Si32 = b.into();
        assert_eq!(a, c);
    }
}
