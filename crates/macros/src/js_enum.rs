use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, ItemEnum, LitStr, parse_macro_input, parse_quote};

use crate::{
    consts::{INSTR_PLATFORMS, INSTR_RENAME, JS_TYPE_PREFIX},
    default_args::{doc_contains, parse_meta_list_tokens},
    options::{append_doc_line, platform_not_from_attributes, platform_only_from_attributes},
};

#[derive(Debug, Default, FromMeta)]
struct JsEnumArguments {
    rename: Option<String>,
}

/// Expand `#[js_enum]` for enum items.
pub(crate) fn expand(arguments: TokenStream, item: TokenStream) -> TokenStream {
    let args_tokens = proc_macro2::TokenStream::from(arguments);
    let args: JsEnumArguments =
        match parse_meta_list_tokens(args_tokens, proc_macro2::Span::call_site()) {
            Ok(args) => args,
            Err(error) => return error.to_compile_error().into(),
        };

    let mut item = parse_macro_input!(item as Item);
    let Item::Enum(item_enum) = &mut item else {
        return syn::Error::new_spanned(&item, "`#[js_enum]` only supports enums")
            .to_compile_error()
            .into();
    };

    if let Err(error) = expand_enum(item_enum, args.rename) {
        return error.to_compile_error().into();
    }

    quote!(#item).into()
}

fn expand_enum(item_enum: &mut ItemEnum, rename: Option<String>) -> syn::Result<()> {
    // Platform annotations on variants
    for variant in &mut item_enum.variants {
        if doc_contains(&variant.attrs, INSTR_PLATFORMS) {
            continue;
        }
        if let Some(platform_name) = platform_only_from_attributes(&variant.attrs)? {
            append_doc_line(
                &mut variant.attrs,
                format!("{INSTR_PLATFORMS} ={platform_name}"),
            );
        } else if let Some(platform_name) = platform_not_from_attributes(&variant.attrs)? {
            append_doc_line(
                &mut variant.attrs,
                format!("{INSTR_PLATFORMS} -{platform_name}"),
            );
        }
    }

    // Determine the JS name:
    // - explicit `rename` param → use it and inject @rename
    // - `Js` prefix → strip it, no @rename
    // - no prefix, no rename → keep the Rust enum name
    let enum_name = item_enum.ident.to_string();
    let (serde_name, inject_rename_instr) = if let Some(rename) = rename {
        (Some(rename), true)
    } else if let Some(stripped) = enum_name.strip_prefix(JS_TYPE_PREFIX) {
        if stripped.is_empty() {
            return Err(syn::Error::new_spanned(
                &item_enum.ident,
                "`#[js_enum]` expects characters after the `Js` prefix",
            ));
        }
        (Some(stripped.to_string()), false)
    } else {
        (None, false)
    };

    if let Some(name) = serde_name {
        let rename_lit = LitStr::new(&name, item_enum.ident.span());
        let serde_attr = parse_quote! { #[serde(rename = #rename_lit)] };
        item_enum.attrs.push(serde_attr);

        if inject_rename_instr && !doc_contains(&item_enum.attrs, INSTR_RENAME) {
            append_doc_line(&mut item_enum.attrs, format!("{INSTR_RENAME} {name}"));
        }
    }

    Ok(())
}
