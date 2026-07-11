use macros::action;
use serde::{Deserialize, Serialize};

use crate::actions::{ActionBranches, ParameterAvailability};

#[action(
    icon = MousePointer2,
    effect = ReadState,
    category = Mouse,
    timeout = true,
    waitable = true
)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WaitForMovement {}

impl ActionBranches for WaitForMovement {}

impl ParameterAvailability for WaitForMovement {}
