use const_default::ConstDefault;
use macros::Parameter;

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = i64)]
pub struct IntegerParameter {
    pub min: Option<i64>,
    pub max: Option<i64>,
}
