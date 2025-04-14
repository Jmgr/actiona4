use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derive `rquickjs::IntoJs` by delegating through serde.
pub(crate) fn derive_into_serde(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl<'js> rquickjs::IntoJs<'js> for #name {
            fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
                rquickjs_serde::to_value(ctx.clone(), &self).map_err(|err| rquickjs::Exception::throw_message(ctx, &format!("{err}")))
            }
        }
    };

    expanded.into()
}

/// Derive `rquickjs::FromJs` by delegating through serde.
pub(crate) fn derive_from_serde(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl<'js> rquickjs::FromJs<'js> for #name {
            fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
                rquickjs_serde::from_value(value).map_err(|err| rquickjs::Exception::throw_message(ctx, &format!("{err}")))
            }
        }
    };

    expanded.into()
}
