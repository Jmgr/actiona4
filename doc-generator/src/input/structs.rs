use std::collections::HashMap;

use convert_case::{Case, Casing};
use eyre::{Context, Result};
use itertools::Itertools;
use log::{error, warn};
use rustdoc_types::{Id, Item, ItemEnum, StructKind};

use crate::{
    input::{convert_type, process_rustdoc},
    types::{Instruction, Method, MethodOverload, RustdocContext, Struct, Type, Variable},
};

pub fn process_structs<'a, I: Iterator<Item = &'a Item>>(
    items: I,
    index: &HashMap<Id, Item>,
) -> Result<Vec<Struct>> {
    let mut result = Vec::new();

    let structs = items.filter_map(|item| match &item.inner {
        ItemEnum::Struct(struct_) => item.name.as_ref().map(|name| (name, &item.docs, struct_)),
        _ => None,
    });

    fn list_properties(
        struct_name: &str,
        fields: &[Id],
        struct_docs: &Option<String>,
        index: &HashMap<Id, Item>,
    ) -> Result<Vec<Variable>> {
        let mut result = Vec::new();

        let (_, instructions, _) = process_rustdoc(struct_docs.as_ref(), RustdocContext::Struct)?;
        let has_properties = instructions
            .iter()
            .any(|instruction| matches!(instruction, Instruction::Property(_)));

        if !has_properties {
            let fields = fields
                .iter()
                // Get an item reference from an ID
                .filter_map(|id| {
                    if let Some(item) = index.get(id) {
                        Some(item)
                    } else {
                        error!("No item found for ID {id:?}");
                        None
                    }
                })
                // Select only Fields
                .filter_map(|item| match &item.inner {
                    ItemEnum::StructField(field) => {
                        item.name.as_ref().map(|name| (name, &item.docs, field))
                    }
                    _ => None,
                });

            for (field_name, docs, field) in fields {
                let (comments, instructions, _) =
                    process_rustdoc(docs.as_ref(), RustdocContext::Property)?;
                if instructions.has_skip() {
                    continue;
                }

                let default_value = instructions.iter().find_map(|instruction| {
                    if let Instruction::Default(default_value) = instruction {
                        Some(default_value.clone())
                    } else {
                        None
                    }
                });

                result.push(Variable {
                    name: field_name.to_string(),
                    type_: convert_type(field, struct_name)?,
                    comments,
                    is_readonly: false,
                    default_value,
                });
            }
        } else {
            for instruction in instructions.0.into_iter() {
                if let Instruction::Property(property) = instruction {
                    result.push(property);
                }
            }
        }

        Ok(result)
    }

    for (struct_name, struct_docs, struct_) in structs {
        let StructKind::Plain { fields, .. } = &struct_.kind else {
            warn!("Only plain structs are supported: {struct_name}");
            continue;
        };

        let (struct_comments, struct_instructions, _) =
            process_rustdoc(struct_docs.as_ref(), RustdocContext::Struct)?;
        if struct_instructions.has_skip() {
            continue;
        }

        let consts = struct_instructions
            .iter()
            .filter_map(|instruction| {
                if let Instruction::Const(code) = instruction {
                    Some(code)
                } else {
                    None
                }
            })
            .cloned()
            .collect_vec();

        let has_global_instance = struct_instructions.has_global();
        let is_options = struct_instructions.is_options();
        let extends = struct_instructions.extends();

        let properties = list_properties(struct_name, fields, struct_docs, index)?;

        let impls = struct_
            .impls
            .iter()
            // Get an item reference from an ID
            .filter_map(|id| {
                if let Some(item) = index.get(id) {
                    Some(item)
                } else {
                    error!("No item found for ID {id:?}");
                    None
                }
            })
            // Select only Impls
            .filter_map(|item| match &item.inner {
                ItemEnum::Impl(impl_) => Some(impl_),
                _ => None,
            })
            // Ignore trait impls
            .filter(|impl_| impl_.trait_.is_none());

        let mut methods = Vec::new();

        for impl_ in impls {
            let functions = impl_
                .items
                .iter()
                // Get an item reference from an ID
                .filter_map(|id| {
                    if let Some(item) = index.get(id) {
                        Some(item)
                    } else {
                        error!("No item found for ID {id:?}");
                        None
                    }
                })
                // Select only Functions
                .filter_map(|item| match &item.inner {
                    ItemEnum::Function(function) => {
                        item.name.as_ref().map(|name| (name, &item.docs, function))
                    }
                    _ => None,
                })
                // Convert the function names into CamelCase and remove _js suffix
                .map(|(name, docs, function)| {
                    (
                        name.strip_suffix("_js")
                            .unwrap_or(name)
                            .to_case(Case::Camel),
                        docs,
                        function,
                    )
                });

            'func: for (mut function_name, function_docs, function) in functions {
                let (comments, instructions, overload_instructions) =
                    process_rustdoc(function_docs.as_ref(), RustdocContext::Method)?;
                if instructions.has_skip() {
                    continue;
                }

                let has_parameters = instructions
                    .iter()
                    .any(|instruction| matches!(instruction, Instruction::Parameter(_)))
                    || !overload_instructions.is_empty();

                let is_constructor = instructions.has_constructor();
                let is_private = instructions.has_private();
                let mut is_static;
                let rest_params = instructions.rest_params();

                if let Some(new_name) = instructions.rename() {
                    function_name = new_name;
                }

                let mut overloads = Vec::new();

                // No @param instructions
                if !has_parameters {
                    is_static = true;

                    let mut parameters = Vec::new();

                    for (parameter_name, parameter_type) in &function.sig.inputs {
                        let parameter_type = convert_type(parameter_type, struct_name);
                        match parameter_type {
                            Ok(Type::This) => {
                                // If we have a "this" parameter ("self" in Rust) that means we should ignore it, and mark
                                // this method as not being static.
                                is_static = false;
                                continue;
                            }
                            Ok(Type::Ignore) => {
                                continue;
                            }
                            Ok(Type::Unknown) => {
                                if instructions.rest_params().is_some() {
                                    continue;
                                }

                                error!(
                                    "{struct_name}::{function_name}'s parameters should be manually defined."
                                );
                                continue 'func;
                            }
                            Ok(parameter_type) => {
                                parameters.push(Variable {
                                    name: parameter_name.to_string(),
                                    type_: parameter_type,
                                    comments: Default::default(), // TODO
                                    is_readonly: false,
                                    default_value: None,
                                });
                            }
                            Err(err) => error!("{err:?}"),
                        }
                    }

                    let return_ = instructions.iter().find_map(|instruction| {
                        if let Instruction::Returns(type_) = instruction {
                            // TODO: return comments
                            Some(type_.clone())
                        } else {
                            None
                        }
                    });

                    let return_ = if let Some(return_) = return_ {
                        return_
                    } else {
                        let return_ = function
                            .sig
                            .output
                            .as_ref()
                            .map_or(Ok(Type::Void), |output| convert_type(output, struct_name))
                            .wrap_err_with(|| format!("{struct_name}::{function_name}"));

                        match return_ {
                            Ok(return_) => return_,
                            Err(err) => {
                                error!("{err:?}");
                                continue;
                            }
                        }
                    };

                    overloads.push(MethodOverload {
                        parameters,
                        return_,
                        comments,
                        rest_params,
                    });
                } else {
                    is_static = instructions.has_static();

                    let default_result = if instructions.has_constructor() {
                        Type::Verbatim(struct_name.clone())
                    } else {
                        Type::Void
                    };

                    for (instructions, comments) in overload_instructions.iter() {
                        let parameters = instructions
                            .iter()
                            .filter_map(|instruction| {
                                if let Instruction::Parameter(variable) = instruction {
                                    Some(variable.clone())
                                } else {
                                    None
                                }
                            })
                            .collect_vec();

                        let return_ = instructions.iter().find_map(|instruction| {
                            if let Instruction::Returns(type_) = instruction {
                                // TODO: return comments
                                Some(type_.clone())
                            } else {
                                None
                            }
                        });

                        overloads.push(MethodOverload {
                            parameters,
                            return_: return_.unwrap_or(default_result.clone()),
                            comments: comments.clone(),
                            rest_params: rest_params.clone(),
                        });
                    }

                    let parameters = instructions
                        .iter()
                        .filter_map(|instruction| {
                            if let Instruction::Parameter(variable) = instruction {
                                Some(variable.clone())
                            } else {
                                None
                            }
                        })
                        .collect_vec();

                    let return_ = instructions.iter().find_map(|instruction| {
                        if let Instruction::Returns(type_) = instruction {
                            Some(type_.clone())
                        } else {
                            None
                        }
                    });

                    if rest_params.is_some() || !parameters.is_empty() {
                        overloads.push(MethodOverload {
                            parameters,
                            return_: return_.unwrap_or(default_result.clone()),
                            comments: comments.clone(),
                            rest_params,
                        });
                    }
                };

                methods.push(Method {
                    name: function_name.clone(),
                    overloads,
                    is_constructor,
                    is_private,
                    is_static,
                    is_async: function.header.is_async,
                });
            }
        }

        // Remove "Js" prefix if present
        let struct_name = struct_name.strip_prefix("Js").unwrap_or(struct_name);

        result.push(Struct {
            name: struct_name.to_string(),
            properties,
            methods,
            comments: struct_comments,
            has_global_instance,
            consts,
            is_options,
            extends,
        });
    }

    Ok(result)
}
