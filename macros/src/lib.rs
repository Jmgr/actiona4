use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

#[proc_macro_derive(ExposeEnum)]
pub fn expose_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // We only handle `enum` here:
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return syn::Error::new_spanned(&input.ident, "ExposeEnum can only be used on enums")
                .to_compile_error()
                .into();
        }
    };

    // Enforce unit variants only:
    for variant in variants {
        if !matches!(variant.fields, syn::Fields::Unit) {
            return syn::Error::new_spanned(
                &variant.ident,
                "ExposeEnum only supports unit variants",
            )
            .to_compile_error()
            .into();
        }
    }

    // Precompute the JS names at macro time
    let set_variants = variants.iter().map(|v| {
        let variant_ident = &v.ident;
        let js_key = v.ident.to_string().to_case(Case::Constant); // e.g. "MY_VARIANT"
        quote! {
            object.set(#js_key, #name::#variant_ident)?;
        }
    });

    // Also precompute the exported object name without the "Js" prefix
    let type_name = name.to_string();
    let export_name = type_name
        .strip_prefix("Js")
        .unwrap_or(&type_name)
        .to_string();

    // Generate the full impl
    let expanded = quote! {
        impl #name {
            /// Registers this enum in the JS global scope with each variant as a property
            pub fn register(ctx: &rquickjs::Ctx) -> rquickjs::Result<()> {
                let object = rquickjs::Object::new(ctx.clone())?;
                #(#set_variants)*

                ctx.globals().set(#export_name, object)?;
                Ok(())
            }
        }
    };

    expanded.into()
}

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
