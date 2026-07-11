use serde::{Deserialize, Serialize};

use crate::actions::{ActionBranches, ParameterAvailability, action};

/// Leaves the nearest enclosing loop.
#[action(icon = CircleOff, effect = ControlFlow, category = Flow)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Break {}

impl ActionBranches for Break {}

impl ParameterAvailability for Break {}
