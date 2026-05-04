use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::{
    point::{Point, point},
    size::Size,
};

impl Add<Size> for Point {
    type Output = Self;

    fn add(self, rhs: Size) -> Self::Output {
        point(
            self.x + rhs.width.to_signed(),
            self.y + rhs.height.to_signed(),
        )
    }
}

impl AddAssign<Size> for Point {
    fn add_assign(&mut self, rhs: Size) {
        *self = point(
            self.x + rhs.width.to_signed(),
            self.y + rhs.height.to_signed(),
        );
    }
}

impl Sub<Size> for Point {
    type Output = Self;

    fn sub(self, rhs: Size) -> Self::Output {
        point(
            self.x - rhs.width.to_signed(),
            self.y - rhs.height.to_signed(),
        )
    }
}

impl SubAssign<Size> for Point {
    fn sub_assign(&mut self, rhs: Size) {
        *self = point(
            self.x - rhs.width.to_signed(),
            self.y - rhs.height.to_signed(),
        );
    }
}
