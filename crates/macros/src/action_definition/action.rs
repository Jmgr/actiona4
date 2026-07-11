use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Error, Expr, Fields, Ident, ItemStruct, Lit, MetaNameValue, Token, parse::Parser,
    parse_macro_input, parse_quote, punctuated::Punctuated,
};

use crate::action_definition::platforms::platform_constraints;

struct ActionParams {
    id: Option<String>,
    icon: Ident,
    effect: Ident,
    category: Ident,
    timeout: bool,
    waitable: bool,
    only: Option<Expr>,
    not: Option<Expr>,
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
    let effect = params.effect;
    let category = params.category;
    let supports_timeout = params.timeout;
    let is_waitable = params.waitable;
    let visibility = &item.vis;

    let action_platforms = match platform_constraints(params.only.as_ref(), params.not.as_ref()) {
        Ok(platforms) => platforms,
        Err(err) => return err.to_compile_error().into(),
    };

    let mut markers = Vec::new();
    let mut parameters = Vec::new();

    for field in &mut fields.named {
        match process_parameter_field(name, &id_kebab, visibility, field) {
            Ok(Some(processed)) => {
                markers.push(processed.markers);
                parameters.push(processed.parameter);
            }
            Ok(None) => {}
            Err(err) => return err.to_compile_error().into(),
        }
    }

    if supports_timeout {
        parameters.push(quote! { crate::actions::CommonParameters::TIMEOUT_PARAMETER });
    }

    let parameters = parameters;

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
                create_instance: || crate::actions::ActionInstance::#variant(
                    crate::actions::WithCommon::new(#name::default())
                ),
                effect: crate::actions::ActionEffect::#effect,
                category: crate::actions::ActionCategory::#category,
                supports_timeout: #supports_timeout,
                is_waitable: #is_waitable,
                platforms: ::types::platform::Platforms(&[#(#action_platforms),*]),
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

/// A `#[parameter]`-tagged field processed into its generated marker type and
/// `Parameter` definition. Shared between `#[action]` (for an action's own
/// fields) and `#[common_parameters]` (for the fields every action carries
/// via `CommonParameters`).
pub(crate) struct ProcessedField {
    pub(crate) markers: proc_macro2::TokenStream,
    pub(crate) parameter: proc_macro2::TokenStream,
    pub(crate) field_name: String,
}

/// Turns a `#[parameter]`-tagged field into its marker type, rewrites the
/// field's type to `Param<T, Marker>`, and builds its `Parameter` definition.
/// Returns `Ok(None)` for fields without `#[parameter]`.
pub(crate) fn process_parameter_field(
    owner_name: &Ident,
    id_kebab: &str,
    visibility: &syn::Visibility,
    field: &mut syn::Field,
) -> Result<Option<ProcessedField>, Error> {
    let is_parameter = field
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("parameter"));

    if !is_parameter {
        return Ok(None);
    }

    let Some(field_ident) = &field.ident else {
        return Err(Error::new_spanned(field, "parameter fields must be named"));
    };

    let field_name = field_ident.to_string();
    let marker = format_ident!("{}{}Param", owner_name, field_name.to_case(Case::Pascal));
    let ty = field.ty.clone();
    let kind = parameter_kind(&ty, &field.attrs)?;
    let parameter = parameter_definition(id_kebab, &field_name, &ty, &field.attrs)?;

    let markers = quote! {
        #visibility struct #marker;

        impl crate::parameters::ParamName for #marker {
            const NAME: &'static str = #field_name;
        }

        impl crate::parameters::ParamSpec for #marker {
            const KIND: crate::parameters::ParameterKind = #kind;
        }
    };

    field
        .attrs
        .retain(|attr| !attr.path().is_ident("parameter"));
    field.ty = parse_quote! {
        crate::parameters::Param<#ty, #marker>
    };

    Ok(Some(ProcessedField {
        markers,
        parameter,
        field_name,
    }))
}

fn parse_action_arguments(arguments: TokenStream) -> Result<ActionParams, Error> {
    let parser = Punctuated::<MetaNameValue, Token![,]>::parse_terminated;
    let arguments = parser.parse(arguments)?;

    let mut id = None;
    let mut icon = None;
    let mut effect = None;
    let mut category = None;
    let mut timeout = false;
    let mut waitable = false;
    let mut only = None;
    let mut not = None;

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
        } else if argument.path.is_ident("effect") {
            let Expr::Path(path) = argument.value else {
                return Err(Error::new_spanned(
                    argument,
                    "`effect` must be an action effect variant",
                ));
            };
            let Some(segment) = path.path.segments.last() else {
                return Err(Error::new_spanned(
                    path,
                    "`effect` must be an action effect variant",
                ));
            };
            effect = Some(segment.ident.clone());
        } else if argument.path.is_ident("category") {
            let Expr::Path(path) = argument.value else {
                return Err(Error::new_spanned(
                    argument,
                    "`category` must be an action category variant",
                ));
            };
            let Some(segment) = path.path.segments.last() else {
                return Err(Error::new_spanned(
                    path,
                    "`category` must be an action category variant",
                ));
            };
            category = Some(segment.ident.clone());
        } else if argument.path.is_ident("timeout") {
            let Expr::Lit(expr) = argument.value else {
                return Err(Error::new_spanned(argument, "`timeout` must be a boolean"));
            };
            let Lit::Bool(value) = expr.lit else {
                return Err(Error::new_spanned(expr, "`timeout` must be a boolean"));
            };
            timeout = value.value;
        } else if argument.path.is_ident("waitable") {
            let Expr::Lit(expr) = argument.value else {
                return Err(Error::new_spanned(argument, "`waitable` must be a boolean"));
            };
            let Lit::Bool(value) = expr.lit else {
                return Err(Error::new_spanned(expr, "`waitable` must be a boolean"));
            };
            waitable = value.value;
        } else if argument.path.is_ident("only") {
            only = Some(argument.value);
        } else if argument.path.is_ident("not") {
            not = Some(argument.value);
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

    let Some(effect) = effect else {
        return Err(Error::new_spanned(
            proc_macro2::TokenStream::new(),
            "missing required `effect` option",
        ));
    };

    let Some(category) = category else {
        return Err(Error::new_spanned(
            proc_macro2::TokenStream::new(),
            "missing required `category` option",
        ));
    };

    Ok(ActionParams {
        id,
        icon,
        effect,
        category,
        timeout,
        waitable,
        only,
        not,
    })
}

fn parameter_definition(
    id_kebab: &str,
    field_name: &str,
    ty: &syn::Type,
    attrs: &[syn::Attribute],
) -> Result<proc_macro2::TokenStream, Error> {
    let param_base = parameter_translation_base(id_kebab, field_name, attrs)?;
    let kind = parameter_kind(ty, attrs)?;
    let platforms = parameter_platforms(attrs)?;

    Ok(quote! {
        crate::parameters::Parameter {
            id: #field_name,
            name: crate::TranslationKey::with_attribute(#param_base, "name"),
            description: crate::TranslationKey::with_attribute(#param_base, "description"),
            kind: #kind,
            platforms: ::types::platform::Platforms(&[#(#platforms),*]),
        }
    })
}

fn parameter_platforms(attrs: &[syn::Attribute]) -> Result<Vec<proc_macro2::TokenStream>, Error> {
    let mut only = None;
    let mut not = None;

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
            let is_only = pair.path.is_ident("only");
            let is_not = pair.path.is_ident("not");
            if !is_only && !is_not {
                continue;
            }

            if is_only {
                if only.replace(pair.value).is_some() {
                    return Err(Error::new_spanned(pair.path, "duplicate `only` option"));
                }
            } else if not.replace(pair.value).is_some() {
                return Err(Error::new_spanned(pair.path, "duplicate `not` option"));
            }
        }
    }

    platform_constraints(only.as_ref(), not.as_ref())
}

fn parameter_translation_base(
    id_kebab: &str,
    field_name: &str,
    attrs: &[syn::Attribute],
) -> Result<String, Error> {
    let default_base = format!("action-{id_kebab}-{}", field_name.to_case(Case::Kebab));
    let mut translation = None;

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
            if !pair.path.is_ident("translation") {
                continue;
            }

            let Expr::Lit(expr) = pair.value else {
                return Err(Error::new_spanned(
                    pair,
                    "`translation` must be a string literal",
                ));
            };
            let Lit::Str(value) = expr.lit else {
                return Err(Error::new_spanned(
                    expr,
                    "`translation` must be a string literal",
                ));
            };

            if translation.replace(value.value()).is_some() {
                return Err(Error::new_spanned(value, "duplicate `translation` option"));
            }
        }
    }

    Ok(translation.unwrap_or(default_base))
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
            if pair.path.is_ident("translation")
                || pair.path.is_ident("only")
                || pair.path.is_ident("not")
            {
                continue;
            }

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
