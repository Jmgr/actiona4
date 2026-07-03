use macros::Parameter;

use crate::TranslationKey;

#[derive(Debug)]
pub struct EnumParameterVariant {
    pub id: &'static str,
    pub name: TranslationKey,
}

// NOTE: storage is implemented by the ActionEnum derive macro.
#[derive(Debug, Parameter)]
pub struct EnumParameter {
    pub variants: &'static [EnumParameterVariant],
}
