use std::fmt::Display;

use color_eyre::{Result, eyre::eyre};
use itertools::Itertools;
use strum::{Display, EnumDiscriminants, EnumIs};

use crate::input::Comments;

#[derive(Debug)]
pub struct EnumVariant {
    pub name: String,
    pub comments: Comments,
    pub platforms: Platforms,
}

#[derive(Debug)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub comments: Comments,
    pub platforms: Platforms,
    pub verbatim: Vec<String>,
    pub default_value: Option<String>,
}

#[derive(Clone, Debug, EnumIs, PartialEq)]
pub enum Type {
    /// Void, "none" type
    Void,

    /// Boolean
    Bool,

    /// Number, can be decimal or integer
    Number,

    /// This, or "self" in Rust
    This,

    /// A parameter that should be ignored, like "Ctx" (rquickjs's context)
    Ignore,

    /// Unknown type, i.e. has to be defined in the rustdoc
    Unknown,

    /// String type, either "String" or "str"
    String,

    /// Optional type
    Option(Box<Self>),

    /// Direct TS type, without any conversion
    Verbatim(String),

    /// Array
    Array(Box<Self>),

    /// Record
    Record(Box<Self>, Box<Self>),
}

#[derive(Clone, Copy, Debug)]
pub enum Context {
    Variable,
    Property,
    ReturnValue,
}

#[derive(Clone, Copy, Debug, Display, PartialEq)]
pub enum RustdocContext {
    StructAlias,
    Struct,
    Method,
    MethodOverload,
    Enum,
    EnumVariant,
    Property,
    Module,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    pub name: String,
    pub type_: Type,
    pub comments: Comments,
    pub is_readonly: bool,
    pub is_readonly_type: bool,
    pub default_value: Option<String>,
    pub platforms: Platforms,
    pub is_promise: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Method {
    pub name: String,
    pub overloads: Vec<MethodOverload>,
    pub is_constructor: bool,
    pub is_private: bool,
    pub is_static: bool,
    pub is_async: bool,
    pub is_generic: bool,
}

#[derive(Clone, Debug)]
pub struct RestParams {
    pub type_: Option<String>,
}

#[derive(Clone, Debug)]
pub struct MethodOverload {
    pub parameters: Vec<Variable>,
    pub return_: Type,
    pub is_readonly_type: bool,
    pub comments: Comments,
    pub rest_params: Option<RestParams>,
    pub platforms: Platforms,
    pub constructor_only: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Const {
    pub value: String,
    pub comments: Comments,
}

#[derive(Clone, Debug, Default)]
pub struct Struct {
    pub name: String,
    pub properties: Vec<Variable>,
    pub methods: Vec<Method>,
    pub comments: Comments,
    pub is_singleton: bool,
    pub consts: Vec<Const>,
    pub is_options: bool,
    pub extends: Option<String>,
    pub platforms: Platforms,
    pub is_generic: bool,
    pub extra_methods: Vec<String>,
    pub verbatim: Vec<String>,
}

#[derive(Debug, Default)]
pub struct File {
    pub enums: Vec<Enum>,
    pub structs: Vec<Struct>,
    pub functions: Vec<Method>,
    pub modules: Vec<Module>,
}

#[derive(Clone, Debug, Default)]
pub struct Module {
    pub verbatim: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlatformConstraint {
    Only,
    Not,
}

impl TryFrom<char> for PlatformConstraint {
    type Error = color_eyre::Report;

    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            '=' => Self::Only,
            '-' => Self::Not,
            _ => return Err(eyre!("unknown platform constraint character: {value}")),
        })
    }
}

#[derive(Clone, Debug, Display, PartialEq)]
pub enum PlatformType {
    Linux,
    Windows,
    X11,
    Wayland,
}

impl TryFrom<&str> for PlatformType {
    type Error = color_eyre::Report;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Ok(match value {
            "linux" => Self::Linux,
            "windows" => Self::Windows,
            "x11" => Self::X11,
            "wayland" => Self::Wayland,
            _ => return Err(eyre!("unknown platform: {value}")),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Platform {
    pub constraint: PlatformConstraint,
    pub type_: PlatformType,
}

impl TryFrom<&str> for Platform {
    type Error = color_eyre::Report;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let mut chars = value.chars();
        let constraint = chars
            .next()
            .ok_or(eyre!("unexpected empty platform string"))?;

        Ok(Self {
            constraint: constraint.try_into()?,
            type_: chars.as_str().try_into()?,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Platforms(Vec<Platform>);

impl TryFrom<&str> for Platforms {
    type Error = color_eyre::Report;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        let platforms = value
            .split_whitespace()
            .map(Platform::try_from)
            .collect::<Result<Vec<_>>>()?;

        if platforms.is_empty() {
            return Ok(Self::default());
        }

        Ok(Self(platforms))
    }
}

impl Display for Platforms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = Vec::new();

        let only_platforms = self.to_string_with_constraint(PlatformConstraint::Only);
        let not_platforms = self.to_string_with_constraint(PlatformConstraint::Not);

        if !only_platforms.is_empty() {
            result.push(format!("only works on {only_platforms}"));
        }
        if !not_platforms.is_empty() {
            result.push(format!("does not work on {not_platforms}"));
        }

        write!(f, "{}", result.join(", "))
    }
}

impl Platforms {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn to_string_with_constraint(&self, constraint: PlatformConstraint) -> String {
        self.0
            .iter()
            .filter(|platform| platform.constraint == constraint)
            .map(|platform| platform.type_.to_string())
            .sorted()
            .collect_vec()
            .join(", ")
    }
}

#[derive(Clone, Debug, EnumDiscriminants, EnumIs, PartialEq)]
#[strum_discriminants(derive(Display))]
pub enum Instruction {
    Constructor,
    Private,
    Property(Variable),
    Parameter(Variable),
    Overload,
    Skip,
    Returns(Type),
    Singleton,
    Const(Const),
    Default(String),
    Options,
    Extends(String),
    Rest(Option<String>),
    Rename(String),
    Static,
    Platforms(Platforms),
    Generic,
    Method(String),
    Type(Type),
    Verbatim(String),
    Getter,
    ReadonlyType,
    ConstructorOnly,
}

pub fn strip_modules(name: &str) -> &str {
    // Remove any modules: foo::Bar => Bar
    let parts = name.split("::");
    if let Some(last) = parts.last() {
        last
    } else {
        name
    }
}
