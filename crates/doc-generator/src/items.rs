use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::Path,
    rc::Rc,
    slice::Iter,
};

use convert_case::{Case, Casing};
use itertools::Itertools;
use rustdoc_types::{Crate, Id, Item, ItemEnum, StructKind, VariantKind};

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
    ignored_ids: Rc<BTreeSet<Id>>,
    js_items: Vec<Item>,
}

impl Items {
    pub fn new(crate_: Crate) -> Self {
        // Store the index into a BTree so we can look up items by ID.
        let items = Rc::new(BTreeMap::from_iter(crate_.index));
        let ignored_ids = Rc::new(Self::collect_ignored_ids(&items));

        let mut js_items: Vec<Item> = items
            .values()
            // Ignore items that live in a `tests` module.
            .filter(|item| !ignored_ids.contains(&item.id))
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

        Self {
            items,
            ignored_ids,
            js_items,
        }
    }

    fn collect_ignored_ids(items: &BTreeMap<Id, Item>) -> BTreeSet<Id> {
        let mut ignored_ids = BTreeSet::new();
        let mut pending = VecDeque::new();

        for item in items.values() {
            if item.name.as_deref() == Some("tests") && matches!(&item.inner, ItemEnum::Module(_)) {
                pending.push_back(item.id);
            }
        }

        while let Some(id) = pending.pop_front() {
            if !ignored_ids.insert(id) {
                continue;
            }

            let Some(item) = items.get(&id) else {
                continue;
            };

            pending.extend(Self::contained_items(item));
        }

        ignored_ids
    }

    fn contained_items(item: &Item) -> Vec<Id> {
        match &item.inner {
            ItemEnum::Module(module) => module.items.clone(),
            ItemEnum::Union(union_) => union_
                .fields
                .iter()
                .chain(union_.impls.iter())
                .copied()
                .collect_vec(),
            ItemEnum::Struct(struct_) => {
                let fields = match &struct_.kind {
                    StructKind::Unit => Vec::new(),
                    StructKind::Tuple(fields) => fields.iter().flatten().copied().collect_vec(),
                    StructKind::Plain { fields, .. } => fields.clone(),
                };

                fields
                    .into_iter()
                    .chain(struct_.impls.iter().copied())
                    .collect_vec()
            }
            ItemEnum::Enum(enum_) => enum_
                .variants
                .iter()
                .chain(enum_.impls.iter())
                .copied()
                .collect_vec(),
            ItemEnum::Variant(variant) => match &variant.kind {
                VariantKind::Plain => Vec::new(),
                VariantKind::Tuple(fields) => fields.iter().flatten().copied().collect_vec(),
                VariantKind::Struct { fields, .. } => fields.clone(),
            },
            ItemEnum::Trait(trait_) => trait_
                .items
                .iter()
                .chain(trait_.implementations.iter())
                .copied()
                .collect_vec(),
            ItemEnum::Impl(impl_) => impl_.items.clone(),
            _ => Vec::new(),
        }
    }

    pub fn get(&self, id: Id) -> &Item {
        self.items
            .get(&id)
            .unwrap_or_else(|| panic!("failed to find item with id {id:?}"))
    }

    /// Resolve a list of IDs and return the items sorted by source span.
    pub fn get_sorted(&self, ids: &[Id]) -> Vec<&Item> {
        let mut items: Vec<&Item> = ids
            .iter()
            .filter(|id| !self.ignored_ids.contains(id))
            .map(|id| self.get(*id))
            .collect();
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
                    if self.ignored_ids.contains(&path.id) {
                        return None;
                    }

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
            ignored_ids: self.ignored_ids.clone(),
            js_items,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashMap},
        path::Path,
    };

    use rustdoc_types::{
        Abi, Function, FunctionHeader, FunctionSignature, Generics, Id, Impl, Item, ItemEnum,
        Module, Type, Visibility,
    };

    use super::Items;

    #[test]
    fn test_category_from_filename() {
        assert_eq!(
            Items::category_from_filename(Path::new("src/api/mouse/js.rs")),
            Some("Mouse".to_string())
        );
        assert_eq!(
            Items::category_from_filename(Path::new("src/api/js/mod.rs")),
            Some("Core".to_string())
        );
        assert_eq!(
            Items::category_from_filename(Path::new("src/api/dialogs/js.rs")),
            Some("Dialogs".to_string())
        );
        assert_eq!(
            Items::category_from_filename(Path::new("src/api/standardpaths/js.rs")),
            Some("Standardpaths".to_string())
        );
        assert_eq!(
            Items::category_from_filename(Path::new("src/api/js/abort_controller.rs")),
            Some("Core".to_string())
        );
        assert_eq!(
            Items::category_from_filename(Path::new("src/something/mod.rs")),
            Some("Something".to_string())
        );
        assert_eq!(Items::category_from_filename(Path::new("lib.rs")), None);
    }

    fn empty_generics() -> Generics {
        Generics {
            params: Vec::new(),
            where_predicates: Vec::new(),
        }
    }

    fn make_module_item(id: u32, name: Option<&str>, module_items: Vec<Id>) -> Item {
        Item {
            id: Id(id),
            crate_id: 0,
            name: name.map(str::to_string),
            span: None,
            visibility: Visibility::Public,
            docs: None,
            links: HashMap::new(),
            attrs: Vec::new(),
            deprecation: None,
            inner: ItemEnum::Module(Module {
                is_crate: false,
                items: module_items,
                is_stripped: false,
            }),
        }
    }

    fn make_function_item(id: u32, name: &str) -> Item {
        Item {
            id: Id(id),
            crate_id: 0,
            name: Some(name.to_string()),
            span: None,
            visibility: Visibility::Public,
            docs: None,
            links: HashMap::new(),
            attrs: Vec::new(),
            deprecation: None,
            inner: ItemEnum::Function(Function {
                sig: FunctionSignature {
                    inputs: Vec::new(),
                    output: None,
                    is_c_variadic: false,
                },
                generics: empty_generics(),
                header: FunctionHeader {
                    is_const: false,
                    is_unsafe: false,
                    is_async: false,
                    abi: Abi::Rust,
                },
                has_body: true,
            }),
        }
    }

    fn make_impl_item(id: u32, impl_items: Vec<Id>) -> Item {
        Item {
            id: Id(id),
            crate_id: 0,
            name: None,
            span: None,
            visibility: Visibility::Public,
            docs: None,
            links: HashMap::new(),
            attrs: Vec::new(),
            deprecation: None,
            inner: ItemEnum::Impl(Impl {
                is_unsafe: false,
                generics: empty_generics(),
                provided_trait_methods: Vec::new(),
                trait_: None,
                for_: Type::Primitive("i32".to_string()),
                items: impl_items,
                is_negative: false,
                is_synthetic: false,
                blanket_impl: None,
            }),
        }
    }

    #[test]
    fn test_collect_ignored_ids_from_tests_module_subtree() {
        let items = BTreeMap::from([
            (
                Id(1),
                make_module_item(Id(1).0, Some("root"), vec![Id(2), Id(5), Id(8)]),
            ),
            (
                Id(2),
                make_module_item(Id(2).0, Some("tests"), vec![Id(3), Id(4)]),
            ),
            (Id(3), make_function_item(Id(3).0, "test_only_function")),
            (Id(4), make_impl_item(Id(4).0, vec![Id(7)])),
            (
                Id(5),
                make_module_item(Id(5).0, Some("api"), vec![Id(6), Id(9)]),
            ),
            (Id(6), make_function_item(Id(6).0, "public_function")),
            (Id(7), make_function_item(Id(7).0, "test_only_method")),
            (
                Id(8),
                make_module_item(Id(8).0, Some("nested"), vec![Id(10)]),
            ),
            (
                Id(9),
                make_module_item(Id(9).0, Some("tests"), vec![Id(11)]),
            ),
            (Id(10), make_function_item(Id(10).0, "still_public")),
            (Id(11), make_function_item(Id(11).0, "nested_test_function")),
        ]);

        let ignored_ids = Items::collect_ignored_ids(&items);

        assert!(ignored_ids.contains(&Id(2)));
        assert!(ignored_ids.contains(&Id(3)));
        assert!(ignored_ids.contains(&Id(4)));
        assert!(ignored_ids.contains(&Id(7)));
        assert!(ignored_ids.contains(&Id(9)));
        assert!(ignored_ids.contains(&Id(11)));

        assert!(!ignored_ids.contains(&Id(1)));
        assert!(!ignored_ids.contains(&Id(5)));
        assert!(!ignored_ids.contains(&Id(6)));
        assert!(!ignored_ids.contains(&Id(8)));
        assert!(!ignored_ids.contains(&Id(10)));
    }
}
