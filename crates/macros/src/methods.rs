use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Attribute, ImplItem, ImplItemFn, ItemImpl, LitStr, Meta, Token, parse::Parser,
    parse_macro_input, parse_quote, punctuated::Punctuated,
};

use crate::{
    consts::{INSTR_GET, RENAME_ALL_CAMEL_CASE},
    default_args::doc_contains,
};

/// Expand `#[js_methods]` into `#[rquickjs::methods]`, processing helper attributes.
pub(crate) fn expand(arguments: TokenStream, item: TokenStream) -> TokenStream {
    let arguments = proc_macro2::TokenStream::from(arguments);
    let mut item_impl = parse_macro_input!(item as ItemImpl);

    for impl_item in item_impl.items.iter_mut() {
        if let ImplItem::Fn(method) = impl_item
            && let Err(error) = apply_accessor_attributes(method)
        {
            return error.to_compile_error().into();
        }
    }

    let final_arguments = normalize_methods_arguments(&arguments);
    if final_arguments.is_empty() {
        quote!(#[rquickjs::methods] #item_impl).into()
    } else {
        quote!(#[rquickjs::methods(#final_arguments)] #item_impl).into()
    }
}

/// Convert `#[get]` / `#[set]` / `#[prop]` into rquickjs accessor/property
/// attributes plus the rustdoc markers consumed by the doc generator.
fn apply_accessor_attributes(method: &mut ImplItemFn) -> syn::Result<()> {
    let mut saw_get = false;
    let mut saw_set = false;
    let mut saw_prop = false;
    let mut get_rename: Option<LitStr> = None;
    let mut set_rename: Option<LitStr> = None;
    let mut prop_rename: Option<LitStr> = None;

    let mut new_attrs = Vec::with_capacity(method.attrs.len());
    for attribute in method.attrs.drain(..) {
        if attribute.path().is_ident("get") {
            if saw_get {
                return Err(syn::Error::new_spanned(
                    attribute,
                    "duplicate `#[get]` attribute",
                ));
            }
            saw_get = true;
            get_rename = parse_accessor_attribute(&attribute, "get")?;
            continue;
        }
        if attribute.path().is_ident("set") {
            if saw_set {
                return Err(syn::Error::new_spanned(
                    attribute,
                    "duplicate `#[set]` attribute",
                ));
            }
            saw_set = true;
            set_rename = parse_accessor_attribute(&attribute, "set")?;
            continue;
        }
        if attribute.path().is_ident("prop") {
            if saw_prop {
                return Err(syn::Error::new_spanned(
                    attribute,
                    "duplicate `#[prop]` attribute",
                ));
            }
            saw_prop = true;
            prop_rename = parse_accessor_attribute(&attribute, "prop")?;
            continue;
        }

        new_attrs.push(attribute);
    }
    method.attrs = new_attrs;

    if !saw_get && !saw_set && !saw_prop {
        return Ok(());
    }

    if [saw_get, saw_set, saw_prop]
        .into_iter()
        .filter(|saw| *saw)
        .count()
        > 1
    {
        return Err(syn::Error::new_spanned(
            method.sig.ident.clone(),
            "`#[get]`, `#[set]`, and `#[prop]` cannot be combined on the same method",
        ));
    }

    if saw_get {
        if has_qjs_flag(&method.attrs, "get") {
            return Err(syn::Error::new_spanned(
                method.sig.ident.clone(),
                "`#[get]` conflicts with existing `#[qjs(get)]` attribute",
            ));
        }

        if !doc_contains(&method.attrs, INSTR_GET) {
            append_doc_line(&mut method.attrs, INSTR_GET.to_string());
        }

        let qjs_attribute: Attribute = match get_rename {
            Some(literal) => parse_quote!(#[qjs(get, rename = #literal)]),
            None => parse_quote!(#[qjs(get)]),
        };

        method.attrs.push(qjs_attribute);
    }

    if saw_prop {
        if has_qjs_flag(&method.attrs, "prop") {
            return Err(syn::Error::new_spanned(
                method.sig.ident.clone(),
                "`#[prop]` conflicts with existing `#[qjs(prop)]` attribute",
            ));
        }

        if !doc_contains(&method.attrs, INSTR_GET) {
            append_doc_line(&mut method.attrs, INSTR_GET.to_string());
        }

        let qjs_attribute: Attribute = match prop_rename {
            Some(literal) => parse_quote!(#[qjs(prop, rename = #literal)]),
            None => parse_quote!(#[qjs(prop)]),
        };

        method.attrs.push(qjs_attribute);
    }

    if saw_set {
        if has_qjs_flag(&method.attrs, "set") {
            return Err(syn::Error::new_spanned(
                method.sig.ident.clone(),
                "`#[set]` conflicts with existing `#[qjs(set)]` attribute",
            ));
        }

        let qjs_attribute: Attribute = match set_rename {
            Some(literal) => parse_quote!(#[qjs(set, rename = #literal)]),
            None => parse_quote!(#[qjs(set)]),
        };

        method.attrs.push(qjs_attribute);
    }

    Ok(())
}

/// Parse `#[get("name")]`, `#[set("name")]`, or `#[prop("name")]` into a rename literal.
fn parse_accessor_attribute(attribute: &Attribute, name: &str) -> syn::Result<Option<LitStr>> {
    match &attribute.meta {
        Meta::Path(_) => Ok(None),
        Meta::List(meta_list) => {
            let literal: LitStr = syn::parse2(meta_list.tokens.clone())?;
            Ok(Some(literal))
        }
        Meta::NameValue(_) => Err(syn::Error::new_spanned(
            attribute,
            format!("`{name}` expects a string literal, like #[{name}(\"name\")]"),
        )),
    }
}

/// Check whether a specific `#[qjs(...)]` flag is already present.
fn has_qjs_flag(attributes: &[Attribute], flag: &str) -> bool {
    for attribute in attributes {
        if !attribute.path().is_ident("qjs") {
            continue;
        }

        let Meta::List(meta_list) = &attribute.meta else {
            continue;
        };

        let items: Punctuated<Meta, Token![,]> = Punctuated::<Meta, Token![,]>::parse_terminated
            .parse2(meta_list.tokens.clone())
            .unwrap_or_default();

        if items.iter().any(|meta| match meta {
            Meta::Path(path) => path.is_ident(flag),
            _ => false,
        }) {
            return true;
        }
    }

    false
}

/// Append a single `#[doc = "..."]` line.
fn append_doc_line(attributes: &mut Vec<Attribute>, doc_line: String) {
    let doc_attribute: Attribute = parse_quote! {
        #[doc = #doc_line]
    };
    attributes.push(doc_attribute);
}

/// Ensure the default `rename_all` argument is present.
fn normalize_methods_arguments(arguments: &TokenStream2) -> TokenStream2 {
    if arguments.is_empty() {
        return quote!(rename_all = #RENAME_ALL_CAMEL_CASE);
    }

    if contains_rename_all(arguments) {
        return arguments.clone();
    }

    quote!(#arguments, rename_all = #RENAME_ALL_CAMEL_CASE)
}

fn contains_rename_all(arguments: &TokenStream2) -> bool {
    let Ok(items) = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(arguments.clone())
    else {
        return false;
    };
    items.iter().any(|meta| match meta {
        Meta::NameValue(name_value) => name_value.path.is_ident("rename_all"),
        _ => false,
    })
}
