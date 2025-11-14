use color_eyre::{
    Result,
    eyre::{Context, eyre},
};
use convert_case::{Case, Casing};
use itertools::Itertools;
use log::error;
use rustdoc_types::{Id, ItemEnum};

use crate::{
    input::{convert_type, process_rustdoc},
    items::Items,
    types::{Instruction, Method, MethodOverload, RustdocContext, Type, Variable},
};

pub fn process_functions(items: &Items) -> Result<Vec<Method>> {
    let functions = items
        .iter()
        // Select only modules
        .filter_map(|item| {
            if let ItemEnum::Module(module) = &item.inner {
                Some(module)
            } else {
                None
            }
        })
        .flat_map(|module| module.items.clone())
        .collect_vec();

    let (functions, _) = extract_functions(items, &functions, None)?;
    Ok(functions)
}

pub fn extract_functions(
    items: &Items,
    function_ids: &[Id],
    struct_name: Option<&str>,
) -> Result<(Vec<Method>, Vec<Variable>)> {
    let mut methods = Vec::new();
    let mut properties = Vec::new();

    let functions = function_ids
        .iter()
        // Get an item reference from an ID
        .map(|id| items.get(*id))
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
            .any(|instruction| instruction.is_parameter())
            || !overload_instructions.is_empty();

        let is_constructor = instructions.has_constructor();
        let is_private = instructions.has_private();
        let mut is_static;
        let rest_params = instructions.rest_params();
        let is_generic = instructions.is_generic();

        if let Some(new_name) = instructions.rename() {
            function_name = new_name;
        }

        let mut overloads = Vec::new();

        // No @param instructions
        if !has_parameters {
            is_static = true;

            let mut parameters = Vec::new();

            for (parameter_name, parameter_type) in &function.sig.inputs {
                let parameter_type = convert_type(parameter_type, struct_name)
                    .wrap_err_with(|| function_name.clone());
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

                        error!("{function_name}'s parameters should be manually defined.");
                        continue 'func;
                    }
                    Ok(parameter_type) => {
                        parameters.push(Variable {
                            name: parameter_name.to_string(),
                            type_: parameter_type,
                            comments: Default::default(), // TODO
                            is_readonly: false,
                            default_value: None,
                            platforms: instructions.platforms(),
                            is_promise: false,
                        });
                    }
                    Err(err) => error!("{err:?}"),
                }
            }

            let return_ = instructions.returns();

            let return_ = if let Some(return_) = return_ {
                return_
            } else {
                let return_ = function
                    .sig
                    .output
                    .as_ref()
                    .map_or(Ok(Type::Void), |output| convert_type(output, struct_name))
                    .wrap_err_with(|| function_name.to_string());

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
                comments: comments.clone(),
                rest_params,
                platforms: instructions.platforms(),
            });
        } else {
            is_static = instructions.has_static();

            let default_result = if instructions.has_constructor() {
                Type::Verbatim(
                    struct_name
                        .ok_or_else(|| {
                            eyre!("expected struct name, but none set (free function?)")
                        })?
                        .to_string(),
                )
            } else {
                function
                    .sig
                    .output
                    .as_ref()
                    .map_or(Ok(Type::Void), |output| convert_type(output, struct_name))
                    .wrap_err_with(|| function_name.to_string())?
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

                let return_ = instructions.returns();

                overloads.push(MethodOverload {
                    parameters,
                    return_: return_.unwrap_or(default_result.clone()),
                    comments: comments.clone(),
                    rest_params: rest_params.clone(),
                    platforms: instructions.platforms(),
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

            let return_ = instructions.returns();

            if rest_params.is_some() || !parameters.is_empty() {
                overloads.push(MethodOverload {
                    parameters,
                    return_: return_.unwrap_or(default_result.clone()),
                    comments: comments.clone(),
                    rest_params,
                    platforms: instructions.platforms(),
                });
            }
        };

        // If the function is @get then it appears as a property, so we skip it
        if instructions.has_getter() {
            properties.push(Variable {
                name: function_name,
                type_: overloads[0].return_.clone(),
                comments,
                is_readonly: true,
                default_value: None,
                platforms: Default::default(),
                is_promise: function.header.is_async,
            });
            continue;
        }

        methods.push(Method {
            name: function_name.clone(),
            overloads,
            is_constructor,
            is_private,
            is_static,
            is_async: function.header.is_async,
            is_generic,
        });
    }

    Ok((methods, properties))
}
