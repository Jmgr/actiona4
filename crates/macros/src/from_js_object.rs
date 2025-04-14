use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericArgument, PathArguments, Type, parse_macro_input};

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
        let field_ty = &field.ty;
        let name_str = convert_case::Casing::to_case(
            &field_name.as_ref().unwrap().to_string(),
            convert_case::Case::Camel,
        );

        if let Some(inner_ty) = option_inner_type(field_ty) {
            return quote! {
                if object.contains_key(#name_str)? {
                    let field_value = object.get::<_, rquickjs::Value<'js>>(#name_str)?;
                    result.#field_name = if field_value.is_null() || field_value.is_undefined() {
                        None
                    } else {
                        Some(<#inner_ty as crate::api::js::FromJsField<'js>>::from_js_field(
                            ctx,
                            field_value,
                        )?)
                    };
                }
            };
        }

        let has_default = field.attrs.iter().any(|attr| attr.path().is_ident("default"));

        if has_default || is_scalar_type(field_ty) {
            quote! {
                if object.contains_key(#name_str)? {
                    let field_value = <#field_ty as crate::api::js::FromJsField<'js>>::from_js_field(
                        ctx,
                        object.get::<_, rquickjs::Value<'js>>(#name_str)?,
                    )?;
                    result.#field_name = field_value;
                }
            }
        } else {
            quote! {
                if object.contains_key(#name_str)? {
                    let field_value = <#field_ty as crate::api::js::FromJsField<'js>>::from_js_field(
                        ctx,
                        object.get::<_, rquickjs::Value<'js>>(#name_str)?,
                    )?;
                    result.#field_name = field_value;
                } else {
                    return Err(rquickjs::Exception::throw_message(
                        ctx,
                        concat!("missing required field '", #name_str, "'"),
                    ));
                }
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

/// Returns `true` for types that have a well-defined "absent" default and should
/// be treated as optional by `FromJsObject` even without an explicit `#[default]`.
///
/// Covered: `bool`, all integer primitives, `f32`/`f64`, and `Vec<T>`.
/// Everything else — `String`, custom structs, enums — is treated as required.
fn is_scalar_type(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };
    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };
    matches!(
        segment.ident.to_string().as_str(),
        "bool"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
            | "Vec"
    )
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }

    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };

    let GenericArgument::Type(inner_ty) = arguments.args.first()? else {
        return None;
    };

    Some(inner_ty)
}
