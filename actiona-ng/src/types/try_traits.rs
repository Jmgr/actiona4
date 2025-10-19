/// Fallible division
pub trait TryDiv<Rhs = Self> {
    type Output;
    type Error;

    fn try_div(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

/// Fallible assign division
pub trait TryDivAssign<Rhs = Self> {
    type Error;

    fn try_div_assign(&mut self, rhs: Rhs) -> Result<(), Self::Error>;
}
