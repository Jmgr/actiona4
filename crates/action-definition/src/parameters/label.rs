use const_default::ConstDefault;
use macros::Parameter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Label(String);

impl Label {
    pub fn new(label: impl Into<String>) -> Self {
        Self(label.into())
    }

    #[must_use]
    pub fn inner(&self) -> &str {
        &self.0
    }
}

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = Label)]
pub struct LabelParameter;
