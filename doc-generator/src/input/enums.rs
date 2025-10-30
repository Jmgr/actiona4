use convert_case::{Case, Casing};
use eyre::Result;
use log::warn;
use rustdoc_types::{ItemEnum, VariantKind};

use super::process_rustdoc;
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
            ItemEnum::Enum(enum_) => item.name.as_ref().map(|name| (name, &item.docs, enum_)),
            _ => None,
        });

    for (enum_name, enum_docs, enum_) in enums {
        let variants = enum_
            .variants
            .iter()
            // Get an item reference from an ID
            .map(|id| items.get(*id))
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

        let (comments, enum_instructions, _) =
            process_rustdoc(enum_docs.as_ref(), RustdocContext::Enum)?;
        let verbatim = enum_instructions.verbatim();

        // Remove "Js" prefix if present
        let enum_name = enum_name.strip_prefix("Js").unwrap_or(enum_name);

        result.push(Enum {
            name: enum_name.to_string(),
            variants: result_variants,
            comments,
            platforms: enum_instructions.platforms(),
            verbatim,
        });
    }

    Ok(result)
}
