use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

// TODO: use https://github.com/rquickjs/rquickjs-serde

#[proc_macro_derive(FromJsObject)]
pub fn derive_from_js_object(input: TokenStream) -> TokenStream {
    // Parse the user's struct
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    // We only handle `struct` with named fields
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => {
                return syn::Error::new_spanned(
                    &data_struct.fields,
                    "`FromJsObject` only supports structs with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input,
                "`FromJsObject` only supports structs with named fields",
            )
            .to_compile_error()
            .into();
        }
    };

    let build_fields = fields.iter().map(|f| {
        let name = &f.ident; // e.g. "create_new"
        let name_str = convert_case::Casing::to_case(
            &name.as_ref().unwrap().to_string(),
            convert_case::Case::Camel,
        ); // "createNew"
        quote! {
            if let Ok(#name) = object.get(#name_str) {
                result.#name = #name;
            }
        }
    });

    // Generate the final `impl FromJs` block
    let expanded = quote! {
        impl<'js> rquickjs::FromJs<'js> for #struct_name {
            fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
                let mut result = Self::default();

                use crate::core::ResultExt;
                let object = value
                    .as_object()
                    .or_throw_message(ctx, "Expected an object")?;

                #(#build_fields)*

                Ok(result)
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(IntoSerde)]
pub fn into_serde(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Generate the full impl
    let expanded = quote! {
        impl<'js> rquickjs::IntoJs<'js> for #name {
            fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
                rquickjs_serde::to_value(ctx.clone(), &self).map_err(|err| rquickjs::Exception::throw_message(ctx, &format!("{err}")))
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(FromSerde)]
pub fn from_serde(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Generate the full impl
    let expanded = quote! {
        impl<'js> rquickjs::FromJs<'js> for #name {
            fn from_js(ctx: &rquickjs::Ctx<'js>, v: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
                rquickjs_serde::from_value(v).map_err(|err| rquickjs::Exception::throw_message(ctx, &format!("{err}")))
            }
        }
    };

    expanded.into()
}
