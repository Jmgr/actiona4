use const_default::ConstDefault;
use macros::Parameter;

use crate::scriptable::Scriptable;

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = Scriptable<bool>)]
pub struct BooleanParameter;
