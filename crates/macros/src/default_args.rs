use darling::{FromMeta, ast::NestedMeta};
use proc_macro2::Span;
use syn::{
    Attribute, Expr, Ident, Lit, LitStr, Meta, Token,
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
};

#[derive(Clone, Debug, Default, FromMeta)]
pub(crate) struct JsDefaultArguments {
    pub rust: Option<Expr>,
    pub ts: Option<String>,
}

#[derive(Clone, Debug, Default, FromMeta)]
pub(crate) struct PlatformArguments {
    pub only: Option<String>,
    #[darling(rename = "not")]
    pub not_platform: Option<String>,
    pub check: Option<String>,
    pub label: Option<String>,
    #[darling(default)]
    pub nested: bool,
}

/// Parse a `#[name(...)]` attribute into a darling-backed struct.
pub(crate) fn parse_attribute_arguments<T: FromMeta + Default>(
    attribute: &Attribute,
    expected_name: &str,
) -> syn::Result<T> {
    if !attribute.path().is_ident(expected_name) {
        return Err(syn::Error::new_spanned(
            attribute,
            format!("expected `#[{expected_name}(..)]`"),
        ));
    }

    match &attribute.meta {
        Meta::Path(_) => {
            parse_meta_list_tokens::<T>(proc_macro2::TokenStream::new(), attribute.span())
        }
        Meta::List(meta_list) => {
            parse_meta_list_tokens::<T>(meta_list.tokens.clone(), attribute.span())
        }
        Meta::NameValue(_) => Err(syn::Error::new_spanned(
            attribute,
            format!("`{expected_name}` expects list-style arguments"),
        )),
    }
}

/// Parse a raw token stream as a `MetaList` for darling.
pub(crate) fn parse_meta_list_tokens<T: FromMeta + Default>(
    tokens: proc_macro2::TokenStream,
    error_span: Span,
) -> syn::Result<T> {
    let nested_items =
        NestedMeta::parse_meta_list(tokens).map_err(|error| syn::Error::new(error_span, error))?;
    T::from_list(&nested_items).map_err(|error| syn::Error::new(error_span, error))
}

enum DefaultArgumentEntry {
    Positional(Expr),
    Rust(Expr),
    Ts(String),
}

impl Parse for DefaultArgumentEntry {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.peek(Ident) && input.peek2(Token![=]) {
            let name: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;

            match name.to_string().as_str() {
                "rust" => Ok(Self::Rust(input.parse()?)),
                "ts" => {
                    let value: LitStr = input.parse()?;
                    Ok(Self::Ts(value.value()))
                }
                _ => Err(syn::Error::new_spanned(
                    name,
                    "unsupported argument, expected `rust` or `ts`",
                )),
            }
        } else {
            Ok(Self::Positional(input.parse()?))
        }
    }
}

/// Return `true` when any `#[doc = "..."]` attribute on `attributes` contains `needle`.
pub(crate) fn doc_contains(attributes: &[Attribute], needle: &str) -> bool {
    attributes.iter().any(|attribute| {
        if !attribute.path().is_ident("doc") {
            return false;
        }
        let Meta::NameValue(nv) = &attribute.meta else {
            return false;
        };
        let syn::Expr::Lit(expr_lit) = &nv.value else {
            return false;
        };
        let Lit::Str(text) = &expr_lit.lit else {
            return false;
        };
        text.value().contains(needle)
    })
}

/// Parse `#[default(...)]` arguments into Rust/TS defaults.
pub(crate) fn parse_default_arguments(
    attribute: &Attribute,
    expected_name: &str,
) -> syn::Result<JsDefaultArguments> {
    if !attribute.path().is_ident(expected_name) {
        return Err(syn::Error::new_spanned(
            attribute,
            format!("expected `#[{expected_name}(..)]`"),
        ));
    }

    let tokens = match &attribute.meta {
        Meta::Path(_) => return Ok(JsDefaultArguments::default()),
        Meta::List(meta_list) => meta_list.tokens.clone(),
        Meta::NameValue(_) => {
            return Err(syn::Error::new_spanned(
                attribute,
                format!("`{expected_name}` expects list-style arguments"),
            ));
        }
    };

    if tokens.is_empty() {
        return Ok(JsDefaultArguments::default());
    }

    let entries = Punctuated::<DefaultArgumentEntry, Token![,]>::parse_terminated.parse2(tokens)?;

    let mut result = JsDefaultArguments::default();
    for entry in entries {
        match entry {
            DefaultArgumentEntry::Positional(value) => {
                if result.rust.is_some() {
                    return Err(syn::Error::new_spanned(
                        value,
                        "duplicate Rust default value",
                    ));
                }
                result.rust = Some(value);
            }
            DefaultArgumentEntry::Rust(value) => {
                if result.rust.is_some() {
                    return Err(syn::Error::new_spanned(value, "duplicate `rust` value"));
                }
                result.rust = Some(value);
            }
            DefaultArgumentEntry::Ts(value) => {
                if result.ts.is_some() {
                    return Err(syn::Error::new(attribute.span(), "duplicate `ts` value"));
                }
                result.ts = Some(value);
            }
        }
    }

    Ok(result)
}
