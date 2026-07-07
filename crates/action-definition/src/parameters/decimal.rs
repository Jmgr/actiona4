use const_default::ConstDefault;
use macros::Parameter;

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = f64)]
pub struct DecimalParameter {
    pub min: Option<f64>,
    pub max: Option<f64>,
}
