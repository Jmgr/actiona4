use const_default::ConstDefault;
use macros::Parameter;

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = u32)]
pub struct UnsignedIntegerParameter {
    pub min: Option<u32>,
    pub max: Option<u32>,
}
