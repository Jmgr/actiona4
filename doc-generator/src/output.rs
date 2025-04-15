use std::{collections::HashMap, fs, io::Write};

use convert_case::{Case, Casing};
use eyre::Result;
use itertools::Itertools;

use crate::{
    built_info,
    types::{Context, File, Type, strip_modules},
};

fn write_comments<W: Write>(comments: &[String], prefix: &str, file: &mut W) -> Result<()> {
    if comments.is_empty() {
        return Ok(());
    }

    write!(file, "{prefix}")?;
    writeln!(file, "/**")?;

    for comment in comments {
        write!(file, "{prefix}")?;
        writeln!(file, " * {comment}")?;
    }

    write!(file, "{prefix}")?;
    writeln!(file, " */")?;

    Ok(())
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
                Context::Variable | Context::Property => (option.to_string(context)?).to_string(),

                // If it's a return value then we have to use the "| undefined" syntax instead
                Context::ReturnValue => format!("{} | undefined", option.to_string(context)?),
            },
            Type::Verbatim(type_) => {
                let type_ = strip_modules(type_);

                // Remove "Js" prefix and "Param" suffix if present
                let type_ = type_.strip_prefix("Js").unwrap_or(type_);
                let type_ = type_.strip_suffix("Param").unwrap_or(type_);

                type_.to_string()
            }
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
            let maybe_constructor_method = s
                .methods
                .iter()
                .find(|m| m.is_constructor)
                .map(|m| m.overloads.clone());

            let Some(overloads) = maybe_constructor_method else {
                // No constructor method on this type => skip
                continue;
            };

            // We only want to store it if it has multiple ways to construct
            if overloads.len() > 1 {
                constructors_for_type.insert(
                    s.name.clone(),
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
                            // (a) Keep the original overload:
                            next_round.push(ov.clone());

                            // (b) For each constructor form, generate a new Overload
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
        let mut output_file = fs::File::create(path)?; // TODO: write to a buffer, then to a file

        write_comments(
            &[format!(
                "Generated by {} {}. DO NOT EDIT.",
                built_info::PKG_NAME,
                built_info::PKG_VERSION
            )],
            "",
            &mut output_file,
        )?;

        for enum_ in self.enums.iter() {
            write_comments(&enum_.comments, "", &mut output_file)?;

            writeln!(output_file, "declare enum {} {{", enum_.name)?;

            for (i, variant) in enum_.variants.iter().enumerate() {
                let is_first = i == 0;

                if !is_first {
                    writeln!(output_file)?;
                }

                write_comments(&variant.comments, "    ", &mut output_file)?;

                writeln!(output_file, "    {},", variant.name)?;
            }

            writeln!(output_file, "}}")?;
        }

        for struct_ in self.structs.iter() {
            write_comments(&struct_.comments, "", &mut output_file)?;

            let has_constructor = struct_.methods.iter().any(|method| method.is_constructor);

            writeln!(
                output_file,
                "declare {} {}{} {{",
                if struct_.methods.is_empty() || !has_constructor {
                    "interface"
                } else {
                    "class"
                },
                struct_.name,
                if let Some(name) = &struct_.extends {
                    format!(" extends {name}")
                } else {
                    String::new()
                },
            )?;

            for const_ in &struct_.consts {
                writeln!(
                    output_file,
                    "    static readonly {}: {};",
                    const_, struct_.name
                )?;
            }

            for property in struct_.properties.iter() {
                let mut comments = property.comments.clone();

                if let Some(default) = &property.default_value {
                    comments.push(format!("@defaultValue {default}"));
                }

                write_comments(&comments, "    ", &mut output_file)?;

                let optional = if struct_.is_options { "?" } else { "" };

                writeln!(
                    output_file,
                    "    {}{}{}: {};",
                    if property.is_readonly {
                        "readonly "
                    } else {
                        ""
                    },
                    property.name,
                    optional,
                    property.type_.to_string(Context::Property)?
                )?;
            }

            // Used for overloading methods
            let mut methods = struct_.methods.clone();

            // Make sure constructors are displayed first
            methods.sort_by(|a, b| b.is_constructor.cmp(&a.is_constructor));

            for method in &methods {
                for overload in &method.overloads {
                    let mut parameters = String::new();
                    let mut is_first = true;

                    if overload.has_rest_params {
                        parameters = "...args: any[]".to_string();
                    } else {
                        for parameter in &overload.parameters {
                            if matches!(parameter.type_, Type::Ignore) {
                                continue;
                            }

                            if !is_first {
                                use std::fmt::Write;
                                write!(parameters, ", ")?;
                            }

                            is_first = false;

                            let is_optional = matches!(parameter.type_, Type::Option(_));

                            use std::fmt::Write;
                            write!(
                                parameters,
                                "{}{}: {}",
                                parameter.name,
                                if is_optional { "?" } else { "" },
                                parameter.type_.to_string(Context::Variable)?
                            )?;
                        }
                    }

                    write_comments(&overload.comments, "    ", &mut output_file)?;
                    if method.is_constructor {
                        writeln!(output_file, "    constructor({parameters});")?;
                    } else {
                        writeln!(
                            output_file,
                            "    {}{}({parameters}): {};",
                            if method.is_static { "static " } else { "" },
                            method.name,
                            overload.return_.to_string(Context::ReturnValue)?
                        )?;
                    }
                }
            }

            writeln!(output_file, "}}")?;

            if struct_.has_global_instance {
                writeln!(
                    output_file,
                    "declare const {}: {};",
                    struct_.name.to_case(Case::Snake),
                    struct_.name
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
                                default_value: None,
                            },
                            Variable {
                                name: "b".to_string(),
                                type_: Type::String,
                                comments: Comments::default(),
                                is_readonly: false,
                                default_value: None,
                            },
                        ],
                        return_: Type::Verbatim("Foo".to_string()),
                        has_rest_params: false,
                    },
                    MethodOverload {
                        comments: Comments::default(),
                        parameters: vec![Variable {
                            name: "p".to_string(),
                            type_: Type::Verbatim("Foo".to_string()),
                            comments: Comments::default(),
                            is_readonly: false,
                            default_value: None,
                        }],
                        return_: Type::Verbatim("Foo".to_string()),
                        has_rest_params: false,
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
                        default_value: None,
                    }],
                    return_: Type::Verbatim("Bar".to_string()),
                    has_rest_params: false,
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
}
