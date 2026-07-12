use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::variable::Variable,
    scriptable::Scriptable,
};

/// Stores a randomly selected integer from an inclusive range.
#[action(icon = CodeXml, effect = TransformData, category = Random)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RandomInteger {
    #[parameter]
    pub minimum: Scriptable<i64>,

    #[parameter]
    pub maximum: Scriptable<i64>,

    #[parameter]
    pub result: Variable,
}

impl Default for RandomInteger {
    fn default() -> Self {
        Self {
            minimum: Scriptable::new_static(0).into(),
            maximum: Scriptable::new_static(100).into(),
            result: Variable::new("random_integer").into(),
        }
    }
}

impl ActionBranches for RandomInteger {}

impl ParameterAvailability for RandomInteger {}
