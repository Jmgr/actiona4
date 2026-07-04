use const_default::ConstDefault;
use macros::Parameter;

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = String)]
pub struct TextParameter {
    pub max_length: Option<u64>,
}
