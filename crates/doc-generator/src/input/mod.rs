use std::{mem::take, sync::LazyLock};

use actiona_core::newtype;
use color_eyre::{
    Result,
    eyre::{OptionExt, bail, eyre},
};
use enums::process_enums;
use itertools::Itertools;
use regex::Regex;
use rustdoc_types::{Crate, Item};
use structs::process_structs;

use crate::{
    input::{functions::process_functions, modules::process_modules},
    items::Items,
    types::{
        Const, File, Instruction, InstructionDiscriminants, Platforms, RestParams, RustdocContext,
        Type, Variable, strip_modules,
    },
};

pub mod enums;
pub mod functions;
pub mod modules;
pub mod structs;

/// Metadata extracted from a rustdoc `Item` during filtering.
pub struct ItemInfo<'a, T> {
    pub name: &'a str,
    pub docs: &'a Option<String>,
    pub inner: &'a T,
    pub item: &'a Item,
}

newtype!(
    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    pub Comments,
    Vec<String>
);

impl Comments {
    pub fn trimmed(mut self) -> Self {
        // Remove leading empty/whitespace-only lines
        while self.first().is_some_and(|s| s.trim().is_empty()) {
            self.remove(0);
        }

        // Remove trailing empty/whitespace-only lines
        while self.last().is_some_and(|s| s.trim().is_empty()) {
            self.pop();
        }

        // Rustdoc can add a shared left margin for some items (notably impl
        // methods). Remove that common margin while preserving relative
        // indentation (important for fenced code blocks).
        let common_left_padding = self
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                line.chars()
                    .take_while(|character| *character == ' ')
                    .count()
            })
            .min()
            .unwrap_or(0);

        if common_left_padding > 0 {
            for line in &mut self.0 {
                if line.trim().is_empty() {
                    continue;
                }

                let trimmed_line = line
                    .strip_prefix(&" ".repeat(common_left_padding))
                    .unwrap_or(line);
                *line = trimmed_line.to_owned();
            }
        }

        self
    }
}

newtype!(pub Instructions, Vec<Instruction>);

impl Instructions {
    pub fn has_skip(&self) -> bool {
        self.iter().any(|instruction| instruction.is_skip())
    }

    pub fn has_constructor(&self) -> bool {
        self.iter().any(|instruction| instruction.is_constructor())
    }

    pub fn has_private(&self) -> bool {
        self.iter().any(|instruction| instruction.is_private())
    }

    pub fn has_static(&self) -> bool {
        self.iter().any(|instruction| instruction.is_static())
    }

    pub fn is_singleton(&self) -> bool {
        self.iter().any(|instruction| instruction.is_singleton())
    }

    pub fn is_generic(&self) -> bool {
        self.iter().any(|instruction| instruction.is_generic())
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

    pub fn default_value(&self) -> Option<String> {
        self.iter().find_map(|instruction| {
            if let Instruction::Default(default_value) = instruction {
                Some(default_value.clone())
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
        self.iter().any(|instruction| instruction.is_options())
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

    pub fn has_getter(&self) -> bool {
        self.iter().any(|instruction| instruction.is_getter())
    }

    pub fn has_readonly_type(&self) -> bool {
        self.iter()
            .any(|instruction| instruction.is_readonly_type())
    }

    pub fn has_constructor_only(&self) -> bool {
        self.iter()
            .any(|instruction| instruction.is_constructor_only())
    }

    pub fn category(&self) -> Option<String> {
        self.iter().find_map(|instruction| {
            if let Instruction::Category(category) = instruction {
                Some(category.clone())
            } else {
                None
            }
        })
    }

    pub fn is_expand(&self) -> bool {
        self.iter().any(|instruction| instruction.is_expand())
    }
}

newtype!(pub Overloads, Vec<(Instructions, Comments)>);

static INSTRUCTION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^@(\w+) ?(.*)$").expect("instruction regex is valid"));
static RETURNS_AND_TYPE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([\w\s\[\]<>,|]+)$").expect("return/type regex is valid"));
static CONST_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?P<value>[\w]+)(?: // (?P<comment>.+))?$").expect("constant regex is valid")
});
static VARIABLE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?x)
        ^
        \s*
        (?: (?P<keyword>\w+) \s+ )?           # optional keyword
        (?P<name>\w+\??)                      # required name
        \s*:\s*

        (?P<type>
            \{[^}]*\}                         # object type: { foo: string }
            |
            # union of atoms (function types or simple types)
            (?:
                # first atom
                (?:
                    \( [^)]* \) \s* => \s* [^=|]+   # function atom: () => string
                    |
                    [^=\s|]+                        # simple atom: Foo, Promise<string>
                )
                # zero or more: | atom
                (?:
                    \s* \| \s*
                    (?:
                        \( [^)]* \) \s* => \s* [^=|]+
                        |
                        [^=\s|]+
                    )
                )*
            )
        )

        (?: \s*=\s*(?P<default>[^/]+?))?      # optional default
        (?: \s*//\s*(?P<comment>.*))?         # optional comment
        \s*
        $",
    )
    .expect("variable regex is valid")
});

fn extract_variable(parameters: &str) -> Result<Variable> {
    let captures = VARIABLE_REGEX
        .captures(parameters)
        .ok_or_else(|| eyre!("expected parameters, got: \"{parameters}\""))?;

    let keyword = captures.name("keyword").map(|m| m.as_str().to_owned());
    let name = captures
        .name("name")
        .map(|m| m.as_str().to_owned())
        .ok_or_eyre("expected name")?;
    let type_ = captures
        .name("type")
        .map(|m| m.as_str().to_owned())
        .ok_or_eyre("expected type")?;
    let default = captures.name("default").map(|m| m.as_str().to_owned());
    let comment = captures.name("comment").map(|m| m.as_str().to_owned());

    let is_readonly = keyword.is_some_and(|keyword| keyword == "readonly");

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
        is_readonly_type: false,
        default_value: default,
        platforms: Platforms::default(),
        is_promise: false,
    })
}

fn extract_const(parameters: &str) -> Result<Const> {
    let captures = CONST_REGEX
        .captures(parameters)
        .ok_or_else(|| eyre!("expected parameters, got: \"{parameters}\""))?;

    let value = captures
        .name("value")
        .map(|m| m.as_str().to_owned())
        .ok_or_eyre("expected value")?;
    let comment = captures.name("comment").map(|m| m.as_str().to_owned());

    let comments = if let Some(comment) = comment {
        vec![comment]
    } else {
        vec![]
    };

    Ok(Const {
        value,
        comments: comments.into(),
    })
}

fn parse_instruction(line: &str) -> Result<Instruction> {
    let captures = INSTRUCTION_REGEX
        .captures(line)
        .ok_or_else(|| eyre!("expected instruction format: {line}"))?;

    let name = captures
        .get(1)
        .ok_or_eyre("expected instruction name")?
        .as_str();
    let parameters = captures
        .get(2)
        .ok_or_eyre("expected instruction parameters")?
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

        // @get
        "get" => {
            if !parameters.is_empty() {
                bail!("unexpected parameters");
            }

            Instruction::Getter
        }

        // @readonly
        "readonly" => {
            if !parameters.is_empty() {
                bail!("unexpected parameters");
            }

            Instruction::ReadonlyType
        }

        // @constructorOnly
        "constructorOnly" => {
            if !parameters.is_empty() {
                bail!("unexpected parameters");
            }

            Instruction::ConstructorOnly
        }

        // @rest
        "rest" => Instruction::Rest(if parameters.is_empty() {
            None
        } else {
            Some(parameters.to_owned())
        }),

        // @const // comment
        "const" => Instruction::Const(extract_const(parameters)?),

        // @extends
        "extends" => Instruction::Extends(parameters.to_owned()),

        // @default
        "default" => Instruction::Default(parameters.to_owned()),

        // @rename
        "rename" => Instruction::Rename(parameters.to_owned()),

        // @verbatim
        "verbatim" => Instruction::Verbatim(parameters.to_owned()),

        // @platforms
        "platforms" => Instruction::Platforms(Platforms::try_from(parameters)?),

        // @returns type // comment
        "returns" => {
            let captures = RETURNS_AND_TYPE_REGEX
                .captures(parameters)
                .ok_or_eyre("expected returns parameters")?;

            let type_ = captures.get(1).ok_or_eyre("expected type")?.as_str();

            Instruction::Returns(Type::Verbatim(type_.to_owned()))
        }

        // @prop name: type // comment
        "prop" => Instruction::Property(extract_variable(parameters)?),

        // @param name: type // comment
        "param" => Instruction::Parameter(extract_variable(parameters)?),

        // @method
        "method" => Instruction::Method(parameters.to_owned()),

        // @type type // comment
        "type" => {
            let captures = RETURNS_AND_TYPE_REGEX
                .captures(parameters)
                .ok_or_eyre("expected type parameters")?;

            let type_ = captures.get(1).ok_or_eyre("expected type")?.as_str();

            Instruction::Type(Type::Verbatim(type_.to_owned()))
        }

        // @category CategoryName
        "category" => {
            if parameters.is_empty() {
                bail!("expected category name");
            }

            Instruction::Category(parameters.to_owned())
        }

        // @expand
        "expand" => {
            if !parameters.is_empty() {
                bail!("unexpected parameters");
            }

            Instruction::Expand
        }

        _ => bail!("unknown instruction {name}"),
    })
}

const fn allowed_context_for_instruction(
    instruction: InstructionDiscriminants,
) -> &'static [RustdocContext] {
    use InstructionDiscriminants::*;

    match instruction {
        Constructor | Private | Returns | Rest | Static | Getter | ReadonlyType => {
            &[RustdocContext::Method]
        }
        Property | Singleton | Const | Options | Extends => {
            &[RustdocContext::Struct, RustdocContext::StructAlias]
        }
        Parameter | Overload => &[RustdocContext::Method, RustdocContext::MethodOverload],
        Skip => &[
            RustdocContext::Method,
            RustdocContext::Struct,
            RustdocContext::Property,
            RustdocContext::Module,
            RustdocContext::Enum,
        ],
        Default => &[RustdocContext::Property, RustdocContext::Enum],
        Rename => &[RustdocContext::Method, RustdocContext::Enum],
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
        Verbatim => &[
            RustdocContext::Struct,
            RustdocContext::Module,
            RustdocContext::Enum,
        ],
        ConstructorOnly => &[RustdocContext::MethodOverload],
        Category => &[
            RustdocContext::Struct,
            RustdocContext::StructAlias,
            RustdocContext::Enum,
            RustdocContext::Method,
        ],
        Expand => &[RustdocContext::Enum],
    }
}

fn check_instruction(instruction: &Instruction, context: RustdocContext) -> Result<()> {
    let instruction = instruction.into();
    if !allowed_context_for_instruction(instruction).contains(&context) {
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

    // Keep leading whitespace in comments. Shared indentation is normalized later
    // in `Comments::trimmed()`.
    let lines = rustdoc.lines().map(str::trim_end);
    let mut comments = Vec::new();

    // Current instructions; will be reset if we encounter an overload instruction
    let mut instructions = Vec::new();

    let mut general_instructions = None;
    let mut general_comments = None;

    // Overloads, if any
    let mut overloads = Vec::new();
    let mut has_overload = false;

    for line in lines {
        let line_without_prefix_whitespace = line.trim_start();
        let is_instruction = line_without_prefix_whitespace.starts_with('@');
        if !is_instruction {
            comments.push(line.to_owned());
            continue;
        }

        let instruction = parse_instruction(line_without_prefix_whitespace)?;

        if instruction == Instruction::Overload {
            check_instruction(&Instruction::Overload, context)?;

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
            check_instruction(instruction, RustdocContext::MethodOverload)?;
        }
    }

    let instructions = general_instructions.unwrap_or_default();
    let comments = general_comments.unwrap_or_default();

    // Check if other instructions are valid
    for instruction in &instructions {
        check_instruction(instruction, context)?;
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
    args.iter()
        .find_map(|arg| {
            if let rustdoc_types::GenericArg::Type(type_) = arg {
                Some(type_)
            } else {
                None
            }
        })
        .ok_or_else(|| eyre!("No type args for ResolvedPath: {path:?}"))
}

fn unwrap_generic_pair(
    path: &rustdoc_types::Path,
) -> Result<(&rustdoc_types::Type, &rustdoc_types::Type)> {
    let Some(args) = &path.args else {
        bail!("No args for ResolvedPath: {path:?}");
    };
    let rustdoc_types::GenericArgs::AngleBracketed { args, .. } = args.as_ref() else {
        bail!("Unsupported ResolvedPath: {path:?}");
    };
    let mut args = args.iter();
    let Some(first_arg) = args.next() else {
        bail!("No first arg for ResolvedPath: {path:?}");
    };
    let rustdoc_types::GenericArg::Type(first_type_) = first_arg else {
        bail!("Unsupported ResolvedPath: {path:?}");
    };
    let Some(second_arg) = args.next() else {
        bail!("No second arg for ResolvedPath: {path:?}");
    };
    let rustdoc_types::GenericArg::Type(second_type_) = second_arg else {
        bail!("Unsupported ResolvedPath: {path:?}");
    };

    Ok((first_type_, second_type_))
}

fn convert_type(output: &rustdoc_types::Type, struct_name: Option<&str>) -> Result<Type> {
    Ok(match output {
        rustdoc_types::Type::Primitive(primitive) => primitive_to_type(primitive)?,
        rustdoc_types::Type::Generic(generic) => match generic.as_str() {
            "Self" => Type::Verbatim(
                struct_name
                    .ok_or_eyre("expected struct name, but none set (free function?)")?
                    .to_owned(),
            ),
            _ => {
                bail!("Unsupported generic type: {generic}, struct: {struct_name:?}");
            }
        },
        rustdoc_types::Type::ResolvedPath(path) => path_to_type(path, struct_name)?,
        rustdoc_types::Type::BorrowedRef { type_, .. } => match type_.as_ref() {
            rustdoc_types::Type::Primitive(primitive) => primitive_to_type(primitive)?,
            rustdoc_types::Type::Generic(generic) if generic == "Self" => Type::This,
            rustdoc_types::Type::ResolvedPath(path) => path_to_type(path, struct_name)?,
            rustdoc_types::Type::Slice(type_) => {
                Type::Array(Box::new(convert_type(type_, struct_name)?))
            }
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

fn path_to_type(path: &rustdoc_types::Path, struct_name: Option<&str>) -> Result<Type> {
    Ok(match strip_modules(path.path.as_str()) {
        // JsName is the internal parser type for APIs that accept NameLike.
        "JsName" => Type::Verbatim("NameLike".to_owned()),
        "String" => Type::String,
        "Result" => {
            let type_ = unwrap_generic(path)?;
            convert_type(type_, struct_name)?
        }
        "Option" | "Opt" => {
            let type_ = unwrap_generic(path)?;
            Type::Option(Box::new(convert_type(type_, struct_name)?))
        }
        "Vec" => {
            let type_ = unwrap_generic(path)?;
            Type::Array(Box::new(convert_type(type_, struct_name)?))
        }
        "HashMap" | "BTreeMap" | "IndexMap" => {
            let (key_type, value_type) = unwrap_generic_pair(path)?;
            Type::Record(
                Box::new(convert_type(key_type, struct_name)?),
                Box::new(convert_type(value_type, struct_name)?),
            )
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
                .find_map(|arg| {
                    if let rustdoc_types::GenericArg::Type(type_) = arg {
                        Some(type_)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| eyre!("Unsupported TypedArray: {path:?}"))?;
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
                .to_owned(),
            )
        }
        "JsDuration" => Type::Verbatim("DurationLike".to_owned()),
        object => Type::Verbatim(object.to_owned()),
    })
}

impl TryFrom<Crate> for File {
    type Error = color_eyre::Report;

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

        Ok(Self {
            enums,
            structs,
            functions,
            modules,
        })
    }
}

#[cfg(test)]
mod tests {
    use rustdoc_types::{Id, Path};

    use super::{convert_type, process_rustdoc};
    use crate::{
        input::{Comments, Instructions, Overloads},
        types::{Context, Instruction, RustdocContext, Type, Variable},
    };

    #[test]
    fn empty() {
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
    fn only_comment() {
        let rustdoc = "Test";
        let (comments, instructions, overloads) =
            process_rustdoc(Some(&rustdoc.to_owned()), RustdocContext::Struct).unwrap();
        assert_eq!(comments, vec!["Test".to_owned()].into());
        assert_eq!(instructions, Instructions::default());
        assert_eq!(overloads, Overloads::default());
    }

    #[test]
    fn only_instruction() {
        let rustdoc = "@skip";
        let (comments, instructions, overloads) =
            process_rustdoc(Some(&rustdoc.to_owned()), RustdocContext::Struct).unwrap();
        assert_eq!(comments, Comments::default());
        assert_eq!(instructions, vec![Instruction::Skip].into());
        assert_eq!(overloads, Overloads::default());
    }

    #[test]
    fn both() {
        let rustdoc = "Some comment

Another comment

@constructor
@skip";
        let (comments, instructions, overloads) =
            process_rustdoc(Some(&rustdoc.to_owned()), RustdocContext::Method).unwrap();
        assert_eq!(
            comments,
            vec![
                "Some comment".to_owned(),
                String::new(),
                "Another comment".to_owned()
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
    fn overloading() {
        let rustdoc = "Constructor.

@constructor

@overload
Comment for the first overload
@param x?: number // X coordinate
@param y: number = 42 // Y coordinate

@overload
@param o: {x: number, y: number} // Object containing the x and y coordinates

@overload
Comment for the last overload
@param p: Point // Other point";
        let (comments, instructions, overloads) =
            process_rustdoc(Some(&rustdoc.to_owned()), RustdocContext::Method).unwrap();
        assert_eq!(comments, vec!["Constructor.".to_owned()].into());
        assert_eq!(instructions, vec![Instruction::Constructor].into());
        assert_eq!(
            overloads,
            Overloads(vec![
                (
                    Instructions(vec![
                        Instruction::Overload,
                        Instruction::Parameter(Variable {
                            name: "x?".to_owned(),
                            type_: Type::Verbatim("number".to_owned()),
                            comments: Comments(vec!["X coordinate".to_owned()]),
                            is_readonly: false,
                            is_readonly_type: false,
                            default_value: None,
                            platforms: instructions.platforms(),
                            is_promise: false,
                        }),
                        Instruction::Parameter(Variable {
                            name: "y".to_owned(),
                            type_: Type::Verbatim("number".to_owned()),
                            comments: Comments(vec!["Y coordinate".to_owned()]),
                            is_readonly: false,
                            is_readonly_type: false,
                            default_value: Some("42".to_owned()),
                            platforms: instructions.platforms(),
                            is_promise: false,
                        })
                    ]),
                    Comments(vec!["Comment for the first overload".to_owned()])
                ),
                (
                    Instructions(vec![
                        Instruction::Overload,
                        Instruction::Parameter(Variable {
                            name: "o".to_owned(),
                            type_: Type::Verbatim("{x: number, y: number}".to_owned()),
                            comments: Comments(vec![
                                "Object containing the x and y coordinates".to_owned()
                            ]),
                            is_readonly: false,
                            is_readonly_type: false,
                            default_value: None,
                            platforms: instructions.platforms(),
                            is_promise: false,
                        })
                    ]),
                    Comments(vec![])
                ),
                (
                    Instructions(vec![
                        Instruction::Overload,
                        Instruction::Parameter(Variable {
                            name: "p".to_owned(),
                            type_: Type::Verbatim("Point".to_owned()),
                            comments: Comments(vec!["Other point".to_owned()]),
                            is_readonly: false,
                            is_readonly_type: false,
                            default_value: None,
                            platforms: instructions.platforms(),
                            is_promise: false,
                        })
                    ]),
                    Comments(vec!["Comment for the last overload".to_owned()])
                )
            ])
        );
    }

    #[test]
    fn convert_js_name_to_name_like() {
        let rustdoc_type = rustdoc_types::Type::ResolvedPath(Path {
            path: "crate::api::name::js::JsName".to_owned(),
            id: Id(0),
            args: None,
        });

        let type_ = convert_type(&rustdoc_type, None).unwrap();
        assert_eq!(type_, Type::Verbatim("NameLike".to_owned()));
        assert_eq!(type_.to_string(Context::Property).unwrap(), "NameLike");
    }

    #[test]
    fn convert_js_duration_to_duration_like() {
        let rustdoc_type = rustdoc_types::Type::ResolvedPath(Path {
            path: "crate::api::js::duration::JsDuration".to_owned(),
            id: Id(0),
            args: None,
        });

        let type_ = convert_type(&rustdoc_type, None).unwrap();
        assert_eq!(type_, Type::Verbatim("DurationLike".to_owned()));
        assert_eq!(type_.to_string(Context::Property).unwrap(), "DurationLike");
    }
}
