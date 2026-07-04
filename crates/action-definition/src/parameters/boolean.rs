use const_default::ConstDefault;
use macros::Parameter;

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = bool)]
pub struct BooleanParameter;
