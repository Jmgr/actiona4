use const_default::ConstDefault;
use macros::Parameter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Value(String);

impl Value {
    pub fn new(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl From<String> for Value {
    fn from(source: String) -> Self {
        Self::new(source)
    }
}

impl From<&str> for Value {
    fn from(source: &str) -> Self {
        Self::new(source)
    }
}

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = Value)]
pub struct ValueParameter;
