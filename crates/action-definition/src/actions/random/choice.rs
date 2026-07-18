use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::{Param, array::Array, variable::Variable},
};

/// Stores one randomly selected item from an array.
#[action(icon = CodeXml, effect = TransformData, category = Random)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RandomItem {
    #[parameter]
    pub array: Array,

    #[parameter]
    pub result: Variable,
}

impl Default for RandomItem {
    fn default() -> Self {
        Self {
            array: Param::default(),
            result: Variable::new("random_item").into(),
        }
    }
}

impl ActionBranches for RandomItem {}

impl ParameterAvailability for RandomItem {}
