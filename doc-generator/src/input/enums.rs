use std::collections::HashMap;

use convert_case::{Case, Casing};
use eyre::Result;
use log::{error, warn};
use rustdoc_types::{Id, Item, ItemEnum, VariantKind};

use super::process_rustdoc;
use crate::types::{Enum, EnumVariant, RustdocContext};

pub fn process_enums<'a, I: Iterator<Item = &'a Item>>(
    items: I,
    index: &HashMap<Id, Item>,
) -> Result<Vec<Enum>> {
    let mut result = Vec::new();

    let enums = items
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
            .filter_map(|id| {
                if let Some(item) = index.get(id) {
                    Some(item)
                } else {
                    error!("No item found for ID {id:?}");
                    None
                }
            })
            // Select only Variants
            .filter_map(|item| match &item.inner {
                ItemEnum::Variant(variant) => {
                    item.name.as_ref().map(|name| (name, &item.docs, variant))
                }
                _ => None,
            })
            // Convert the variant names into constant case
            .map(|(name, docs, function)| (name.to_case(Case::Constant), docs, function));

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
            });
        }

        let (comments, instructions, _) =
            process_rustdoc(enum_docs.as_ref(), RustdocContext::Enum)?;
        if !instructions.is_empty() {
            warn!("Unexpected instructions: {enum_name}");
        }

        // Remove "Js" prefix if present
        let enum_name = enum_name.strip_prefix("Js").unwrap_or(enum_name);

        result.push(Enum {
            name: enum_name.to_string(),
            variants: result_variants,
            comments,
        });
    }

    Ok(result)
}
