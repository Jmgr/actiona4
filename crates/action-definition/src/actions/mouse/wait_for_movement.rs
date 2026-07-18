use macros::action;
use serde::{Deserialize, Serialize};

use crate::actions::{ActionBranches, ParameterAvailability};

/// Waits for mouse cursor movement.
#[action(
    icon = MousePointer2,
    effect = ReadState,
    category = Mouse,
    timeout = true,
    waitable = true
)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[allow(clippy::empty_structs_with_brackets)]
pub struct WaitForMovement {}

impl ActionBranches for WaitForMovement {}

impl ParameterAvailability for WaitForMovement {}
