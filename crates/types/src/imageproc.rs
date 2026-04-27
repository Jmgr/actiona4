use color_eyre::{Report, Result, eyre::eyre};
use imageproc::rect::Rect as ImgRect;

use crate::rect::Rect;

impl TryFrom<Rect> for ImgRect {
    type Error = Report;

    fn try_from(value: Rect) -> Result<Self> {
        if value.size.width == 0 || value.size.height == 0 {
            return Err(eyre!("rectangle must have a non-zero size"));
        }

        Ok(Self::at(value.top_left.x.into(), value.top_left.y.into())
            .of_size(value.size.width.into(), value.size.height.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{point::point, rect::rect, size::size};

    fn r(x: i32, y: i32, w: u32, h: u32) -> Rect {
        rect(point(x, y), size(w, h))
    }

    #[test]
    fn try_from_rejects_zero_dims() {
        assert!(ImgRect::try_from(r(0, 0, 0, 10)).is_err());
        assert!(ImgRect::try_from(r(0, 0, 10, 0)).is_err());
    }

    #[test]
    fn try_from_ok_non_zero() {
        let a = r(1, 2, 3, 4);
        let img = ImgRect::try_from(a).expect("non-zero dims should convert");
        let expected = ImgRect::at(1, 2).of_size(3, 4);
        assert_eq!(img, expected);
    }
}
