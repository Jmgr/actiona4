use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::variable::Variable,
    scriptable::Scriptable,
};

/// Stores a randomly selected number from a half-open range.
#[action(icon = CodeXml, effect = TransformData, category = Random)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RandomNumber {
    #[parameter]
    pub minimum: Scriptable<f64>,

    #[parameter]
    pub maximum: Scriptable<f64>,

    #[parameter]
    pub result: Variable,
}

impl Default for RandomNumber {
    fn default() -> Self {
        Self {
            minimum: Scriptable::new_static(0.0).into(),
            maximum: Scriptable::new_static(1.0).into(),
            result: Variable::new("random_number").into(),
        }
    }
}

impl ActionBranches for RandomNumber {}

impl ParameterAvailability for RandomNumber {}
