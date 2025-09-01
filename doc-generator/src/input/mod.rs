use std::mem::take;

use actiona_ng::newtype;
use enums::process_enums;
use eyre::{Result, bail, eyre};
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use rustdoc_types::Crate;
use structs::process_structs;

use crate::{
    input::{functions::process_functions, modules::process_modules},
    items::Items,
    types::{
        File, Instruction, InstructionDiscriminants, Platforms, RestParams, RustdocContext, Type,
        Variable, strip_modules,
    },
};

pub mod enums;
pub mod functions;
pub mod modules;
pub mod structs;

newtype!(pub Comments, Vec<String>);

impl Comments {
    pub fn trimmed(mut self) -> Self {
        // Remove leading empty strings
        while self.first().is_some_and(|s| s.is_empty()) {
            self.remove(0);
        }

        // Remove trailing empty strings
        while self.last().is_some_and(|s| s.is_empty()) {
            self.pop();
        }

        self
    }
}

newtype!(pub Instructions, Vec<Instruction>);

impl Instructions {
    pub fn has_skip(&self) -> bool {
        self.iter()
            .any(|instruction| matches!(instruction, Instruction::Skip))
    }

    pub fn has_constructor(&self) -> bool {
        self.iter()
            .any(|instruction| matches!(instruction, Instruction::Constructor))
    }

    pub fn has_private(&self) -> bool {
        self.iter()
            .any(|instruction| matches!(instruction, Instruction::Private))
    }

    pub fn has_static(&self) -> bool {
        self.iter()
            .any(|instruction| matches!(instruction, Instruction::Static))
    }

    pub fn is_singleton(&self) -> bool {
        self.iter()
            .any(|instruction| matches!(instruction, Instruction::Singleton))
    }

    pub fn is_generic(&self) -> bool {
        self.iter()
            .any(|instruction| matches!(instruction, Instruction::Generic))
    }

    pub fn rest_params(&self) -> Option<RestParams> {
        self.iter().find_map(|instruction| {
            if let Instruction::Rest(type_) = instruction {
                Some(RestParams {
                    type_: type_.clone(),
                })
            } else {
                None
            }
        })
    }

    pub fn returns(&self) -> Option<Type> {
        self.iter().find_map(|instruction| {
            if let Instruction::Returns(type_) = instruction {
                Some(type_.clone())
            } else {
                None
            }
        })
    }

    pub fn type_(&self) -> Option<Type> {
        self.iter().find_map(|instruction| {
            if let Instruction::Type(type_) = instruction {
                Some(type_.clone())
            } else {
                None
            }
        })
    }

    pub fn platforms(&self) -> Platforms {
        self.iter()
            .find_map(|instruction| {
                if let Instruction::Platforms(platforms) = instruction {
                    Some(platforms.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    pub fn is_options(&self) -> bool {
        self.iter()
            .any(|instruction| matches!(instruction, Instruction::Options))
    }

    pub fn extends(&self) -> Option<String> {
        self.iter()
            .find_map(|instruction| {
                if let Instruction::Extends(name) = instruction {
                    Some(name)
                } else {
                    None
                }
            })
            .cloned()
    }

    pub fn rename(&self) -> Option<String> {
        self.iter()
            .find_map(|instruction| {
                if let Instruction::Rename(name) = instruction {
                    Some(name)
                } else {
                    None
                }
            })
            .cloned()
    }

    pub fn verbatim(&self) -> Vec<String> {
        self.iter()
            .filter_map(|instruction| {
                if let Instruction::Verbatim(verbatim) = instruction {
                    Some(verbatim)
                } else {
                    None
                }
            })
            .cloned()
            .collect_vec()
    }

    pub fn extra_methods(&self) -> Vec<String> {
        self.iter()
            .filter_map(|instruction| {
                if let Instruction::Method(method) = instruction {
                    Some(method)
                } else {
                    None
                }
            })
            .cloned()
            .collect_vec()
    }
}

newtype!(pub Overloads, Vec<(Instructions, Comments)>);

static INSTRUCTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^@(\w+) ?(.*)$"#).unwrap());
static RETURNS_AND_TYPE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^([\w\s\[\]<>,]+)$"#).unwrap());
static VARIABLE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r#"(?x)
        ^
        \s*
        (?: (?P<keyword>\w+) \s+ )?           # optional keyword
        (?P<name>\w+\??)                      # required name
        \s*:\s*
       
        # (3) Required type - either '{...}' (no nesting!) or a nonwhitespace token
        (?P<type>
            \{[^}]*\}
            |
            [^\s=]+
        )

        (?: \s*=\s*(?P<default>[^/]+?))?      # optional default
        (?: \s*//\s*(?P<comment>.*))?         # optional comment
        \s*
        $"#,
    )
    .unwrap()
});

fn extract_variable(parameters: &str) -> Result<Variable> {
    let captures = VARIABLE_REGEX
        .captures(parameters)
        .ok_or(eyre!("expected parameters, got: \"{parameters}\""))?;

    let keyword = captures.name("keyword").map(|m| m.as_str().to_string());
    let name = captures
        .name("name")
        .map(|m| m.as_str().to_string())
        .ok_or(eyre!("expected name"))?;
    let type_ = captures
        .name("type")
        .map(|m| m.as_str().to_string())
        .ok_or(eyre!("expected type"))?;
    let default = captures.name("default").map(|m| m.as_str().to_string());
    let comment = captures.name("comment").map(|m| m.as_str().to_string());

    let is_readonly = if let Some(keyword) = keyword {
        keyword == "readonly"
    } else {
        false
    };

    let comments = if let Some(comment) = comment {
        vec![comment]
    } else {
        vec![]
    };

    Ok(Variable {
        name,
        type_: Type::Verbatim(type_),
        comments: comments.into(),
        is_readonly,
        default_value: default,
        platforms: Default::default(),
    })
}

fn parse_instruction(line: &str) -> Result<Instruction> {
    let captures = INSTRUCTION_REGEX
        .captures(line)
        .ok_or(eyre!("expected instruction format: {line}"))?;

    let name = captures
        .get(1)
        .ok_or(eyre!("expected instruction name"))?
        .as_str();
    let parameters = captures
        .get(2)
        .ok_or(eyre!("expected instruction parameters"))?
        .as_str()
        .trim_end();

    Ok(match name {
        // @constructor
        "constructor" => {
            if !parameters.is_empty() {
                bail!("unexpected parameters");
            }

            Instruction::Constructor
        }

        // @private
        "private" => {
            if !parameters.is_empty() {
                bail!("unexpected parameters");
            }

            Instruction::Private
        }

        // @overload
        "overload" => {
            if !parameters.is_empty() {
                bail!("unexpected parameters");
            }

            Instruction::Overload
        }

        // @skip
        "skip" => {
            if !parameters.is_empty() {
                bail!("unexpected parameters");
            }

            Instruction::Skip
        }

        // @singleton
        "singleton" => {
            if !parameters.is_empty() {
                bail!("unexpected parameters");
            }

            Instruction::Singleton
        }

        // @options
        "options" => {
            if !parameters.is_empty() {
                bail!("unexpected parameters");
            }

            Instruction::Options
        }

        // @static
        "static" => {
            if !parameters.is_empty() {
                bail!("unexpected parameters");
            }

            Instruction::Static
        }

        // @generic
        "generic" => {
            if !parameters.is_empty() {
                bail!("unexpected parameters");
            }

            Instruction::Generic
        }

        // @rest
        "rest" => Instruction::Rest(if parameters.is_empty() {
            None
        } else {
            Some(parameters.to_string())
        }),

        // @const
        "const" => Instruction::Const(parameters.to_string()),

        // @extends
        "extends" => Instruction::Extends(parameters.to_string()),

        // @default
        "default" => Instruction::Default(parameters.to_string()),

        // @rename
        "rename" => Instruction::Rename(parameters.to_string()),

        // @verbatim
        "verbatim" => Instruction::Verbatim(parameters.to_string()),

        // @platforms
        "platforms" => Instruction::Platforms(Platforms::try_from(parameters)?),

        // @returns type // comment
        "returns" => {
            let captures = RETURNS_AND_TYPE_REGEX
                .captures(parameters)
                .ok_or(eyre!("expected returns parameters"))?;

            let type_ = captures.get(1).ok_or(eyre!("expected type"))?.as_str();

            Instruction::Returns(Type::Verbatim(type_.to_string()))
        }

        // @prop name: type // comment
        "prop" => Instruction::Property(extract_variable(parameters)?),

        // @param name: type // comment
        "param" => Instruction::Parameter(extract_variable(parameters)?),

        // @method
        "method" => Instruction::Method(parameters.to_string()),

        // @type type // comment
        "type" => {
            let captures = RETURNS_AND_TYPE_REGEX
                .captures(parameters)
                .ok_or(eyre!("expected type parameters"))?;

            let type_ = captures.get(1).ok_or(eyre!("expected type"))?.as_str();

            Instruction::Type(Type::Verbatim(type_.to_string()))
        }

        _ => bail!("unknown instruction {name}"),
    })
}

const fn allowed_context_for_instruction(
    instruction: InstructionDiscriminants,
) -> &'static [RustdocContext] {
    use InstructionDiscriminants::*;

    match instruction {
        Constructor => &[RustdocContext::Method],
        Private => &[RustdocContext::Method],
        Property => &[RustdocContext::Struct, RustdocContext::StructAlias],
        Parameter => &[RustdocContext::Method, RustdocContext::MethodOverload],
        Overload => &[RustdocContext::Method, RustdocContext::MethodOverload],
        Skip => &[
            RustdocContext::Method,
            RustdocContext::Struct,
            RustdocContext::Property,
            RustdocContext::Module,
        ],
        Returns => &[RustdocContext::Method],
        Singleton => &[RustdocContext::Struct, RustdocContext::StructAlias],
        Const => &[RustdocContext::Struct, RustdocContext::StructAlias],
        Default => &[RustdocContext::Property],
        Options => &[RustdocContext::Struct, RustdocContext::StructAlias],
        Extends => &[RustdocContext::Struct, RustdocContext::StructAlias],
        Rest => &[RustdocContext::Method],
        Rename => &[RustdocContext::Method],
        Static => &[RustdocContext::Method],
        Platforms => &[
            RustdocContext::Method,
            RustdocContext::MethodOverload,
            RustdocContext::Struct,
            RustdocContext::Property,
            RustdocContext::Enum,
            RustdocContext::EnumVariant,
        ],
        Generic => &[RustdocContext::Struct, RustdocContext::Method],
        Method => &[RustdocContext::Struct],
        Type => &[RustdocContext::Property],
        Verbatim => &[RustdocContext::Struct, RustdocContext::Module],
    }
}

fn check_instruction(instruction: &Instruction, context: &RustdocContext) -> Result<()> {
    let instruction = instruction.into();
    if !allowed_context_for_instruction(instruction).contains(context) {
        bail!("Instruction {instruction} is not allowed within context {context}");
    }

    Ok(())
}

fn process_rustdoc(
    rustdoc: Option<&String>,
    context: RustdocContext,
) -> Result<(Comments, Instructions, Overloads)> {
    let Some(rustdoc) = rustdoc else {
        return Ok((
            Comments::default(),
            Instructions::default(),
            Overloads::default(),
        ));
    };

    let lines = rustdoc.lines().map(|line| line.trim());
    let mut comments = Vec::new();

    // Current instructions; will be reset if we encounter an overload instruction
    let mut instructions = Vec::new();

    let mut general_instructions = None;
    let mut general_comments = None;

    // Overloads, if any
    let mut overloads = Vec::new();
    let mut has_overload = false;

    for line in lines {
        let is_instruction = line.starts_with("@");
        if !is_instruction {
            comments.push(line.to_string());
            continue;
        }

        let instruction = parse_instruction(line)?;

        if let Instruction::Overload = instruction {
            check_instruction(&Instruction::Overload, &context)?;

            if has_overload {
                overloads.push((take(&mut instructions), take(&mut comments)));
            } else {
                general_instructions = Some(take(&mut instructions));
                general_comments = Some(take(&mut comments));
            }
            has_overload = true;
        }

        instructions.push(instruction);
    }

    if has_overload {
        overloads.push((take(&mut instructions), take(&mut comments)));
    } else {
        general_instructions = Some(take(&mut instructions));
        general_comments = Some(take(&mut comments));
    }

    // Check if instructions are valid in overloads
    for (overload, _) in &overloads {
        for instruction in overload {
            check_instruction(instruction, &RustdocContext::MethodOverload)?;
        }
    }

    let instructions = general_instructions.unwrap_or_default();
    let comments = general_comments.unwrap_or_default();

    // Check if other instructions are valid
    for instruction in &instructions {
        check_instruction(instruction, &context)?;
    }

    let overloads = overloads
        .into_iter()
        .map(|(instructions, comments)| (Instructions(instructions), Comments(comments).trimmed()))
        .collect_vec();

    Ok((
        Comments(comments).trimmed(),
        Instructions(instructions),
        Overloads(overloads),
    ))
}

fn primitive_to_type(primitive: &String) -> Result<Type> {
    Ok(match primitive.as_str() {
        "bool" => Type::Bool,
        "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "f32" | "f64" => Type::Number,
        "str" => Type::String,
        _ => {
            bail!("Unsupported primitive type: {primitive}");
        }
    })
}

fn unwrap_generic(path: &rustdoc_types::Path) -> Result<&rustdoc_types::Type> {
    let Some(args) = &path.args else {
        bail!("No args for ResolvedPath: {path:?}");
    };
    let rustdoc_types::GenericArgs::AngleBracketed { args, .. } = args.as_ref() else {
        bail!("Unsupported ResolvedPath: {path:?}");
    };
    let Some(first_arg) = args.first() else {
        bail!("No args for ResolvedPath: {path:?}");
    };
    let rustdoc_types::GenericArg::Type(type_) = first_arg else {
        bail!("Unsupported ResolvedPath: {path:?}");
    };

    Ok(type_)
}

fn convert_type(output: &rustdoc_types::Type, struct_name: Option<&str>) -> Result<Type> {
    Ok(match output {
        rustdoc_types::Type::Primitive(primitive) => primitive_to_type(primitive)?,
        rustdoc_types::Type::Generic(generic) => match generic.as_str() {
            "Self" => Type::Verbatim(
                struct_name
                    .ok_or_else(|| eyre!("expected struct name, but none set (free function?)"))?
                    .to_string(),
            ),
            _ => {
                bail!("Unsupported generic type: {generic}, struct: {struct_name:?}");
            }
        },
        rustdoc_types::Type::ResolvedPath(path) => match strip_modules(path.path.as_str()) {
            "String" => Type::String,
            "Result" => {
                let type_ = unwrap_generic(path)?;
                convert_type(type_, struct_name)?
            }
            "Option" | "Opt" => {
                let type_ = unwrap_generic(path)?;
                Type::Option(Box::new(convert_type(type_, struct_name)?))
            }
            "Class" | "This" => Type::This,
            "Ctx" => Type::Ignore,
            "Rest" => Type::Unknown,
            "TypedArray" => {
                let Some(args) = &path.args else {
                    bail!("No args for TypedArray: {path:?}");
                };
                let rustdoc_types::GenericArgs::AngleBracketed { args, .. } = args.as_ref() else {
                    bail!("Unsupported TypedArray: {path:?}");
                };
                let type_ = args
                    .iter()
                    .filter_map(|arg| {
                        if let rustdoc_types::GenericArg::Type(type_) = arg {
                            Some(type_)
                        } else {
                            None
                        }
                    })
                    .next()
                    .ok_or(eyre!("Unsupported TypedArray: {path:?}"))?;
                let rustdoc_types::Type::Primitive(type_) = type_ else {
                    bail!("Unsupported TypedArray type: {path:?}, type: {type_:?}");
                };
                Type::Verbatim(
                    match type_.as_str() {
                        "u8" => "Uint8Array",
                        _ => {
                            bail!("Unsupported TypedArray type: {path:?}, type: {type_:?}");
                        }
                    }
                    .to_string(),
                )
            }
            "JsDuration" => Type::Number,
            object => Type::Verbatim(object.to_string()),
        },
        rustdoc_types::Type::BorrowedRef { type_, .. } => match type_.as_ref() {
            rustdoc_types::Type::Primitive(primitive) => primitive_to_type(primitive)?,
            rustdoc_types::Type::Generic(generic) if generic == "Self" => Type::This,
            rustdoc_types::Type::ResolvedPath(path) => Type::Verbatim(path.path.clone()),
            _ => {
                bail!("Unsupported BorrowedRef type: {type_:?}");
            }
        },
        rustdoc_types::Type::Tuple(tuple) => {
            if tuple.is_empty() {
                Type::Void
            } else {
                bail!("Unsupported tuple type: {tuple:?}");
            }
        }
        type_ => {
            bail!("Unsupported type: {type_:?}");
        }
    })
}

impl TryFrom<Crate> for File {
    type Error = eyre::Error;

    fn try_from(crate_: Crate) -> Result<Self, Self::Error> {
        let items = Items::new(crate_);

        // TODO: get rustdoc from Modules
        let modules = process_modules(&items)?;

        let mut structs = process_structs(&items)?;
        let mut struct_aliases = process_structs(&items.aliases())?;
        structs.append(&mut struct_aliases);

        let mut enums = process_enums(&items)?;
        let mut enum_aliases = process_enums(&items.aliases())?;
        enums.append(&mut enum_aliases);

        let functions = process_functions(&items)?;

        Ok(File {
            enums,
            structs,
            functions,
            modules,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::process_rustdoc;
    use crate::{
        input::{Comments, Instructions, Overloads},
        types::{Instruction, RustdocContext, Type, Variable},
    };

    #[test]
    fn test_empty() {
        let (comments, instructions, overloads) =
            process_rustdoc(None, RustdocContext::Struct).unwrap();
        assert_eq!(comments, Comments::default());
        assert_eq!(instructions, Instructions::default());
        assert_eq!(overloads, Overloads::default());

        let (comments, instructions, overloads) =
            process_rustdoc(Some(&String::default()), RustdocContext::Struct).unwrap();
        assert_eq!(comments, Comments::default());
        assert_eq!(instructions, Instructions::default());
        assert_eq!(overloads, Overloads::default());
    }

    #[test]
    fn test_only_comment() {
        let rustdoc = r#"Test"#;
        let (comments, instructions, overloads) =
            process_rustdoc(Some(&rustdoc.to_string()), RustdocContext::Struct).unwrap();
        assert_eq!(comments, vec!["Test".to_string()].into());
        assert_eq!(instructions, Instructions::default());
        assert_eq!(overloads, Overloads::default());
    }

    #[test]
    fn test_only_instruction() {
        let rustdoc = r#"@skip"#;
        let (comments, instructions, overloads) =
            process_rustdoc(Some(&rustdoc.to_string()), RustdocContext::Struct).unwrap();
        assert_eq!(comments, Comments::default());
        assert_eq!(instructions, vec![Instruction::Skip].into());
        assert_eq!(overloads, Overloads::default());
    }

    #[test]
    fn test_both() {
        let rustdoc = r#"Some comment
        
        Another comment
        
        @constructor
        @skip"#;
        let (comments, instructions, overloads) =
            process_rustdoc(Some(&rustdoc.to_string()), RustdocContext::Method).unwrap();
        assert_eq!(
            comments,
            vec![
                "Some comment".to_string(),
                "".to_string(),
                "Another comment".to_string()
            ]
            .into()
        );
        assert_eq!(
            instructions,
            vec![Instruction::Constructor, Instruction::Skip].into()
        );
        assert_eq!(overloads, Overloads::default());
    }

    #[test]
    fn test_overloading() {
        let rustdoc = r#"Constructor.

            @constructor

            @overload
            Comment for the first overload
            @param x?: number // X coordinate
            @param y: number = 42 // Y coordinate

            @overload
            @param o: {x: number, y: number} // Object containing the x and y coordinates

            @overload
            Comment for the last overload
            @param p: Point // Other point"#;
        let (comments, instructions, overloads) =
            process_rustdoc(Some(&rustdoc.to_string()), RustdocContext::Method).unwrap();
        assert_eq!(comments, vec!["Constructor.".to_string()].into());
        assert_eq!(instructions, vec![Instruction::Constructor].into());
        assert_eq!(
            overloads,
            Overloads(vec![
                (
                    Instructions(vec![
                        Instruction::Overload,
                        Instruction::Parameter(Variable {
                            name: "x?".to_string(),
                            type_: Type::Verbatim("number".to_string()),
                            comments: Comments(vec!["X coordinate".to_string()]),
                            is_readonly: false,
                            default_value: None,
                            platforms: instructions.platforms(),
                        }),
                        Instruction::Parameter(Variable {
                            name: "y".to_string(),
                            type_: Type::Verbatim("number".to_string()),
                            comments: Comments(vec!["Y coordinate".to_string()]),
                            is_readonly: false,
                            default_value: Some("42".to_string()),
                            platforms: instructions.platforms(),
                        })
                    ]),
                    Comments(vec!["Comment for the first overload".to_string()])
                ),
                (
                    Instructions(vec![
                        Instruction::Overload,
                        Instruction::Parameter(Variable {
                            name: "o".to_string(),
                            type_: Type::Verbatim("{x: number, y: number}".to_string()),
                            comments: Comments(vec![
                                "Object containing the x and y coordinates".to_string()
                            ]),
                            is_readonly: false,
                            default_value: None,
                            platforms: instructions.platforms(),
                        })
                    ]),
                    Comments(vec![])
                ),
                (
                    Instructions(vec![
                        Instruction::Overload,
                        Instruction::Parameter(Variable {
                            name: "p".to_string(),
                            type_: Type::Verbatim("Point".to_string()),
                            comments: Comments(vec!["Other point".to_string()]),
                            is_readonly: false,
                            default_value: None,
                            platforms: instructions.platforms(),
                        })
                    ]),
                    Comments(vec!["Comment for the last overload".to_string()])
                )
            ])
        );
    }
}
