use const_default::ConstDefault;
use macros::Parameter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Variable(String);

impl Variable {
    pub fn new(variable: impl Into<String>) -> Self {
        Self(variable.into())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = Variable)]
pub struct VariableParameter;
