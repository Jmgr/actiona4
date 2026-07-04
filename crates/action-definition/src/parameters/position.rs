use const_default::ConstDefault;
use macros::Parameter;
use types::Point;

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = Point)]
pub struct PositionParameter;
