use macros::ActionEnum;
use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{Param, label::Label, source_code::SourceCode},
    scriptable::Scriptable,
    tree::BranchKind,
};

/// Marks a location that other flow actions can target.
#[action(icon = MapPin, effect = ControlFlow, category = Flow)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Marker {}

impl ActionBranches for Marker {}

impl ParameterAvailability for Marker {}
