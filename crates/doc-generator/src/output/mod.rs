use std::{cmp::Reverse, collections::HashMap, fs, io::Write};

use color_eyre::Result;
use convert_case::{Case, Casing};
use itertools::Itertools;

use crate::{
    built_info,
    types::{Context, File, Method, Type, strip_modules},
};

fn write_comments<W: Write>(comments: &[String], prefix: &str, file: &mut W) -> Result<()> {
    if comments.is_empty() {
        return Ok(());
    }

    write!(file, "{prefix}")?;
    writeln!(file, "/**")?;

    for comment in comments {
        write!(file, "{prefix}")?;
        writeln!(file, " * {}", comment.replace("*/", "*\\/"))?;
    }

    write!(file, "{prefix}")?;
    writeln!(file, " */")?;

    Ok(())
}

fn add_category_comment(comments: &mut Vec<String>, category: Option<&str>) {
    let Some(category) = category else {
        return;
    };

    let has_category = comments
        .iter()
        .any(|comment| comment.trim_start().starts_with("@category"));

    if !has_category {
        comments.push(format!("@category {category}"));
    }
}

impl Type {
    pub fn to_string(&self, context: Context) -> Result<String> {
        Ok(match self {
            Type::Void => "void".into(),
            Type::Bool => "boolean".into(),
            Type::Number => "number".into(),
            Type::This => "this".into(),
            Type::Ignore => unreachable!("ignored type should not be output"),
            Type::Unknown => "unknown".into(),
            Type::String => "string".into(),
            Type::Option(option) => match context {
                // When a parameter or property is optional we use ? after the name
                Context::Variable | Context::Property => option.to_string(context)?,

                // If it's a return value then we have to use the "| undefined" syntax instead
                Context::ReturnValue => format!("{} | undefined", option.to_string(context)?),
            },
            Type::Verbatim(type_) => {
                let type_ = strip_modules(type_);

                // Remove "Js" prefix if present
                let type_ = type_.strip_prefix("Js").unwrap_or(type_);

                type_.to_string()
            }
            Type::Array(type_) => format!("{}[]", type_.to_string(context)?),
            Type::Record(key_type, value_type) => format!(
                "Record<{}, {} | undefined>",
                key_type.to_string(context)?,
                value_type.to_string(context)?
            ),
        })
    }
}

impl File {
    pub fn fix_duplicate_parameter_names(&mut self) {
        for struct_ in &mut self.structs {
            for method in &mut struct_.methods {
                for overload in &mut method.overloads {
                    let mut total_counts = HashMap::new();
                    for variable in &overload.parameters {
                        *total_counts.entry(variable.name.clone()).or_insert(0) += 1;
                    }

                    let mut current_counts = HashMap::new();
                    for parameter in &mut overload.parameters {
                        let total = total_counts.get(&parameter.name).unwrap_or(&1);
                        if *total > 1 {
                            let count = current_counts.entry(parameter.name.clone()).or_insert(0);
                            *count += 1;
                            parameter.name = format!("{}{}", parameter.name, count);
                        }
                    }
                }
            }
        }
    }

    pub fn auto_generate_overloads(&mut self) -> Result<()> {
        // TODO: cleanup
        // We can auto-generate overloads for types that have multiple constructors, like Point.
        // For instance: mouse.move(p: Point) should also have mouse.move(x, y) and mouse.move({x: number, y: number})

        // 1) Build a lookup of all types that have multiple (≥1) constructor overloads
        //    (i.e., the user’s custom structs that define `is_constructor == true`).
        //    We only care if there’s more than 1 overload. If it’s exactly 1, we gain nothing by inlining.
        let mut constructors_for_type = HashMap::new();

        for s in &self.structs {
            let struct_type = Type::Verbatim(s.name.clone());
            // Find a constructor method on that struct
            let maybe_constructor_method = s.methods.iter().find(|m| m.is_constructor).map(|m| {
                m.overloads
                    .iter()
                    .filter(|overload| !overload.constructor_only)
                    .cloned()
                    .collect_vec()
            });

            let Some(overloads) = maybe_constructor_method else {
                // No constructor method on this type => skip
                continue;
            };

            constructors_for_type.insert(
                format!("{}Like", s.name), // TODO: hack
                overloads
                    .iter()
                    .filter(|overload| {
                        if let Some(parameter) = overload.parameters.first() {
                            parameter.type_ != struct_type
                        } else {
                            false
                        }
                    })
                    .cloned()
                    .collect_vec(),
            );
        }

        // 2) Now for each struct & each method, expand the overloads
        for s in self.structs.iter_mut() {
            for method in &mut s.methods {
                if method.is_constructor {
                    continue;
                }

                // Start with the original overloads
                let mut expanded_overloads = method.overloads.clone();

                // We’ll expand them parameter‐by‐parameter so that expansions
                // from param #1 feed into expansions from param #2, etc.
                // Because different overloads can have different param lengths,
                // we’ll run a loop that stops when we exceed the max param
                // count among the expansions.
                let mut param_idx = 0;
                loop {
                    // The maximum parameter count in the *current* expansions
                    let max_params = expanded_overloads
                        .iter()
                        .map(|ov| ov.parameters.len())
                        .max()
                        .unwrap_or(0);
                    if param_idx >= max_params {
                        break;
                    }

                    let mut next_round = Vec::new();

                    // Expand each existing overload in `expanded_overloads`
                    for ov in &expanded_overloads {
                        // If this overload doesn't even have `param_idx`,
                        // then we just keep it as-is.
                        if param_idx >= ov.parameters.len() {
                            next_round.push(ov.clone());
                            continue;
                        }

                        let param = &ov.parameters[param_idx];
                        let type_ = param.type_.to_string(Context::Variable)?;
                        if let Some(ctor_overloads) = constructors_for_type.get(&type_) {
                            // Keep the original overload:
                            next_round.push(ov.clone());

                            // For each constructor form, generate a new Overload
                            //     with those parameters inlined at param_idx.
                            for ctor_ov in ctor_overloads {
                                let mut new_ov = ov.clone();
                                // Remove the single param of type_...
                                new_ov.parameters.remove(param_idx);

                                // ...and insert all the constructor params
                                for (i, ctor_param) in ctor_ov.parameters.iter().enumerate() {
                                    new_ov.parameters.insert(param_idx + i, ctor_param.clone());
                                }
                                next_round.push(new_ov);
                            }
                        } else {
                            // If no multiple constructor expansions, just carry forward the overload
                            next_round.push(ov.clone());
                        }
                    }

                    expanded_overloads = next_round;
                    param_idx += 1;
                }

                // Finally, store them all back
                method.overloads = expanded_overloads;
            }
        }

        Ok(())
    }

    pub fn write(&self, path: &str) -> Result<()> {
        let mut output_file = fs::File::create(path)?;

        write_comments(
            &[format!(
                "Generated by {} {}. DO NOT EDIT.",
                built_info::PKG_NAME,
                built_info::PKG_VERSION
            )],
            "",
            &mut output_file,
        )?;
        writeln!(output_file)?;

        for module in self.modules.iter() {
            let verbatim = module.verbatim.join("\n");
            if !verbatim.is_empty() {
                writeln!(output_file, "{verbatim}")?;
            }
        }

        output_methods(&self.functions, true, &mut output_file)?;

        for enum_ in self.enums.iter() {
            let mut comments = enum_.comments.clone();
            add_category_comment(&mut comments, enum_.category.as_deref());

            if let Some(default) = &enum_.default_value {
                comments.push(format!("@defaultValue {default}"));
            }

            if !enum_.platforms.is_empty() {
                comments.push(format!("@platform {}", enum_.platforms));
            }

            if enum_.is_expand {
                comments.push("@expand".to_string());
            }

            write_comments(&comments, "", &mut output_file)?;

            let verbatim = enum_.verbatim.join("\n");
            if verbatim.is_empty() {
                writeln!(output_file, "declare enum {} {{", enum_.name)?;

                for (i, variant) in enum_.variants.iter().enumerate() {
                    let is_first = i == 0;

                    if !is_first {
                        writeln!(output_file)?;
                    }

                    let mut comments = variant.comments.clone();

                    if !variant.platforms.is_empty() {
                        comments.push(format!("@platform {}", variant.platforms));
                    }

                    write_comments(&comments, "    ", &mut output_file)?;

                    writeln!(output_file, "    {},", variant.name)?;
                }

                writeln!(output_file, "}}")?;
            } else {
                writeln!(output_file, "{verbatim}")?;
            }
        }

        for struct_ in self.structs.iter() {
            let mut comments = struct_.comments.clone();
            add_category_comment(&mut comments, struct_.category.as_deref());

            if struct_.is_options {
                comments.push("@expand".to_string());
            }

            if !struct_.platforms.is_empty() {
                comments.push(format!("@platform {}", struct_.platforms));
            }

            write_comments(&comments, "", &mut output_file)?;

            let verbatim = struct_.verbatim.join("\n");
            if verbatim.is_empty() {
                let has_constructor = struct_.methods.iter().any(|method| method.is_constructor);

                let kind = if struct_.methods.is_empty() || !has_constructor {
                    "interface"
                } else {
                    "class"
                };

                writeln!(
                    output_file,
                    "declare {kind} {}{}{} {{",
                    struct_.name,
                    if struct_.is_generic { "<T>" } else { "" },
                    if let Some(name) = &struct_.extends {
                        format!(" extends {name}")
                    } else {
                        String::new()
                    },
                )?;

                for const_ in &struct_.consts {
                    write_comments(&const_.comments, "    ", &mut output_file)?;

                    writeln!(
                        output_file,
                        "    static readonly {}: {};",
                        const_.value, struct_.name
                    )?;
                }

                for property in struct_.properties.iter() {
                    let mut comments = property.comments.clone();

                    if let Some(default) = &property.default_value {
                        comments.push(format!("@defaultValue {default}"));
                    }

                    if !property.platforms.is_empty() {
                        comments.push(format!("@platform {}", property.platforms));
                    }

                    write_comments(&comments, "    ", &mut output_file)?;

                    let optional = if struct_.is_options || property.type_.is_option() {
                        "?"
                    } else {
                        ""
                    };

                    let mut type_ = property.type_.to_string(Context::Property)?;

                    if property.is_readonly_type {
                        if property.type_.is_array() {
                            type_ = format!("readonly {type_}");
                        } else {
                            type_ = format!("Readonly<{type_}>");
                        }
                    }

                    if property.is_promise {
                        type_ = format!("Promise<{type_}>");
                    }

                    writeln!(
                        output_file,
                        "    {}{}{}: {};",
                        if property.is_readonly {
                            "readonly "
                        } else {
                            ""
                        },
                        property.name.to_case(Case::Camel),
                        optional,
                        type_
                    )?;
                }

                for extra_method in &struct_.extra_methods {
                    writeln!(output_file, "    {extra_method};")?;
                }

                output_methods(&struct_.methods, false, &mut output_file)?;

                writeln!(output_file, "}}")?;

                if struct_.is_singleton {
                    let mut comments = Vec::new();
                    add_category_comment(&mut comments, struct_.category.as_deref());
                    write_comments(&comments, "", &mut output_file)?;

                    writeln!(
                        output_file,
                        "declare const {}: {};",
                        struct_.name.to_case(Case::Camel),
                        struct_.name
                    )?;
                }
            } else {
                writeln!(output_file, "{verbatim}")?;
            }
        }

        Ok(())
    }
}

fn output_methods(
    methods: &[Method],
    is_free_function: bool,
    output_file: &mut std::fs::File,
) -> Result<()> {
    let mut methods = methods.to_vec();

    // Make sure constructors are displayed first
    methods.sort_by_key(|method| Reverse(method.is_constructor));

    for method in &methods {
        for overload in &method.overloads {
            let mut parameters = String::new();
            let mut is_first = true;

            if let Some(rest_params) = &overload.rest_params {
                parameters = format!(
                    "...args: {}[]",
                    if let Some(type_) = &rest_params.type_ {
                        &type_
                    } else {
                        "unknown"
                    }
                );
            } else {
                for parameter in &overload.parameters {
                    if parameter.type_.is_ignore() {
                        continue;
                    }

                    if !is_first {
                        use std::fmt::Write;
                        write!(parameters, ", ")?;
                    }

                    is_first = false;

                    use std::fmt::Write;
                    write!(
                        parameters,
                        "{}{}: {}",
                        parameter.name.to_case(Case::Camel),
                        if parameter.type_.is_option() { "?" } else { "" },
                        parameter.type_.to_string(Context::Variable)?
                    )?;
                }
            }

            let mut comments = overload.comments.clone();
            if is_free_function {
                add_category_comment(&mut comments, method.category.as_deref());
            }

            if !overload.platforms.is_empty() {
                comments.push(format!("@platform {}", overload.platforms));
            }

            write_comments(
                &comments,
                if is_free_function { "" } else { "    " },
                output_file,
            )?;

            let private = if method.is_private { "private " } else { "" };

            if method.is_constructor {
                writeln!(output_file, "    {private}constructor({parameters});")?;
            } else {
                let mut return_ = overload.return_.to_string(Context::ReturnValue)?;

                if overload.is_readonly_type {
                    if overload.return_.is_array() {
                        return_ = format!("readonly {return_}");
                    } else {
                        return_ = format!("Readonly<{return_}>");
                    }
                }

                if method.is_async {
                    return_ = format!("Promise<{return_}>");
                }

                let prefix = if is_free_function {
                    "declare function "
                } else {
                    "    "
                };

                writeln!(
                    output_file,
                    "{prefix}{private}{}{}{}({parameters}): {};",
                    if method.is_static && !is_free_function {
                        "static "
                    } else {
                        ""
                    },
                    method.name,
                    if method.is_generic { "<T>" } else { "" },
                    return_
                )?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::write_comments;
    use crate::{
        input::Comments,
        types::{File, Method, MethodOverload, Struct, Type, Variable},
    };

    #[test]
    fn test_overloading() {
        let mut file = File::default();

        // Create a Foo struct that has two constructors: Foo(a: number, b: string) and Foo(p: Foo)
        file.structs.push(Struct {
            name: "Foo".to_string(),
            methods: vec![Method {
                overloads: vec![
                    MethodOverload {
                        comments: Comments::default(),
                        parameters: vec![
                            Variable {
                                name: "a".to_string(),
                                type_: Type::Number,
                                comments: Comments::default(),
                                is_readonly: false,
                                is_readonly_type: false,
                                default_value: None,
                                platforms: Default::default(),
                                is_promise: false,
                            },
                            Variable {
                                name: "b".to_string(),
                                type_: Type::String,
                                comments: Comments::default(),
                                is_readonly: false,
                                is_readonly_type: false,
                                default_value: None,
                                platforms: Default::default(),
                                is_promise: false,
                            },
                        ],
                        return_: Type::Verbatim("Foo".to_string()),
                        is_readonly_type: false,
                        rest_params: None,
                        platforms: Default::default(),
                        constructor_only: false,
                    },
                    MethodOverload {
                        comments: Comments::default(),
                        parameters: vec![Variable {
                            name: "p".to_string(),
                            type_: Type::Verbatim("Foo".to_string()),
                            comments: Comments::default(),
                            is_readonly: false,
                            is_readonly_type: false,
                            default_value: None,
                            platforms: Default::default(),
                            is_promise: false,
                        }],
                        return_: Type::Verbatim("Foo".to_string()),
                        is_readonly_type: false,
                        rest_params: None,
                        platforms: Default::default(),
                        constructor_only: false,
                    },
                ],
                is_constructor: true,
                ..Default::default()
            }],
            ..Default::default()
        });

        // Create a Bar struct that has a constructor method that takes a Foo as a parameter
        file.structs.push(Struct {
            name: "Bar".to_string(),
            methods: vec![Method {
                overloads: vec![MethodOverload {
                    comments: Comments::default(),
                    parameters: vec![Variable {
                        name: "p".to_string(),
                        type_: Type::Verbatim("Foo".to_string()),
                        comments: Comments::default(),
                        is_readonly: false,
                        is_readonly_type: false,
                        default_value: None,
                        platforms: Default::default(),
                        is_promise: false,
                    }],
                    return_: Type::Verbatim("Bar".to_string()),
                    is_readonly_type: false,
                    rest_params: None,
                    platforms: Default::default(),
                    constructor_only: false,
                }],
                is_constructor: true,
                ..Default::default()
            }],
            ..Default::default()
        });

        // Auto generate overloads
        file.auto_generate_overloads().unwrap();

        // Check that Bar as an overload
        let overloads = &file
            .structs
            .iter()
            .find(|struct_| struct_.name == "Bar")
            .unwrap()
            .methods
            .first()
            .unwrap()
            .overloads;

        println!("{overloads:#?}");
    }

    #[test]
    fn test_write_comments_escapes_comment_terminators() {
        let comments = vec!["const match = /HDMI-.*/;".to_string()];
        let mut output = Vec::new();

        write_comments(&comments, "", &mut output).unwrap();

        let output = String::from_utf8(output).unwrap();

        assert_eq!(output, "/**\n * const match = /HDMI-.*\\/;\n */\n");
    }
}
