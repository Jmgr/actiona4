use const_default::ConstDefault;
use macros::Parameter;
use serde::{Deserialize, Serialize};

/// A JavaScript expression that must evaluate to an array.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Array(String);

impl Array {
    pub fn new(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    pub fn inner(&self) -> &str {
        &self.0
    }
}

impl Default for Array {
    fn default() -> Self {
        Self::new("[]")
    }
}

impl From<String> for Array {
    fn from(source: String) -> Self {
        Self::new(source)
    }
}

impl From<&str> for Array {
    fn from(source: &str) -> Self {
        Self::new(source)
    }
}

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = Array)]
pub struct ArrayParameter;
