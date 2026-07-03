use const_default::ConstDefault;
use macros::Parameter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SourceCode(String);

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = SourceCode)]
pub struct SourceParameter;
