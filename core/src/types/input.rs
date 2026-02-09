use strum::EnumIs;

#[derive(Clone, Copy, Debug, EnumIs, Eq, PartialEq)]
pub enum Direction {
    Press,
    Release,
}
