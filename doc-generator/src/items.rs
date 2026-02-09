use std::{collections::BTreeMap, path::Path, rc::Rc, slice::Iter};

use convert_case::{Case, Casing};
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

    pub fn category_for_item(&self, item: &Item) -> Option<String> {
        item.span
            .as_ref()
            .and_then(|span| Self::category_from_filename(span.filename.as_path()))
    }

    fn category_from_filename(path: &Path) -> Option<String> {
        let segments = path
            .components()
            .filter_map(|component| component.as_os_str().to_str())
            .collect_vec();

        if let Some(core_index) = segments.iter().position(|segment| *segment == "api")
            && let Some(core_child) = segments.get(core_index + 1)
        {
            if *core_child == "js" {
                return Some("Core".to_string());
            }

            return Some(core_child.to_case(Case::Pascal));
        }

        if let Some(src_index) = segments.iter().position(|segment| *segment == "src")
            && let Some(src_child) = segments.get(src_index + 1)
        {
            return Some(src_child.to_case(Case::Pascal));
        }

        None
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Items;

    #[test]
    fn test_category_from_filename() {
        assert_eq!(
            Items::category_from_filename(Path::new("src/core/mouse/js.rs")),
            Some("Mouse".to_string())
        );
        assert_eq!(
            Items::category_from_filename(Path::new("src/core/js/mod.rs")),
            Some("Core".to_string())
        );
        assert_eq!(
            Items::category_from_filename(Path::new("src/core/ui/js.rs")),
            Some("Ui".to_string())
        );
        assert_eq!(
            Items::category_from_filename(Path::new("src/core/standardpaths/js.rs")),
            Some("Standardpaths".to_string())
        );
        assert_eq!(
            Items::category_from_filename(Path::new("src/core/js/abort_controller.rs")),
            Some("Core".to_string())
        );
        assert_eq!(
            Items::category_from_filename(Path::new("src/something/mod.rs")),
            Some("Something".to_string())
        );
        assert_eq!(Items::category_from_filename(Path::new("lib.rs")), None);
    }
}
