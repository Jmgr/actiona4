use std::{collections::BTreeMap, rc::Rc, slice::Iter};

use itertools::Itertools;
use rustdoc_types::{Crate, Id, Item, ItemEnum};

/// Compare two items by their source span (filename, then line, then column).
/// Items without a span are sorted to the end.
pub fn cmp_by_span(a: &Item, b: &Item) -> std::cmp::Ordering {
    match (&a.span, &b.span) {
        (Some(a_span), Some(b_span)) => a_span
            .filename
            .cmp(&b_span.filename)
            .then(a_span.begin.cmp(&b_span.begin)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

pub struct Items {
    items: Rc<BTreeMap<Id, Item>>,
    js_items: Vec<Item>,
}

impl Items {
    pub fn new(crate_: Crate) -> Self {
        // Store the index into a BTree so we can look up items by ID.
        let items = Rc::new(BTreeMap::from_iter(crate_.index));

        let mut js_items: Vec<Item> = items
            .values()
            // From a js.rs file, or in a js directory
            .filter(|item| {
                item.span.as_ref().is_some_and(|span| {
                    span.filename.ends_with("js.rs")
                        || span.filename.to_str().unwrap().contains("/js/")
                })
            })
            // With a name that doesn't start with _
            .filter(|item| {
                item.name
                    .as_ref()
                    .is_some_and(|name| !name.starts_with("_"))
            })
            .cloned()
            .collect_vec();

        // Sort by source location so the output order matches the Rust source order.
        js_items.sort_by(cmp_by_span);

        Self { items, js_items }
    }

    pub fn get(&self, id: Id) -> &Item {
        self.items
            .get(&id)
            .unwrap_or_else(|| panic!("failed to find item with id {id:?}"))
    }

    /// Resolve a list of IDs and return the items sorted by source span.
    pub fn get_sorted(&self, ids: &[Id]) -> Vec<&Item> {
        let mut items: Vec<&Item> = ids.iter().map(|id| self.get(*id)).collect();
        items.sort_by(|a, b| cmp_by_span(a, b));
        items
    }

    pub fn iter(&'_ self) -> Iter<'_, Item> {
        self.js_items.iter()
    }

    pub fn aliases(&self) -> Self {
        let js_items = self
            .js_items
            .iter()
            .filter_map(|item| match &item.inner {
                ItemEnum::TypeAlias(alias) => {
                    let rustdoc_types::Type::ResolvedPath(path) = &alias.type_ else {
                        return None;
                    };

                    // Clone the target item but preserve the alias's name so we emit the aliased
                    // identifier in the TS output instead of the underlying target name.
                    let mut target = self.get(path.id).clone();
                    target.name = item.name.clone().or_else(|| target.name.clone());
                    Some(target)
                }
                _ => None,
            })
            .collect_vec();

        Self {
            items: self.items.clone(),
            js_items,
        }
    }
}
