use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, FnArg, Ident, Item, ItemFn, Pat, ReturnType, Type, parse_quote};

use crate::{
    consts::{
        INSTR_PLATFORMS, PLATFORM_DISPLAY_LINUX, PLATFORM_DISPLAY_WAYLAND,
        PLATFORM_DISPLAY_WINDOWS, PLATFORM_DISPLAY_X11, PLATFORM_LINUX, PLATFORM_WAYLAND,
        PLATFORM_WINDOWS, PLATFORM_X11,
    },
    default_args::{
        PlatformArguments as RawPlatformArguments, doc_contains, parse_meta_list_tokens,
    },
};

/// Expand `#[platform]` to a runtime guard and rustdoc instruction.
pub(crate) fn expand(arguments: TokenStream, item: TokenStream) -> TokenStream {
    let raw_arguments = match parse_meta_list_tokens::<RawPlatformArguments>(
        proc_macro2::TokenStream::from(arguments),
        Span::call_site(),
    ) {
        Ok(raw_arguments) => raw_arguments,
        Err(error) => return error.to_compile_error().into(),
    };
    let arguments = match PlatformAttributeArguments::from_raw(raw_arguments) {
        Ok(arguments) => arguments,
        Err(error) => return error.to_compile_error().into(),
    };
    let item_tokens = proc_macro2::TokenStream::from(item);

    if let Ok(function) = syn::parse2::<ItemFn>(item_tokens.clone()) {
        return expand_function(arguments, function);
    }

    let parsed_item = match syn::parse2::<Item>(item_tokens) {
        Ok(parsed_item) => parsed_item,
        Err(error) => return error.to_compile_error().into(),
    };
    expand_non_function_item(arguments, parsed_item)
}

/// Inject a platform guard into a function item.
fn expand_function(arguments: PlatformAttributeArguments, mut function: ItemFn) -> TokenStream {
    if !returns_result(&function.sig.output) {
        return syn::Error::new_spanned(
            &function.sig.output,
            "`platform` only supports functions returning `Result<...>`",
        )
        .to_compile_error()
        .into();
    }

    let ctx_parameter = match find_ctx_parameter(&function) {
        Ok(parameter) => parameter,
        Err(error) => return error.to_compile_error().into(),
    };
    let platform_expression = if let Some(ctx_parameter) = &ctx_parameter {
        quote! { crate::runtime::WithUserData::user_data(&#ctx_parameter).platform() }
    } else {
        quote! { crate::platform_info::Platform::detect() }
    };

    let platform_check = arguments.platform_check();
    let error_message = arguments.error_message();
    let unsupported_platform_error = if let Some(ctx_parameter) = &ctx_parameter {
        quote! {
            return Err(crate::IntoJSError::into_js(
                crate::error::CommonError::UnsupportedPlatform(#error_message.into()),
                &#ctx_parameter
            ));
        }
    } else {
        quote! {
            return Err(crate::error::CommonError::UnsupportedPlatform(#error_message.into()).into());
        }
    };

    let guard_statement: syn::Stmt = parse_quote! {
        {
            let __actiona_platform = #platform_expression;
            if !(#platform_check) {
                #unsupported_platform_error
            }
        }
    };
    function.block.stmts.insert(0, guard_statement);

    if !doc_contains(&function.attrs, INSTR_PLATFORMS) {
        let rustdoc_instruction = arguments.rustdoc_instruction();
        let rustdoc_instruction_attribute: Attribute = parse_quote! {
            #[doc = #rustdoc_instruction]
        };
        function.attrs.push(rustdoc_instruction_attribute);
    }

    quote!(#function).into()
}

/// Apply `#[platform]` to non-functions (structs/enums) for rustdoc only.
fn expand_non_function_item(arguments: PlatformAttributeArguments, item: Item) -> TokenStream {
    let rustdoc_instruction = arguments.rustdoc_instruction();
    let rustdoc_instruction_attribute: Attribute = parse_quote! {
        #[doc = #rustdoc_instruction]
    };

    match item {
        Item::Struct(mut item_struct) => {
            if !doc_contains(&item_struct.attrs, INSTR_PLATFORMS) {
                item_struct.attrs.push(rustdoc_instruction_attribute);
            }
            quote!(#item_struct).into()
        }
        Item::Enum(mut item_enum) => {
            if !doc_contains(&item_enum.attrs, INSTR_PLATFORMS) {
                item_enum.attrs.push(rustdoc_instruction_attribute);
            }
            quote!(#item_enum).into()
        }
        Item::Type(mut item_type) => {
            if !doc_contains(&item_type.attrs, INSTR_PLATFORMS) {
                item_type.attrs.push(rustdoc_instruction_attribute);
            }
            quote!(#item_type).into()
        }
        unsupported_item => syn::Error::new_spanned(
            unsupported_item,
            "`platform` only supports functions, structs, enums, and type aliases",
        )
        .to_compile_error()
        .into(),
    }
}

#[derive(Clone, Copy)]
enum PlatformMode {
    Only,
    Not,
}

#[derive(Clone, Copy)]
enum GuardedPlatform {
    Linux,
    Windows,
    Wayland,
    X11,
}

impl GuardedPlatform {
    fn parse(platform: &str, span: Span) -> syn::Result<Self> {
        match platform.to_ascii_lowercase().as_str() {
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

    const fn name_for_messages(self) -> &'static str {
        match self {
            Self::Linux => PLATFORM_DISPLAY_LINUX,
            Self::Windows => PLATFORM_DISPLAY_WINDOWS,
            Self::Wayland => PLATFORM_DISPLAY_WAYLAND,
            Self::X11 => PLATFORM_DISPLAY_X11,
        }
    }

    const fn rustdoc_name(self) -> &'static str {
        match self {
            Self::Linux => PLATFORM_LINUX,
            Self::Windows => PLATFORM_WINDOWS,
            Self::Wayland => PLATFORM_WAYLAND,
            Self::X11 => PLATFORM_X11,
        }
    }
}

#[derive(Clone, Copy)]
struct PlatformAttributeArguments {
    mode: PlatformMode,
    platform: GuardedPlatform,
}

impl PlatformAttributeArguments {
    fn from_raw(raw_arguments: RawPlatformArguments) -> syn::Result<Self> {
        if raw_arguments.check.is_some() || raw_arguments.label.is_some() || raw_arguments.nested {
            return Err(syn::Error::new(
                Span::call_site(),
                "`platform` only supports `only` or `not`",
            ));
        }

        let only_platform = raw_arguments
            .only
            .map(|platform_name| GuardedPlatform::parse(&platform_name, Span::call_site()))
            .transpose()?;
        let excluded_platform = raw_arguments
            .not_platform
            .map(|platform_name| GuardedPlatform::parse(&platform_name, Span::call_site()))
            .transpose()?;

        match (only_platform, excluded_platform) {
            (Some(platform), None) => Ok(Self {
                mode: PlatformMode::Only,
                platform,
            }),
            (None, Some(platform)) => Ok(Self {
                mode: PlatformMode::Not,
                platform,
            }),
            (Some(_), Some(_)) => Err(syn::Error::new(
                Span::call_site(),
                "`platform` accepts either `only` or `not`, not both",
            )),
            (None, None) => Err(syn::Error::new(
                Span::call_site(),
                "`platform` requires one of `only = \"...\"` or `not = \"...\"`",
            )),
        }
    }

    fn platform_check(self) -> proc_macro2::TokenStream {
        match (self.mode, self.platform) {
            (PlatformMode::Only, GuardedPlatform::Linux) => {
                quote! { __actiona_platform.is_linux() }
            }
            (PlatformMode::Only, GuardedPlatform::Windows) => {
                quote! { __actiona_platform.is_windows() }
            }
            (PlatformMode::Only, GuardedPlatform::Wayland) => {
                quote! { __actiona_platform.is_wayland() }
            }
            (PlatformMode::Only, GuardedPlatform::X11) => {
                quote! { matches!(__actiona_platform, crate::platform_info::Platform::X11) }
            }
            (PlatformMode::Not, GuardedPlatform::Linux) => {
                quote! { !__actiona_platform.is_linux() }
            }
            (PlatformMode::Not, GuardedPlatform::Windows) => {
                quote! { !__actiona_platform.is_windows() }
            }
            (PlatformMode::Not, GuardedPlatform::Wayland) => {
                quote! { !__actiona_platform.is_wayland() }
            }
            (PlatformMode::Not, GuardedPlatform::X11) => {
                quote! { !matches!(__actiona_platform, crate::platform_info::Platform::X11) }
            }
        }
    }

    fn error_message(self) -> String {
        match self.mode {
            PlatformMode::Only => {
                format!("only available on {}", self.platform.name_for_messages())
            }
            PlatformMode::Not => {
                format!("not supported on {}", self.platform.name_for_messages())
            }
        }
    }

    fn rustdoc_instruction(self) -> String {
        match self.mode {
            PlatformMode::Only => format!("{INSTR_PLATFORMS} ={}", self.platform.rustdoc_name()),
            PlatformMode::Not => format!("{INSTR_PLATFORMS} -{}", self.platform.rustdoc_name()),
        }
    }
}

/// True when the function returns a `Result`.
fn returns_result(output: &ReturnType) -> bool {
    let ReturnType::Type(_, output_type) = output else {
        return false;
    };
    let Type::Path(type_path) = output_type.as_ref() else {
        return false;
    };
    type_path
        .path
        .segments
        .last()
        .is_some_and(|path_segment| path_segment.ident == "Result")
}

/// Find the `Ctx` parameter, if present.
fn find_ctx_parameter(function: &ItemFn) -> syn::Result<Option<Ident>> {
    let mut ctx_parameter = None;

    for function_argument in &function.sig.inputs {
        let FnArg::Typed(argument_type) = function_argument else {
            continue;
        };

        if !type_is_ctx(argument_type.ty.as_ref()) {
            continue;
        }

        let Pat::Ident(pattern_identifier) = argument_type.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &argument_type.pat,
                "`platform` requires `Ctx` parameters to use a simple identifier",
            ));
        };

        if ctx_parameter.is_some() {
            return Err(syn::Error::new_spanned(
                &function.sig.inputs,
                "`platform` found multiple `Ctx` parameters",
            ));
        }

        ctx_parameter = Some(pattern_identifier.ident.clone());
    }

    Ok(ctx_parameter)
}

/// Detect the rquickjs `Ctx` type in a parameter.
fn type_is_ctx(type_: &Type) -> bool {
    match type_ {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .is_some_and(|path_segment| path_segment.ident == "Ctx"),
        Type::Reference(type_reference) => type_is_ctx(type_reference.elem.as_ref()),
        Type::Paren(type_paren) => type_is_ctx(type_paren.elem.as_ref()),
        Type::Group(type_group) => type_is_ctx(type_group.elem.as_ref()),
        _ => false,
    }
}
