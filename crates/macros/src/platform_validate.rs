use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DataEnum, DeriveInput, Field, Fields, GenericArgument, LitStr, PathArguments,
    Type, Variant, parse_macro_input, spanned::Spanned,
};

use crate::{
    consts::{
        JS_TYPE_PREFIX, PLATFORM_DISPLAY_LINUX, PLATFORM_DISPLAY_WAYLAND, PLATFORM_DISPLAY_WINDOWS,
        PLATFORM_DISPLAY_X11, PLATFORM_LINUX, PLATFORM_WAYLAND, PLATFORM_WINDOWS, PLATFORM_X11,
        RAW_IDENT_PREFIX,
    },
    default_args::{PlatformArguments, parse_attribute_arguments},
};

/// Derive `validate_for_platform(...)` for structs and enums.
pub(crate) fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = &input.ident;

    let expanded = match &input.data {
        Data::Struct(data_struct) => {
            derive_platform_validate_for_struct(type_name, &data_struct.fields)
        }
        Data::Enum(data_enum) => derive_platform_validate_for_enum(type_name, data_enum),
        _ => Err(syn::Error::new_spanned(
            &input,
            "`PlatformValidate` only supports structs with named fields or enums",
        )),
    };

    match expanded {
        Ok(expanded) => expanded.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Generate validation for a struct with platform-limited fields.
fn derive_platform_validate_for_struct(
    struct_name: &syn::Ident,
    fields: &Fields,
) -> syn::Result<proc_macro2::TokenStream> {
    let fields = match fields {
        Fields::Named(fields_named) => &fields_named.named,
        _ => {
            return Err(syn::Error::new_spanned(
                fields,
                "`PlatformValidate` on structs only supports named fields",
            ));
        }
    };

    let mut field_checks = Vec::new();
    let mut nested_checks = Vec::new();

    for field in fields {
        let Some(field_name) = &field.ident else {
            continue;
        };

        let platform_field_config = parse_platform_config(field)?;

        if platform_field_config.nested {
            let nested_check = build_nested_validation_check(field_name, &field.ty)?;
            nested_checks.push(nested_check);
        }

        let field_label_literal = platform_field_config
            .label
            .clone()
            .unwrap_or_else(|| default_platform_label(field_name));

        let check_kind = platform_field_config
            .check_kind
            .unwrap_or(PlatformCheckKind::Auto);

        if let Some(allowed_platform) = platform_field_config.allowed_platform {
            let presence_check = build_presence_check(field_name, &field.ty, check_kind)?;

            let supported_platform = allowed_platform.display_name();
            let error_message = LitStr::new(
                &format!(
                    "{} is only available on {supported_platform}",
                    field_label_literal.value()
                ),
                field_label_literal.span(),
            );

            let guard_condition = allowed_platform.guard_condition();
            field_checks.push(quote! {
                if #guard_condition && #presence_check {
                    return Err(crate::error::CommonError::UnsupportedPlatform(#error_message.into()).into());
                }
            });
        }

        if let Some(forbidden_platform) = platform_field_config.forbidden_platform {
            let presence_check = build_presence_check(field_name, &field.ty, check_kind)?;

            let forbidden_platform_name = forbidden_platform.display_name();
            let error_message = LitStr::new(
                &format!(
                    "{} is not available on {forbidden_platform_name}",
                    field_label_literal.value()
                ),
                field_label_literal.span(),
            );

            let not_guard_condition = forbidden_platform.not_guard_condition();
            field_checks.push(quote! {
                if #not_guard_condition && #presence_check {
                    return Err(crate::error::CommonError::UnsupportedPlatform(#error_message.into()).into());
                }
            });
        }
    }

    Ok(quote! {
        impl #struct_name {
            fn validate_for_platform(&self, platform: crate::platform_info::Platform) -> color_eyre::Result<()> {
                #(#field_checks)*

                #(#nested_checks)*

                Ok(())
            }
        }
    })
}

/// Generate validation for an enum with platform-limited variants.
fn derive_platform_validate_for_enum(
    enum_name: &syn::Ident,
    data_enum: &DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut variant_checks = Vec::new();

    for variant in &data_enum.variants {
        let platform_field_config = parse_platform_config_from_attributes(&variant.attrs)?;

        if platform_field_config.check_kind.is_some() {
            return Err(syn::Error::new_spanned(
                variant,
                "`platform(check = ...)` is only supported on struct fields",
            ));
        }

        if platform_field_config.nested {
            return Err(syn::Error::new_spanned(
                variant,
                "`platform(nested)` is only supported on struct fields",
            ));
        }

        let variant_name = &variant.ident;
        let variant_pattern = variant_pattern(variant);
        let variant_label_literal = platform_field_config
            .label
            .unwrap_or_else(|| default_enum_variant_label(enum_name, variant_name));

        if let Some(allowed_platform) = platform_field_config.allowed_platform {
            let supported_platform = allowed_platform.display_name();
            let error_message = LitStr::new(
                &format!(
                    "{} is only available on {supported_platform}",
                    variant_label_literal.value()
                ),
                variant_label_literal.span(),
            );

            let guard_condition = allowed_platform.guard_condition();
            variant_checks.push(quote! {
                if #guard_condition && matches!(self, #variant_pattern) {
                    return Err(crate::error::CommonError::UnsupportedPlatform(#error_message.into()).into());
                }
            });
        }

        if let Some(forbidden_platform) = platform_field_config.forbidden_platform {
            let forbidden_platform_name = forbidden_platform.display_name();
            let error_message = LitStr::new(
                &format!(
                    "{} is not available on {forbidden_platform_name}",
                    variant_label_literal.value()
                ),
                variant_label_literal.span(),
            );

            let not_guard_condition = forbidden_platform.not_guard_condition();
            variant_checks.push(quote! {
                if #not_guard_condition && matches!(self, #variant_pattern) {
                    return Err(crate::error::CommonError::UnsupportedPlatform(#error_message.into()).into());
                }
            });
        }
    }

    Ok(quote! {
        impl #enum_name {
            fn validate_for_platform(&self, platform: crate::platform_info::Platform) -> color_eyre::Result<()> {
                #(#variant_checks)*

                Ok(())
            }
        }
    })
}

/// Build a match pattern for a given variant.
fn variant_pattern(variant: &Variant) -> proc_macro2::TokenStream {
    let variant_name = &variant.ident;
    match &variant.fields {
        Fields::Unit => quote! { Self::#variant_name },
        Fields::Unnamed(_) => quote! { Self::#variant_name(..) },
        Fields::Named(_) => quote! { Self::#variant_name { .. } },
    }
}

#[derive(Clone, Copy)]
enum SupportedPlatform {
    Linux,
    Windows,
    Wayland,
    X11,
}

impl SupportedPlatform {
    fn parse(platform_name: &str, span: proc_macro2::Span) -> syn::Result<Self> {
        match platform_name.to_ascii_lowercase().as_str() {
            PLATFORM_LINUX => Ok(Self::Linux),
            PLATFORM_WINDOWS => Ok(Self::Windows),
            PLATFORM_WAYLAND => Ok(Self::Wayland),
            PLATFORM_X11 => Ok(Self::X11),
            _ => Err(syn::Error::new(
                span,
                "unknown platform, expected `linux`, `windows`, `wayland`, or `x11`",
            )),
        }
    }

    const fn display_name(self) -> &'static str {
        match self {
            Self::Linux => PLATFORM_DISPLAY_LINUX,
            Self::Windows => PLATFORM_DISPLAY_WINDOWS,
            Self::Wayland => PLATFORM_DISPLAY_WAYLAND,
            Self::X11 => PLATFORM_DISPLAY_X11,
        }
    }

    /// The condition that must be true for an error to be emitted at runtime.
    ///
    /// For a field only available on platform P, the check fires when the
    /// current platform is *not* P.
    fn guard_condition(self) -> proc_macro2::TokenStream {
        match self {
            // Windows-only → error on Linux
            Self::Windows => quote! { crate::platform_info::is_linux() },
            // Linux-only → error on Windows
            Self::Linux => quote! { crate::platform_info::is_windows() },
            // Wayland-only → error when not Wayland (includes XWayland)
            Self::Wayland => quote! { !(platform.is_wayland() || platform.is_x_wayland()) },
            // X11-only → error when not exactly X11
            Self::X11 => {
                quote! { !matches!(platform, crate::platform_info::Platform::X11) }
            }
        }
    }

    /// The condition that must be true for an error to be emitted at runtime.
    ///
    /// For a field *not* available on platform P, the check fires when the
    /// current platform *is* P.
    fn not_guard_condition(self) -> proc_macro2::TokenStream {
        match self {
            Self::Windows => quote! { crate::platform_info::is_windows() },
            Self::Linux => quote! { crate::platform_info::is_linux() },
            Self::Wayland => quote! { platform.is_wayland() || platform.is_x_wayland() },
            Self::X11 => {
                quote! { matches!(platform, crate::platform_info::Platform::X11) }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum PlatformCheckKind {
    Auto,
    IsSome,
    IsTrue,
    NonEmpty,
}

impl PlatformCheckKind {
    fn parse(check_name: &str, span: proc_macro2::Span) -> syn::Result<Self> {
        match check_name.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "is_some" => Ok(Self::IsSome),
            "is_true" => Ok(Self::IsTrue),
            "non_empty" => Ok(Self::NonEmpty),
            _ => Err(syn::Error::new(
                span,
                "unknown check, expected `auto`, `is_some`, `is_true`, or `non_empty`",
            )),
        }
    }
}

#[derive(Default)]
struct PlatformFieldConfig {
    allowed_platform: Option<SupportedPlatform>,
    forbidden_platform: Option<SupportedPlatform>,
    check_kind: Option<PlatformCheckKind>,
    label: Option<LitStr>,
    nested: bool,
}

/// Parse platform config from a field and its attributes.
fn parse_platform_config(field: &Field) -> syn::Result<PlatformFieldConfig> {
    parse_platform_config_from_attributes(&field.attrs)
}

/// Parse `#[platform(...)]` arguments into a field config.
fn parse_platform_config_from_attributes(
    attributes: &[Attribute],
) -> syn::Result<PlatformFieldConfig> {
    let mut config = PlatformFieldConfig::default();

    for attribute in attributes {
        if !attribute.path().is_ident("platform") {
            continue;
        }

        let parsed_arguments: PlatformArguments = parse_attribute_arguments(attribute, "platform")?;

        if let Some(not_platform) = parsed_arguments.not_platform {
            if config.forbidden_platform.is_some() {
                return Err(syn::Error::new_spanned(attribute, "duplicate `not` value"));
            }

            config.forbidden_platform =
                Some(SupportedPlatform::parse(&not_platform, attribute.span())?);
        }

        if let Some(only_platform) = parsed_arguments.only {
            if config.allowed_platform.is_some() {
                return Err(syn::Error::new_spanned(attribute, "duplicate `only` value"));
            }

            config.allowed_platform =
                Some(SupportedPlatform::parse(&only_platform, attribute.span())?);
        }

        if let Some(check_name) = parsed_arguments.check {
            if config.check_kind.is_some() {
                return Err(syn::Error::new_spanned(
                    attribute,
                    "duplicate `check` value",
                ));
            }

            config.check_kind = Some(PlatformCheckKind::parse(&check_name, attribute.span())?);
        }

        if let Some(label_text) = parsed_arguments.label {
            if config.label.is_some() {
                return Err(syn::Error::new_spanned(
                    attribute,
                    "duplicate `label` value",
                ));
            }

            config.label = Some(LitStr::new(&label_text, attribute.span()));
        }

        if parsed_arguments.nested {
            config.nested = true;
        }
    }

    Ok(config)
}

/// Default label used in error messages for a struct field.
fn default_platform_label(field_name: &syn::Ident) -> LitStr {
    let field_name_string = field_name.to_string();
    let normalized_field_name = field_name_string
        .strip_prefix(RAW_IDENT_PREFIX)
        .unwrap_or(&field_name_string);
    let field_name_in_camel_case = convert_case::Casing::to_case(
        &normalized_field_name.to_string(),
        convert_case::Case::Camel,
    );

    LitStr::new(&field_name_in_camel_case, field_name.span())
}

/// Default label used in error messages for an enum variant.
fn default_enum_variant_label(enum_name: &syn::Ident, variant_name: &syn::Ident) -> LitStr {
    let enum_name_string = enum_name.to_string();
    let normalized_enum_name = enum_name_string
        .strip_prefix(JS_TYPE_PREFIX)
        .unwrap_or(&enum_name_string);

    let variant_name_string = variant_name.to_string();
    let normalized_variant_name = variant_name_string
        .strip_prefix(RAW_IDENT_PREFIX)
        .unwrap_or(&variant_name_string);

    let full_variant_name = format!("{normalized_enum_name}.{normalized_variant_name}");
    LitStr::new(&full_variant_name, variant_name.span())
}

/// Build the runtime presence check for a field.
fn build_presence_check(
    field_name: &syn::Ident,
    field_type: &Type,
    check_kind: PlatformCheckKind,
) -> syn::Result<proc_macro2::TokenStream> {
    match check_kind {
        PlatformCheckKind::Auto => {
            if type_is_bool(field_type) {
                return Ok(quote! { self.#field_name });
            }

            if option_inner_type(field_type).is_some() {
                return Ok(quote! { self.#field_name.is_some() });
            }

            if vec_inner_type(field_type).is_some() {
                return Ok(quote! { !self.#field_name.is_empty() });
            }

            Err(syn::Error::new_spanned(
                field_type,
                "cannot infer `platform` check automatically; set `check = \"...\"`",
            ))
        }
        PlatformCheckKind::IsSome => Ok(quote! { self.#field_name.is_some() }),
        PlatformCheckKind::IsTrue => Ok(quote! { self.#field_name }),
        PlatformCheckKind::NonEmpty => {
            if option_inner_type(field_type).is_some() {
                Ok(quote! {
                    self.#field_name
                        .as_ref()
                        .is_some_and(|value| !value.is_empty())
                })
            } else {
                Ok(quote! { !self.#field_name.is_empty() })
            }
        }
    }
}

/// Build nested validation calls for fields containing option structs.
fn build_nested_validation_check(
    field_name: &syn::Ident,
    field_type: &Type,
) -> syn::Result<proc_macro2::TokenStream> {
    if let Some(option_inner) = option_inner_type(field_type) {
        if vec_inner_type(option_inner).is_some() {
            return Ok(quote! {
                if let Some(values) = &self.#field_name {
                    for nested_value in values {
                        nested_value.validate_for_platform(platform)?;
                    }
                }
            });
        }

        return Ok(quote! {
            if let Some(value) = &self.#field_name {
                value.validate_for_platform(platform)?;
            }
        });
    }

    if vec_inner_type(field_type).is_some() {
        return Ok(quote! {
            for value in &self.#field_name {
                value.validate_for_platform(platform)?;
            }
        });
    }

    if type_is_bool(field_type) {
        return Err(syn::Error::new_spanned(
            field_type,
            "`platform(nested)` is not supported on `bool` fields",
        ));
    }

    Ok(quote! {
        self.#field_name.validate_for_platform(platform)?;
    })
}

/// True if the type is a `bool`.
fn type_is_bool(field_type: &Type) -> bool {
    let Type::Path(type_path) = field_type else {
        return false;
    };

    type_path
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "bool")
}

/// Extract the inner type from `Option<T>`.
fn option_inner_type(field_type: &Type) -> Option<&Type> {
    extract_single_type_argument(field_type, "Option")
}

/// Extract the inner type from `Vec<T>`.
fn vec_inner_type(field_type: &Type) -> Option<&Type> {
    extract_single_type_argument(field_type, "Vec")
}

/// Extract a single type argument from `TypePath` generics.
fn extract_single_type_argument<'field_type>(
    field_type: &'field_type Type,
    wrapper_name: &str,
) -> Option<&'field_type Type> {
    let Type::Path(type_path) = field_type else {
        return None;
    };
    let path_segment = type_path.path.segments.last()?;
    if path_segment.ident != wrapper_name {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &path_segment.arguments else {
        return None;
    };
    let generic_argument = arguments.args.first()?;
    let GenericArgument::Type(inner_type) = generic_argument else {
        return None;
    };
    Some(inner_type)
}
