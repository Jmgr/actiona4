use crate::{
    TranslationKey,
    parameters::{
        boolean::BooleanParameter, enumeration::EnumParameter, integer::IntegerParameter,
        position::PositionParameter, source_code::SourceParameter, text::TextParameter,
    },
};

pub mod boolean;
pub mod enumeration;
pub mod integer;
pub mod position;
pub mod source_code;
pub mod text;

#[derive(Debug)]
pub enum ParameterKind {
    Boolean(BooleanParameter),
    Integer(IntegerParameter),
    Position(PositionParameter),
    Text(TextParameter),
    Source(SourceParameter),
    Enum(EnumParameter),
}

#[derive(Debug)]
pub struct Parameter {
    pub id: &'static str,
    pub name: TranslationKey,
    pub description: TranslationKey,
    pub kind: ParameterKind,
}

pub trait ParameterStorage {
    type Settings;
    const DEFAULT_SETTINGS: Self::Settings;
    const KIND: ParameterKind;
}
