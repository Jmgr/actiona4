use serde::{Deserialize, Serialize};

use crate::{
    actions::{ActionBranches, ParameterAvailability, action},
    parameters::variable::Variable,
    scriptable::Scriptable,
};

/// Stores a random string assembled from a character pool.
#[action(icon = CodeXml, effect = TransformData, category = Random)]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RandomString {
    #[parameter]
    pub length: Scriptable<u32>,

    #[parameter]
    pub characters: Scriptable<String>,

    #[parameter]
    pub result: Variable,
}

impl Default for RandomString {
    fn default() -> Self {
        Self {
            length: Scriptable::new_static(16_u32).into(),
            characters: Scriptable::new_static(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
            )
            .into(),
            result: Variable::new("random_string").into(),
        }
    }
}

impl ActionBranches for RandomString {}

impl ParameterAvailability for RandomString {}
