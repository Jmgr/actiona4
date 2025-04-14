use std::{
    fmt::Display,
    ops::{Mul, MulAssign},
};

use color_eyre::{Report, Result};
use derive_more::{Add, Constructor, Mul, MulAssign};
use serde::{Deserialize, Serialize};

use crate::types::{
    display::DisplayFields,
    si32::{TryDiv, TryDivAssign},
    su32::Su32,
};

pub mod js;

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
pub fn size<W: Into<Su32>, H: Into<Su32>>(width: W, height: H) -> Size {
    Size::new(width.into(), height.into())
}

pub fn try_size<W, H>(width: W, height: H) -> Result<Size>
where
    W: TryInto<Su32>,
    H: TryInto<Su32>,
    color_eyre::Report: From<W::Error> + From<H::Error>,
{
    Ok(Size::new(width.try_into()?, height.try_into()?))
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

impl TryDiv<u32> for Size {
    type Output = Self;
    type Error = Report;

    fn try_div(self, rhs: u32) -> std::result::Result<Self::Output, Self::Error> {
        Ok(Self::new(
            self.width.try_div(rhs)?,
            self.height.try_div(rhs)?,
        ))
    }
}

impl TryDivAssign<u32> for Size {
    type Error = Report;

    fn try_div_assign(&mut self, rhs: u32) -> std::result::Result<(), Self::Error> {
        self.width.try_div_assign(rhs)?;
        self.height.try_div_assign(rhs)?;
        Ok(())
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
            width: (w * factor).try_into()?,
            height: (h * factor).try_into()?,
        })
    }

    pub(crate) fn as_f64(&self) -> (f64, f64) {
        (self.width.into(), self.height.into())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn ctor_and_default() {
        assert_eq!(Size::default(), size(0, 0));
        assert_eq!(size(3, 5), Size::new(3u32.into(), 5u32.into()));
    }

    #[rstest]
    #[case::ok_u32(3u32, 5u32, size(3, 5))]
    #[case::ok_mix(7u32, 0u32, size(7, 0))]
    fn try_size_ok<W, H>(#[case] w: W, #[case] h: H, #[case] want: Size)
    where
        W: TryInto<Su32>,
        H: TryInto<Su32>,
        color_eyre::Report: From<W::Error> + From<H::Error>,
    {
        let got = try_size(w, h).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn mul_and_mul_assign_by_u32() {
        let a = size(3, 4) * 2;
        assert_eq!(a, size(6, 8));

        let mut b = size(3, 4);
        b *= 3;
        assert_eq!(b, size(9, 12));
    }

    #[test]
    fn try_div_and_try_div_assign() {
        let c = size(8, 10).try_div(2).unwrap();
        assert_eq!(c, size(4, 5));

        let mut d = size(9, 12);
        d.try_div_assign(3).unwrap();
        assert_eq!(d, size(3, 4));
    }

    #[test]
    fn try_div_by_zero_errorsize() {
        assert!(size(1, 1).try_div(0).is_err());

        let mut e = size(6, 6);
        let err = e.try_div_assign(0).unwrap_err();
        assert_eq!(e, size(6, 6), "Size mutated on failed division: {err}");
    }

    #[rstest]
    #[case::double(size(3, 4), 2.0, size(6, 8))]
    #[case::zero(size(3, 4), 0.0, size(0, 0))]
    #[case::fraction_exact(size(10, 5), 0.2, size(2, 1))] // exact integers after scaling
    fn scaled_ok(#[case] input: Size, #[case] factor: f64, #[case] want: Size) {
        let got = input.scaled(factor).unwrap();
        assert_eq!(got, want);
    }
}
