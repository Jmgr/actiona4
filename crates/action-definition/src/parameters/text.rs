use const_default::ConstDefault;
use macros::Parameter;

use crate::scriptable::Scriptable;

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = Scriptable<String>)]
pub struct TextParameter {
    pub max_length: Option<u64>,
}
