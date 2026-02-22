use color_eyre::Result;
use convert_case::{Case, Casing};
use log::warn;
use rustdoc_types::{ItemEnum, VariantKind};

use super::{ItemInfo, process_rustdoc};
use crate::{
    items::Items,
    types::{Enum, EnumVariant, RustdocContext},
};

pub fn process_enums(items: &Items) -> Result<Vec<Enum>> {
    let mut result = Vec::new();

    let enums = items
        .iter()
        // Select only Enums
        .filter_map(|item| match &item.inner {
            ItemEnum::Enum(inner) => item.name.as_ref().map(|name| ItemInfo {
                name,
                docs: &item.docs,
                inner,
                item,
            }),
            _ => None,
        });

    for info in enums {
        let (comments, enum_instructions, _) =
            process_rustdoc(info.docs.as_ref(), RustdocContext::Enum)?;

        if enum_instructions.has_skip() {
            continue;
        }

        let enum_name = if let Some(new_name) = enum_instructions.rename() {
            new_name
        } else {
            info.name.to_string()
        };

        let variants = items
            .get_sorted(&info.inner.variants)
            .into_iter()
            // Select only Variants
            .filter_map(|item| match &item.inner {
                ItemEnum::Variant(variant) => {
                    item.name.as_ref().map(|name| (name, &item.docs, variant))
                }
                _ => None,
            })
            // Convert the variant names into constant case
            .map(|(name, docs, function)| (name.to_case(Case::Pascal), docs, function));

        let mut result_variants = Vec::new();

        for (name, docs, variant) in variants {
            if variant.discriminant.is_some() {
                warn!("Discriminants are not supported: {enum_name}::{name}");
                continue;
            }
            if variant.kind != VariantKind::Plain {
                warn!("Only plain variants are supported: {enum_name}::{name}");
                continue;
            }

            let (comments, instructions, _) =
                process_rustdoc(docs.as_ref(), RustdocContext::EnumVariant)?;
            if !instructions.is_empty() {
                warn!("Unexpected instructions: {enum_name}::{name}");
            }

            result_variants.push(EnumVariant {
                name: name.clone(),
                comments,
                platforms: instructions.platforms(),
            });
        }

        let verbatim = enum_instructions.verbatim();
        let category = enum_instructions
            .category()
            .or_else(|| items.category_for_item(info.item));

        // Remove "Js" prefix if present
        let enum_name = enum_name.strip_prefix("Js").unwrap_or(&enum_name);

        let default_value = enum_instructions.default_value();

        result.push(Enum {
            name: enum_name.to_string(),
            variants: result_variants,
            comments,
            category,
            platforms: enum_instructions.platforms(),
            verbatim,
            default_value,
            is_expand: enum_instructions.is_expand(),
        });
    }

    Ok(result)
}
