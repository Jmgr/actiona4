use eyre::Result;
use rustdoc_types::ItemEnum;

use crate::{
    input::process_rustdoc,
    items::Items,
    types::{Module, RustdocContext},
};

pub fn process_modules(items: &Items) -> Result<Vec<Module>> {
    let mut result = Vec::new();

    let modules = items.iter().filter_map(|item| match &item.inner {
        ItemEnum::Module(module) => item.name.as_ref().map(|name| (name, &item.docs, module)),
        _ => None,
    });

    for (_module_name, module_docs, _module_) in modules {
        let (_module_comments, module_instructions, _) =
            process_rustdoc(module_docs.as_ref(), RustdocContext::Module)?;
        if module_instructions.has_skip() {
            continue;
        }

        let verbatim = module_instructions.verbatim();

        result.push(Module { verbatim });
    }

    Ok(result)
}
