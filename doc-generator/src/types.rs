use strum::{Display, EnumDiscriminants};

use crate::input::Comments;

#[derive(Debug)]
pub struct EnumVariant {
    pub name: String,
    pub comments: Comments,
}

#[derive(Debug)]
pub struct Enum {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub comments: Comments,
}

#[derive(Clone, Debug, PartialEq)]
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    pub name: String,
    pub type_: Type,
    pub comments: Comments,
    pub is_readonly: bool,
    pub default_value: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Method {
    pub name: String,
    pub overloads: Vec<MethodOverload>,
    pub is_constructor: bool,
    pub is_static: bool,
}

#[derive(Clone, Debug)]
pub struct MethodOverload {
    pub parameters: Vec<Variable>,
    pub return_: Type,
    pub comments: Comments,
    pub has_rest_params: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Struct {
    pub name: String,
    pub properties: Vec<Variable>,
    pub methods: Vec<Method>,
    pub comments: Comments,
    pub has_global_instance: bool,
    pub consts: Vec<String>,
    pub is_options: bool,
    pub extends: Option<String>,
}

#[derive(Debug, Default)]
pub struct File {
    pub enums: Vec<Enum>,
    pub structs: Vec<Struct>,
}

#[derive(Clone, Debug, EnumDiscriminants, PartialEq)]
#[strum_discriminants(derive(Display))]
pub enum Instruction {
    Constructor,
    Property(Variable),
    Parameter(Variable),
    Overload,
    Skip,
    Returns(Type),
    Global,
    Const(String),
    Default(String),
    Options,
    Extends(String),
    Rest,
    Rename(String),
    Static,
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
