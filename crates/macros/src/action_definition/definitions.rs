use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Error, Fields, GenericArgument, PathArguments, Type, parse_macro_input,
};

/// Each variant holds `WithCommon<Action>`; the `DEFINITION` const lives on the
/// inner `Action`, so unwrap the single generic argument of `WithCommon<_>`.
/// Any other type is returned unchanged.
fn inner_action_type(ty: &Type) -> &Type {
    let Type::Path(type_path) = ty else {
        return ty;
    };
    let Some(segment) = type_path.path.segments.last() else {
        return ty;
    };
    if segment.ident != "WithCommon" {
        return ty;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return ty;
    };
    match args.args.first() {
        Some(GenericArgument::Type(inner)) => inner,
        _ => ty,
    }
}

pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let Data::Enum(data) = &ast.data else {
        return Error::new_spanned(ast, "ActionDefinitions can only be used on enums")
            .to_compile_error()
            .into();
    };

    let variants = data
        .variants
        .iter()
        .map(|variant| {
            let Fields::Unnamed(fields) = &variant.fields else {
                return Err(Error::new_spanned(
                    variant,
                    "ActionDefinitions requires newtype variants like `Click(Click)`",
                ));
            };
            let mut fields = fields.unnamed.iter();
            let (Some(field), None) = (fields.next(), fields.next()) else {
                return Err(Error::new_spanned(
                    variant,
                    "ActionDefinitions requires each variant to hold exactly one action type",
                ));
            };
            Ok((&variant.ident, &field.ty, inner_action_type(&field.ty)))
        })
        .collect::<Result<Vec<_>, Error>>();

    let variants = match variants {
        Ok(variants) => variants,
        Err(err) => return err.to_compile_error().into(),
    };

    let definitions = variants
        .iter()
        .map(|(_, _, action)| quote! { <#action>::DEFINITION })
        .collect::<Vec<_>>();
    let enum_name = &ast.ident;
    let human_readable_ref = format_ident!("__{enum_name}HumanReadableRef");
    let human_readable = format_ident!("__{enum_name}HumanReadable");
    let binary_ref = format_ident!("__{enum_name}BinaryRef");
    let binary = format_ident!("__{enum_name}Binary");
    let human_readable_ref_variants = variants
        .iter()
        .map(|(name, ty, _)| quote! { #name(&'a #ty) })
        .collect::<Vec<_>>();
    let human_readable_variants = variants
        .iter()
        .map(|(name, ty, _)| quote! { #name(#ty) })
        .collect::<Vec<_>>();
    let binary_ref_variants = variants
        .iter()
        .map(|(name, ty, _)| quote! { #name(&'a #ty) })
        .collect::<Vec<_>>();
    let binary_variants = variants
        .iter()
        .map(|(name, ty, _)| quote! { #name(#ty) })
        .collect::<Vec<_>>();
    let human_readable_serialize_arms = variants
        .iter()
        .map(|(name, _, _)| quote! { Self::#name(value) => #human_readable_ref::#name(value) })
        .collect::<Vec<_>>();
    let binary_serialize_arms = variants
        .iter()
        .map(|(name, _, _)| quote! { Self::#name(value) => #binary_ref::#name(value) })
        .collect::<Vec<_>>();
    let human_readable_deserialize_arms = variants
        .iter()
        .map(|(name, _, _)| quote! { #human_readable::#name(value) => Self::#name(value) })
        .collect::<Vec<_>>();
    let binary_deserialize_arms = variants
        .iter()
        .map(|(name, _, _)| quote! { #binary::#name(value) => Self::#name(value) })
        .collect::<Vec<_>>();

    quote! {
        pub const ACTION_DEFINITIONS: &[crate::actions::ActionDefinition] =
            &[#(#definitions),*];

        #[derive(serde::Serialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        enum #human_readable_ref<'a> {
            #(#human_readable_ref_variants),*
        }

        #[derive(serde::Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case")]
        enum #human_readable {
            #(#human_readable_variants),*
        }

        #[derive(serde::Serialize)]
        enum #binary_ref<'a> {
            #(#binary_ref_variants),*
        }

        #[derive(serde::Deserialize)]
        enum #binary {
            #(#binary_variants),*
        }

        impl serde::Serialize for #enum_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                if serializer.is_human_readable() {
                    let value = match self {
                        #(#human_readable_serialize_arms),*
                    };
                    serde::Serialize::serialize(&value, serializer)
                } else {
                    let value = match self {
                        #(#binary_serialize_arms),*
                    };
                    serde::Serialize::serialize(&value, serializer)
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for #enum_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    let value = <#human_readable as serde::Deserialize>::deserialize(deserializer)?;
                    Ok(match value {
                        #(#human_readable_deserialize_arms),*
                    })
                } else {
                    let value = <#binary as serde::Deserialize>::deserialize(deserializer)?;
                    Ok(match value {
                        #(#binary_deserialize_arms),*
                    })
                }
            }
        }
    }
    .into()
}
