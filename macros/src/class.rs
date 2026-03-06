use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Error, Ident, Item, LitStr, Meta, Token, parse::Parser, parse_macro_input,
    punctuated::Punctuated,
};

use crate::consts::JS_TYPE_PREFIX;

/// Expand `#[js_class]` into `#[rquickjs::class(rename = "...")]`.
pub(crate) fn expand(arguments: TokenStream, item: TokenStream) -> TokenStream {
    let arguments = TokenStream2::from(arguments);
    let parsed_item = parse_macro_input!(item as Item);

    let class_ident = match &parsed_item {
        Item::Struct(item_struct) => &item_struct.ident,
        Item::Enum(item_enum) => &item_enum.ident,
        Item::Union(item_union) => &item_union.ident,
        _ => {
            return Error::new_spanned(
                &parsed_item,
                "`#[js_class]` can only be applied to a struct, enum, or union",
            )
            .to_compile_error()
            .into();
        }
    };

    let rename = match derive_class_name(class_ident) {
        Ok(rename) => rename,
        Err(error) => return error.to_compile_error().into(),
    };

    let final_arguments = normalize_class_arguments(&arguments, &rename);
    quote!(#[rquickjs::class(#final_arguments)] #parsed_item).into()
}

fn derive_class_name(class_ident: &Ident) -> syn::Result<LitStr> {
    let class_name = class_ident.to_string();
    let Some(stripped_name) = class_name.strip_prefix(JS_TYPE_PREFIX) else {
        return Err(Error::new_spanned(
            class_ident,
            "`#[js_class]` expects a Rust type named with a `Js` prefix (e.g. `JsMouse`)",
        ));
    };

    if stripped_name.is_empty() {
        return Err(Error::new_spanned(
            class_ident,
            "`#[js_class]` expects characters after the `Js` prefix",
        ));
    }

    Ok(LitStr::new(stripped_name, class_ident.span()))
}

fn normalize_class_arguments(arguments: &TokenStream2, rename: &LitStr) -> TokenStream2 {
    if arguments.is_empty() {
        return quote!(rename = #rename);
    }

    if contains_rename(arguments) {
        return arguments.clone();
    }

    quote!(#arguments, rename = #rename)
}

fn contains_rename(arguments: &TokenStream2) -> bool {
    let Ok(items) = Punctuated::<Meta, Token![,]>::parse_terminated.parse2(arguments.clone())
    else {
        return false;
    };
    items.iter().any(|meta| match meta {
        Meta::NameValue(name_value) => name_value.path.is_ident("rename"),
        _ => false,
    })
}
