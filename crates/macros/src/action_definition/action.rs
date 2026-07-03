use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Error, Ident, MetaNameValue, Token, parse_macro_input,
    punctuated::Punctuated,
};

#[derive(FromDeriveInput)]
#[darling(attributes(action))]
struct ActionParams {
    #[darling(default)]
    id: Option<String>,
    icon: Ident,
}

pub(crate) fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let Data::Struct(data) = &ast.data else {
        return Error::new_spanned(ast, "Action can only be used on structs")
            .to_compile_error()
            .into();
    };

    let params = match ActionParams::from_derive_input(&ast) {
        Ok(opts) => opts,
        Err(err) => return err.write_errors().into(),
    };

    if data.fields.iter().any(|field| field.ident.is_none()) {
        return Error::new_spanned(ast, "Action can only be used on structs with named fields")
            .to_compile_error()
            .into();
    }

    let name = &ast.ident;
    let name_str = name.to_string();
    let variant = quote::format_ident!("{name_str}");
    let id = params
        .id
        .unwrap_or_else(|| name_str.to_string().to_case(Case::Snake));
    // Translation keys use kebab-case for consistency with the enum keys; the
    // `id` itself stays snake_case to match the `ActionInstance` serde tag.
    let id_kebab = id.to_case(Case::Kebab);
    let action_base = format!("action-{id_kebab}");
    let icon = params.icon;

    let parameters = data
        .fields
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|attr| attr.path().is_ident("parameter"))
        })
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let ty = &field.ty;
            let param_base = format!("action-{id_kebab}-{}", field_name.to_case(Case::Kebab));

            // Per-field settings: `#[parameter(max_length = Some(100), ...)]`.
            // Each `key = value` becomes a field assignment on the storage's
            // default settings; the storage type decides which keys are valid.
            // The value is spliced verbatim, so it must be the full field value
            // (e.g. `Some(100)` for an `Option` setting). A bare `#[parameter]`
            // (no parentheses) is also accepted and contributes no assignments.
            let mut assignments = Vec::new();
            for attr in field
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("parameter"))
            {
                // `#[parameter]` parses as `Meta::Path`, which has no args to
                // parse; only `#[parameter(...)]` (a `Meta::List`) carries them.
                if matches!(attr.meta, syn::Meta::Path(_)) {
                    continue;
                }
                let pairs =
                    attr.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;
                for pair in pairs {
                    let key = pair.path;
                    let value = pair.value;
                    assignments.push(quote! { settings.#key = #value; });
                }
            }

            let kind = if assignments.is_empty() {
                quote! { <#ty as crate::parameters::ParameterStorage>::KIND }
            } else {
                quote! {{
                    let mut settings = <#ty as crate::parameters::ParameterStorage>::DEFAULT_SETTINGS;
                    #(#assignments)*
                    settings.into_kind()
                }}
            };

            Ok(quote! {
                crate::parameters::Parameter {
                    id: #field_name,
                    name: crate::TranslationKey::with_attribute(#param_base, "name"),
                    description: crate::TranslationKey::with_attribute(#param_base, "description"),
                    kind: #kind,
                }
            })
        })
        .collect::<Result<Vec<_>, Error>>();

    let parameters = match parameters {
        Ok(parameters) => parameters,
        Err(err) => return err.to_compile_error().into(),
    };

    let expanded = quote! {
        impl #name {
            pub const DEFINITION: crate::actions::ActionDefinition = crate::actions::ActionDefinition {
                id: #id,
                name: crate::TranslationKey::with_attribute(#action_base, "name"),
                description: crate::TranslationKey::with_attribute(#action_base, "description"),
                icon: ::icons::common::IconType::#icon,
                parameters: &[#(#parameters),*],
                create_instance: || crate::actions::ActionInstance::#variant(#name::default()),
            };
        }

        impl crate::actions::WithDefinition for #name {
            fn definition(&self) -> &'static crate::actions::ActionDefinition {
                &Self::DEFINITION
            }
        }
    };

    expanded.into()
}
