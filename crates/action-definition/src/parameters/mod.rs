use std::{fmt, marker::PhantomData, ops::Deref};

use serde::{Deserialize, Serialize};
use types::platform::Platforms;

use crate::{
    TranslationKey,
    parameters::{
        boolean::BooleanParameter, decimal::DecimalParameter, duration::DurationParameter,
        enumeration::EnumParameter, integer::IntegerParameter, label::LabelParameter,
        position::PositionParameter, source_code::SourceCodeParameter, text::TextParameter,
        unsigned_integer::UnsignedIntegerParameter, value::ValueParameter,
        variable::VariableParameter,
    },
    scriptable::Scriptable,
};

pub mod boolean;
pub mod decimal;
pub mod duration;
pub mod enumeration;
pub mod integer;
pub mod label;
pub mod position;
pub mod source_code;
pub mod text;
pub mod unsigned_integer;
pub mod value;
pub mod variable;

#[derive(Debug)]
pub enum ParameterKind {
    /// A true-or-false value.
    Boolean(BooleanParameter),
    /// A signed whole number.
    Integer(IntegerParameter),
    /// A point on the screen.
    Position(PositionParameter),
    /// A plain text value.
    Text(TextParameter),
    /// Source code evaluated by the script engine.
    SourceCode(SourceCodeParameter),
    /// One value chosen from a fixed set.
    Enum(EnumParameter),
    /// A span of time.
    Duration(DurationParameter),
    /// A signed decimal number.
    Decimal(DecimalParameter),
    /// A label that identifies an action-tree location.
    Label(LabelParameter),
    /// A non-negative whole number.
    UnsignedInteger(UnsignedIntegerParameter),
    /// A script variable name.
    Variable(VariableParameter),
    /// A dynamically typed script value.
    Value(ValueParameter),
}

#[derive(Debug)]
pub struct Parameter {
    pub id: &'static str,
    pub name: TranslationKey,
    pub description: TranslationKey,
    pub kind: ParameterKind,
    pub platforms: Platforms,
}

pub trait ParameterStorage {
    type Settings;
    const DEFAULT_SETTINGS: Self::Settings;
    const KIND: ParameterKind;
}

impl<T: ParameterStorage> ParameterStorage for Scriptable<T> {
    type Settings = T::Settings;
    const DEFAULT_SETTINGS: Self::Settings = T::DEFAULT_SETTINGS;
    const KIND: ParameterKind = T::KIND;
}

impl<T: ParameterStorage> ParameterStorage for Option<T> {
    type Settings = T::Settings;
    const DEFAULT_SETTINGS: Self::Settings = T::DEFAULT_SETTINGS;
    const KIND: ParameterKind = T::KIND;
}

pub trait ParamName {
    const NAME: &'static str;
}

pub trait ParamSpec: ParamName {
    const KIND: ParameterKind;
}

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct Param<T, N> {
    value: T,
    #[serde(skip)]
    name: PhantomData<N>,
}

impl<T, N> Param<T, N> {
    pub const fn new(value: T) -> Self {
        Self {
            value,
            name: PhantomData,
        }
    }

    pub const fn value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }
}

impl<T, N: ParamName> Param<T, N> {
    pub const fn name(&self) -> &'static str {
        N::NAME
    }
}

impl<T: Copy, N> Copy for Param<T, N> {}

impl<T: Clone, N> Clone for Param<T, N> {
    fn clone(&self) -> Self {
        Self::new(self.value.clone())
    }
}

impl<T: fmt::Debug, N> fmt::Debug for Param<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: Default, N> Default for Param<T, N> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T, N> Deref for Param<T, N> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, N> From<T> for Param<T, N> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T: ParameterStorage, N> ParameterStorage for Param<T, N> {
    type Settings = T::Settings;
    const DEFAULT_SETTINGS: Self::Settings = T::DEFAULT_SETTINGS;
    const KIND: ParameterKind = T::KIND;
}
