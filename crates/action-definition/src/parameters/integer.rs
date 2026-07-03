use const_default::ConstDefault;
use macros::Parameter;

use crate::scriptable::Scriptable;

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = Scriptable<i64>)]
pub struct IntegerParameter {
    pub min: Option<i64>,
    pub max: Option<i64>,
}
