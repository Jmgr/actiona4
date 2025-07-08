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

    // Generate code to set each variant on a JS object
    let set_variants = variants.iter().map(|v| {
        let variant_name = &v.ident;
        quote! {
            object.set(stringify!(#variant_name).to_case(Case::Constant), #name::#variant_name)?;
        }
    });

    // Generate the full impl
    let expanded = quote! {
        impl #name {
            /// Registers this enum in the JS global scope with each variant as a property
            pub fn register(ctx: &rquickjs::Ctx) -> rquickjs::Result<()> {
                let object = rquickjs::Object::new(ctx.clone())?;
                #(#set_variants)*

                // Remove "Js" prefix if present
                let name = stringify!(#name).strip_prefix("Js").unwrap_or(&stringify!(#name));

                ctx.globals().set(name, object)?;
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
        let name_str = name.as_ref().unwrap().to_string().to_case(Case::Camel); // "createNew"
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
