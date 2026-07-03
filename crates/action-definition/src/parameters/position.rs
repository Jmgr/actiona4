use const_default::ConstDefault;
use macros::Parameter;
use types::Point;

use crate::scriptable::Scriptable;

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = Scriptable<Point>)]
pub struct PositionParameter;
