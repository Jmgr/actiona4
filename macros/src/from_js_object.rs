use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derive `rquickjs::FromJs` for named-field option structs.
pub(crate) fn derive(input: TokenStream) -> TokenStream {
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

    let build_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let name_str = convert_case::Casing::to_case(
            &field_name.as_ref().unwrap().to_string(),
            convert_case::Case::Camel,
        );
        quote! {
            if let Ok(field_value) = object.get(#name_str) {
                result.#field_name = field_value;
            }
        }
    });

    // Generate the final `impl FromJs` block
    let expanded = quote! {
        impl<'js> rquickjs::FromJs<'js> for #struct_name {
            fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
                let mut result = Self::default();

                use crate::api::ResultExt;
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
