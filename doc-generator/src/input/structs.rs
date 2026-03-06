use color_eyre::{Result, eyre::Context};
use itertools::Itertools;
use log::warn;
use rustdoc_types::{Id, ItemEnum, StructKind};

use crate::{
    input::{ItemInfo, convert_type, functions::extract_functions, process_rustdoc},
    items::Items,
    types::{Instruction, RustdocContext, Struct, Variable},
};

pub fn process_structs(items: &Items) -> Result<Vec<Struct>> {
    let mut result = Vec::new();

    let structs = items
        .iter()
        .filter(|item| item.links.is_empty()) // We use this to filter out generated structs
        .filter_map(|item| match &item.inner {
            ItemEnum::Struct(inner) => item.name.as_ref().map(|name| ItemInfo {
                name,
                docs: &item.docs,
                inner,
                item,
            }),
            _ => None,
        });

    fn list_properties(
        items: &Items,
        struct_name: &str,
        fields: &[Id],
        struct_docs: &Option<String>,
    ) -> Result<Vec<Variable>> {
        let mut result = Vec::new();

        let (_, instructions, _) = process_rustdoc(struct_docs.as_ref(), RustdocContext::Struct)?;
        let has_properties = instructions
            .iter()
            .any(|instruction| instruction.is_property());

        if !has_properties {
            let fields = items
                .get_sorted(fields)
                .into_iter()
                // Select only Fields
                .filter_map(|item| match &item.inner {
                    ItemEnum::StructField(field) => item
                        .name
                        .as_ref()
                        .map(|name| (name, item.docs.as_ref(), field)),
                    _ => None,
                });

            for (field_name, field_docs, field) in fields {
                let (comments, instructions, _) =
                    process_rustdoc(field_docs, RustdocContext::Property)?;
                if instructions.has_skip() {
                    continue;
                }

                let default_value = instructions.default_value();
                let platforms = instructions.platforms();

                let type_ = if let Some(type_) = instructions.type_() {
                    type_
                } else {
                    convert_type(field, Some(struct_name))?
                };

                result.push(Variable {
                    name: field_name.to_string(),
                    type_,
                    comments,
                    is_readonly: false,
                    is_readonly_type: false,
                    default_value,
                    platforms,
                    is_promise: false,
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

    for info in structs {
        let StructKind::Plain { fields, .. } = &info.inner.kind else {
            warn!("Only plain structs are supported: {}", info.name);
            continue;
        };

        let (struct_comments, struct_instructions, _) =
            process_rustdoc(info.docs.as_ref(), RustdocContext::Struct)?;
        if struct_instructions.has_skip() {
            continue;
        }

        let consts = struct_instructions
            .iter()
            .filter_map(|instruction| {
                if let Instruction::Const(const_) = instruction {
                    Some(const_)
                } else {
                    None
                }
            })
            .cloned()
            .collect_vec();

        let is_singleton = struct_instructions.is_singleton();
        let is_options = struct_instructions.is_options();
        let extends = struct_instructions.extends();
        let is_struct_generic = struct_instructions.is_generic();
        let extra_methods = struct_instructions.extra_methods();
        let verbatim = struct_instructions.verbatim();
        let category = struct_instructions
            .category()
            .or_else(|| items.category_for_item(info.item));

        let mut properties = list_properties(items, info.name, fields, info.docs)?;

        let impls = items
            .get_sorted(&info.inner.impls)
            .into_iter()
            // Select only Impls
            .filter_map(|item| match &item.inner {
                ItemEnum::Impl(impl_) => Some(impl_),
                _ => None,
            })
            // Ignore trait impls
            .filter(|impl_| impl_.trait_.is_none());

        let mut methods = Vec::new();

        for impl_ in impls {
            let (mut impl_methods, mut extra_properties) =
                extract_functions(items, &impl_.items, Some(info.name))
                    .wrap_err_with(|| info.name.to_string())?;
            methods.append(&mut impl_methods);
            properties.append(&mut extra_properties);
        }

        // Remove "Js" prefix if present
        let name = info.name.strip_prefix("Js").unwrap_or(info.name);

        result.push(Struct {
            name: name.to_string(),
            properties,
            methods,
            comments: struct_comments,
            category,
            is_singleton,
            consts,
            is_options,
            extends,
            platforms: struct_instructions.platforms(),
            is_generic: is_struct_generic,
            extra_methods,
            verbatim,
        });
    }

    Ok(result)
}
