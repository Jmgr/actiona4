use convert_case::{Case, Casing};
use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DeriveInput, Error, Expr, Fields, Ident, ItemStruct, Lit, MetaNameValue, Token,
    parse::Parser, parse_macro_input, parse_quote, punctuated::Punctuated,
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
            field
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("parameter"))
        })
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap().to_string();
            let ty = &field.ty;
            let param_base = format!("action-{id_kebab}-{}", field_name.to_case(Case::Kebab));
            let kind = parameter_kind(ty, &field.attrs)?;

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

pub(crate) fn expand(arguments: TokenStream, item: TokenStream) -> TokenStream {
    let params = match parse_action_arguments(arguments) {
        Ok(params) => params,
        Err(err) => return err.to_compile_error().into(),
    };
    let mut item = parse_macro_input!(item as ItemStruct);

    let Fields::Named(fields) = &mut item.fields else {
        return Error::new_spanned(item, "action can only be used on structs with named fields")
            .to_compile_error()
            .into();
    };

    let name = &item.ident;
    let name_str = name.to_string();
    let variant = quote::format_ident!("{name_str}");
    let id = params
        .id
        .unwrap_or_else(|| name_str.to_string().to_case(Case::Snake));
    let id_kebab = id.to_case(Case::Kebab);
    let action_base = format!("action-{id_kebab}");
    let icon = params.icon;
    let visibility = &item.vis;

    let mut markers = Vec::new();
    let mut parameters = Vec::new();

    for field in &mut fields.named {
        let is_parameter = field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("parameter"));

        if !is_parameter {
            continue;
        }

        let Some(field_ident) = &field.ident else {
            return Error::new_spanned(field, "parameter fields must be named")
                .to_compile_error()
                .into();
        };

        let field_name = field_ident.to_string();
        let marker = format_ident!("{}{}Param", name, field_name.to_case(Case::Pascal));
        let ty = field.ty.clone();
        let kind = match parameter_kind(&ty, &field.attrs) {
            Ok(kind) => kind,
            Err(err) => return err.to_compile_error().into(),
        };
        let parameter = match parameter_definition(&id_kebab, &field_name, &ty, &field.attrs) {
            Ok(parameter) => parameter,
            Err(err) => return err.to_compile_error().into(),
        };

        markers.push(quote! {
            #visibility struct #marker;

            impl crate::parameters::ParamName for #marker {
                const NAME: &'static str = #field_name;
            }

            impl crate::parameters::ParamSpec for #marker {
                const KIND: crate::parameters::ParameterKind = #kind;
            }
        });
        parameters.push(parameter);

        field
            .attrs
            .retain(|attr| !attr.path().is_ident("parameter"));
        field.ty = parse_quote! {
            crate::parameters::Param<#ty, #marker>
        };
    }

    let expanded = quote! {
        #(#markers)*

        #item

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

fn parse_action_arguments(arguments: TokenStream) -> Result<ActionParams, Error> {
    let parser = Punctuated::<MetaNameValue, Token![,]>::parse_terminated;
    let arguments = parser.parse(arguments)?;

    let mut id = None;
    let mut icon = None;

    for argument in arguments {
        if argument.path.is_ident("id") {
            let Expr::Lit(expr) = argument.value else {
                return Err(Error::new_spanned(
                    argument,
                    "`id` must be a string literal",
                ));
            };
            let Lit::Str(value) = expr.lit else {
                return Err(Error::new_spanned(expr, "`id` must be a string literal"));
            };
            id = Some(value.value());
        } else if argument.path.is_ident("icon") {
            let Expr::Path(path) = argument.value else {
                return Err(Error::new_spanned(
                    argument,
                    "`icon` must be an icon variant",
                ));
            };
            let Some(segment) = path.path.segments.last() else {
                return Err(Error::new_spanned(path, "`icon` must be an icon variant"));
            };
            icon = Some(segment.ident.clone());
        } else {
            return Err(Error::new_spanned(argument.path, "unknown `action` option"));
        }
    }

    let Some(icon) = icon else {
        return Err(Error::new_spanned(
            proc_macro2::TokenStream::new(),
            "missing required `icon` option",
        ));
    };

    Ok(ActionParams { id, icon })
}

fn parameter_definition(
    id_kebab: &str,
    field_name: &str,
    ty: &syn::Type,
    attrs: &[syn::Attribute],
) -> Result<proc_macro2::TokenStream, Error> {
    let param_base = format!("action-{id_kebab}-{}", field_name.to_case(Case::Kebab));
    let kind = parameter_kind(ty, attrs)?;

    Ok(quote! {
        crate::parameters::Parameter {
            id: #field_name,
            name: crate::TranslationKey::with_attribute(#param_base, "name"),
            description: crate::TranslationKey::with_attribute(#param_base, "description"),
            kind: #kind,
        }
    })
}

fn parameter_kind(
    ty: &syn::Type,
    attrs: &[syn::Attribute],
) -> Result<proc_macro2::TokenStream, Error> {
    let mut assignments = Vec::new();
    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("parameter"))
    {
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

    Ok(kind)
}
