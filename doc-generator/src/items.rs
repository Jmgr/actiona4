use std::{collections::BTreeMap, rc::Rc, slice::Iter};

use itertools::Itertools;
use rustdoc_types::{Crate, Id, Item, ItemEnum};

pub struct Items {
    items: Rc<BTreeMap<Id, Item>>,
    js_items: Vec<Item>,
}

impl Items {
    pub fn new(crate_: Crate) -> Self {
        // Store the index into a BTree so we get all entries sorted by ID.
        let items = Rc::new(BTreeMap::from_iter(crate_.index));

        let js_items = items
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

        Self { items, js_items }
    }

    pub fn get(&self, id: Id) -> &Item {
        self.items
            .get(&id)
            .unwrap_or_else(|| panic!("failed to find item with id {id:?}"))
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
