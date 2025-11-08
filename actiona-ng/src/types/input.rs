use strum::EnumIs;

#[derive(Clone, Copy, PartialEq, Eq, EnumIs, Debug)]
pub enum Direction {
    Press,
    Release,
}
