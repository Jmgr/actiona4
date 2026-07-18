use std::{
    fmt::{self, Display},
    num::NonZero,
    ops::{Div, DivAssign, Mul, MulAssign},
    result::Result as StdResult,
};

use color_eyre::Result;
use derive_more::{Add, Constructor, Mul, MulAssign};
use satint::{DivError, SaturatingInto, Su32, TryDiv, TryDivAssign};
use serde::{Deserialize, Serialize};

use crate::display::DisplayFields;

#[derive(
    Add,
    Clone,
    Constructor,
    Copy,
    Debug,
    Default,
    Deserialize,
    Eq,
    Mul,
    MulAssign,
    PartialEq,
    Serialize,
    derive_more::Sub,
    derive_more::SubAssign,
)]
#[mul(forward)]
#[mul_assign(forward)]
pub struct Size {
    pub width: Su32,
    pub height: Su32,
}

#[must_use]
pub fn size<W: SaturatingInto<Su32>, H: SaturatingInto<Su32>>(width: W, height: H) -> Size {
    Size::new(width.saturating_into(), height.saturating_into())
}

impl Mul<u32> for Size {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self {
        Self::new(self.width * rhs, self.height * rhs)
    }
}

impl MulAssign<u32> for Size {
    fn mul_assign(&mut self, rhs: u32) {
        self.width *= rhs;
        self.height *= rhs;
    }
}

impl Div<NonZero<u32>> for Size {
    type Output = Self;

    fn div(self, rhs: NonZero<u32>) -> Self::Output {
        let width = self.width / rhs;
        let height = self.height / rhs;
        size(width, height)
    }
}

impl DivAssign<NonZero<u32>> for Size {
    fn div_assign(&mut self, rhs: NonZero<u32>) {
        self.width /= rhs;
        self.height /= rhs;
    }
}

impl TryDiv<u32> for Size {
    type Output = Self;

    fn try_div(self, rhs: u32) -> StdResult<Self::Output, DivError> {
        Ok(Self::new(
            self.width.try_div(rhs)?,
            self.height.try_div(rhs)?,
        ))
    }
}

impl TryDivAssign<u32> for Size {
    fn try_div_assign(&mut self, rhs: u32) -> StdResult<(), DivError> {
        self.width.try_div_assign(rhs)?;
        self.height.try_div_assign(rhs)?;
        Ok(())
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        DisplayFields::default()
            .display("width", self.width)
            .display("height", self.height)
            .finish(f)
    }
}

impl Size {
    pub fn scaled(&self, factor: f64) -> Result<Self> {
        let (w, h) = self.as_f64();

        Ok(Self {
            width: (w * factor).saturating_into(),
            height: (h * factor).saturating_into(),
        })
    }

    pub(crate) fn as_f64(self) -> (f64, f64) {
        (self.width.into(), self.height.into())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn ctor_and_default() {
        assert_eq!(Size::default(), size(0_u32, 0_u32));
        assert_eq!(size(3_u32, 5_u32), Size::new(3_u32.into(), 5_u32.into()));
    }

    #[test]
    fn mul_and_mul_assign_by_u32() {
        let a = size(3_u32, 4_u32) * 2;
        assert_eq!(a, size(6_u32, 8_u32));

        let mut b = size(3_u32, 4_u32);
        b *= 3;
        assert_eq!(b, size(9_u32, 12_u32));
    }

    #[test]
    fn try_div_and_try_div_assign() {
        let c = size(8_u32, 10_u32).try_div(2).unwrap();
        assert_eq!(c, size(4_u32, 5_u32));

        let mut d = size(9_u32, 12_u32);
        d.try_div_assign(3).unwrap();
        assert_eq!(d, size(3_u32, 4_u32));
    }

    #[test]
    fn try_div_by_zero_errorsize() {
        assert!(size(1_u32, 1_u32).try_div(0).is_err());

        let mut e = size(6_u32, 6_u32);
        let err = e.try_div_assign(0).unwrap_err();
        assert_eq!(
            e,
            size(6_u32, 6_u32),
            "Size mutated on failed division: {err}"
        );
    }

    #[rstest]
    #[case::double(size(3_u32, 4_u32), 2.0, size(6_u32, 8_u32))]
    #[case::zero(size(3_u32, 4_u32), 0.0, size(0_u32, 0_u32))]
    #[case::fraction_exact(size(10_u32, 5_u32), 0.2, size(2_u32, 1_u32))] // exact integers after scaling
    fn scaled_ok(#[case] input: Size, #[case] factor: f64, #[case] want: Size) {
        let got = input.scaled(factor).unwrap();
        assert_eq!(got, want);
    }
}
