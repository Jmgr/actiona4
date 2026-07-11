use serde::{Deserialize, Serialize};

use crate::actions::{ActionBranches, ParameterAvailability, action};

/// Skips to the next iteration of the nearest enclosing loop.
#[action(icon = CircleArrowRight, effect = ControlFlow, category = Flow)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Continue {}

impl ActionBranches for Continue {}

impl ParameterAvailability for Continue {}
