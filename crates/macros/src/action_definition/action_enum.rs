use convert_case::{Case, Casing};
use darling::{FromDeriveInput, FromMeta, FromVariant};
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Expr, parse_macro_input};

use crate::action_definition::platforms::platform_constraints;

#[derive(FromDeriveInput)]
#[darling(attributes(serde), allow_unknown_fields)]
struct SerdeOpts {
    rename_all: Option<String>,
}

#[derive(FromVariant)]
#[darling(attributes(serde), supports(unit))]
struct SerdeVariantOpts {
    ident: syn::Ident,
    #[darling(default)]
    rename: Option<SerdeRename>,
}

#[derive(FromVariant)]
#[darling(attributes(action_enum), supports(unit))]
struct ActionEnumVariantOpts {
    #[darling(default)]
    only: Option<Expr>,
    #[darling(default)]
    not: Option<Expr>,
}

enum SerdeRename {
    Both(String),
    Split(SplitSerdeRename),
}

#[derive(FromMeta)]
struct SplitSerdeRename {
    #[darling(default)]
    serialize: Option<String>,
    #[darling(default, rename = "deserialize")]
    _deserialize: Option<String>,
}

impl FromMeta for SerdeRename {
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self::Both(value.to_owned()))
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        SplitSerdeRename::from_list(items).map(Self::Split)
    }
}

impl SerdeRename {
    fn serialize_name(self) -> Option<String> {
        match self {
            Self::Both(name) => Some(name),
            Self::Split(rename) => rename.serialize,
        }
    }
}

pub(crate) fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let Data::Enum(data) = &ast.data else {
        return Error::new_spanned(ast, "ActionEnum can only be used on enums")
            .to_compile_error()
            .into();
    };

    let serde = match SerdeOpts::from_derive_input(&ast) {
        Ok(opts) => opts,
        Err(err) => return err.write_errors().into(),
    };

    let Some(rename_all) = serde.rename_all else {
        return Error::new_spanned(&ast, "ActionEnum requires #[serde(rename_all = \"...\")]")
            .to_compile_error()
            .into();
    };

    if rename_all != "kebab-case" {
        return Error::new_spanned(
            &ast,
            "ActionEnum requires #[serde(rename_all = \"kebab-case\")]",
        )
        .to_compile_error()
        .into();
    }

    let name = &ast.ident;
    let name_key = format!("enum-{}", name.to_string().to_case(Case::Kebab));

    let variants = data
        .variants
        .iter()
        .map(|variant| {
            let opts = SerdeVariantOpts::from_variant(variant)?;
            let platform_opts = ActionEnumVariantOpts::from_variant(variant)?;
            let variant_name = opts
                .rename
                .and_then(SerdeRename::serialize_name)
                .unwrap_or_else(|| opts.ident.to_string().to_case(Case::Kebab));
            let platforms =
                platform_constraints(platform_opts.only.as_ref(), platform_opts.not.as_ref())?;
            Ok(quote! {
                crate::parameters::enumeration::EnumParameterVariant {
                    id: #variant_name,
                    name: crate::TranslationKey::with_attribute(#name_key, #variant_name),
                    platforms: ::types::platform::Platforms(&[#(#platforms),*]),
                }
            })
        })
        .collect::<Result<Vec<_>, Error>>();

    let variants = match variants {
        Ok(variants) => variants,
        Err(err) => return err.to_compile_error().into(),
    };

    let expanded = quote! {
        impl crate::parameters::ParameterStorage for #name {
            type Settings = crate::parameters::enumeration::EnumParameter;
            const DEFAULT_SETTINGS: Self::Settings = crate::parameters::enumeration::EnumParameter {
                variants: &[#(#variants),*],
            };
            const KIND: crate::parameters::ParameterKind = Self::DEFAULT_SETTINGS.into_kind();
        }
    };

    expanded.into()
}
