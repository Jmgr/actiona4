use derive_more::Display;
use serde::{Deserialize, Serialize};
use strum::EnumIs;

#[derive(Clone, Copy, Debug, Deserialize, Display, EnumIs, Eq, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum Direction {
    Press,
    Release,
}
