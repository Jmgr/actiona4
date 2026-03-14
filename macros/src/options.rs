use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Attribute, Expr, ExprLit, ExprPath, ExprUnary, Field, Fields, Item, ItemStruct, Lit, LitFloat,
    LitInt, Meta, Path, PathArguments, Type, UnOp, Visibility, parse::Parser, parse_macro_input,
    parse_quote, punctuated::Punctuated, token::Comma,
};

use crate::{
    consts::{INSTR_DEFAULT, INSTR_OPTIONS, INSTR_PLATFORMS, INSTR_SKIP, RAW_IDENT_PREFIX},
    default_args::{
        JsDefaultArguments, PlatformArguments, doc_contains, parse_attribute_arguments,
        parse_default_arguments,
    },
};

/// Expand `#[options]` to rustdoc instructions and generated defaults.
pub(crate) fn expand(
    arguments: TokenStream,
    item: TokenStream,
    attribute_name: &str,
    add_options_instruction: bool,
) -> TokenStream {
    if !proc_macro2::TokenStream::from(arguments).is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("`{attribute_name}` takes no arguments"),
        )
        .to_compile_error()
        .into();
    }

    let mut item = parse_macro_input!(item as Item);
    let expansion_result = match &mut item {
        Item::Struct(item_struct) => expand_struct(item_struct, add_options_instruction),
        Item::Enum(_) => Err(syn::Error::new_spanned(
            &item,
            format!("`{attribute_name}` does not support enums; use `#[js_enum]` instead"),
        )),
        _ => Err(syn::Error::new_spanned(
            &item,
            format!("`{attribute_name}` only supports structs"),
        )),
    };

    match expansion_result {
        Ok(extra_tokens) => quote!(#item #extra_tokens).into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Expand a struct by injecting doc instructions and default metadata.
fn expand_struct(
    item_struct: &mut ItemStruct,
    add_options_instruction: bool,
) -> syn::Result<TokenStream2> {
    if add_options_instruction && !doc_contains(&item_struct.attrs, INSTR_OPTIONS) {
        append_doc_line(&mut item_struct.attrs, INSTR_OPTIONS.to_string());
    }

    let Fields::Named(fields_named) = &mut item_struct.fields else {
        return Err(syn::Error::new_spanned(
            &item_struct.fields,
            "`options` only supports structs with named fields",
        ));
    };

    if add_options_instruction && has_derive(&item_struct.attrs, "Default") {
        return Err(syn::Error::new_spanned(
            &item_struct.ident,
            "`#[options]` already generates `Default`; remove `Default` from the derive list",
        ));
    }

    let mut default_assignments = Vec::with_capacity(fields_named.named.len());
    for field in fields_named.named.iter_mut() {
        let is_public = matches!(field.vis, Visibility::Public(_));
        let is_skipped = doc_contains(&field.attrs, INSTR_SKIP);
        let infer_when_attribute_missing = is_public && !is_skipped;

        let field_defaults = compute_field_defaults(field, infer_when_attribute_missing)?;

        if infer_when_attribute_missing
            && !doc_contains(&field.attrs, INSTR_DEFAULT)
            && let Some(ts_default) = &field_defaults.ts_default
        {
            append_doc_line(&mut field.attrs, format!("{INSTR_DEFAULT} `{ts_default}`"));
        }

        if !doc_contains(&field.attrs, INSTR_PLATFORMS)
            && let Some(platform_name) = platform_only_from_attributes(&field.attrs)?
        {
            append_doc_line(
                &mut field.attrs,
                format!("{INSTR_PLATFORMS} ={platform_name}"),
            );
        }

        field
            .attrs
            .retain(|attribute| !is_default_attribute(attribute));

        let Some(field_name) = &field.ident else {
            continue;
        };
        let rust_default_expr = field_defaults.rust_default_expr;
        default_assignments.push(quote! {
            #field_name: #rust_default_expr
        });
    }

    if !add_options_instruction {
        return Ok(TokenStream2::new());
    }

    let struct_name = &item_struct.ident;
    let generics = item_struct.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let extra_tokens = quote! {
        impl #impl_generics ::core::default::Default for #struct_name #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(#default_assignments),*
                }
            }
        }

    };

    Ok(extra_tokens)
}

/// Append a single `#[doc = "..."]` line to an item.
pub(crate) fn append_doc_line(attributes: &mut Vec<Attribute>, doc_line: String) {
    let doc_attribute: Attribute = parse_quote! {
        #[doc = #doc_line]
    };
    attributes.push(doc_attribute);
}

/// Check if `#[derive(...)]` contains a target trait.
fn has_derive(attributes: &[Attribute], target: &str) -> bool {
    attributes.iter().any(|attribute| {
        if !attribute.path().is_ident("derive") {
            return false;
        }

        let Meta::List(meta_list) = &attribute.meta else {
            return false;
        };

        let existing_paths: Punctuated<Path, Comma> = Punctuated::<Path, Comma>::parse_terminated
            .parse2(meta_list.tokens.clone())
            .unwrap_or_default();

        existing_paths.iter().any(|path| path.is_ident(target))
    })
}

/// True when this attribute is `#[default(...)]`.
fn is_default_attribute(attribute: &Attribute) -> bool {
    attribute.path().is_ident("default")
}

/// Extract `platform(only = "...")` from a field or variant.
pub(crate) fn platform_only_from_attributes(
    attributes: &[Attribute],
) -> syn::Result<Option<String>> {
    let mut only_platform: Option<String> = None;

    for attribute in attributes {
        if !attribute.path().is_ident("platform") {
            continue;
        }

        let parsed_arguments: PlatformArguments = parse_attribute_arguments(attribute, "platform")?;

        if let Some(only_value) = parsed_arguments.only {
            if only_platform.is_some() {
                return Err(syn::Error::new_spanned(attribute, "duplicate `only` value"));
            }

            only_platform = Some(only_value);
        }
    }

    Ok(only_platform.map(|platform| platform.to_ascii_lowercase()))
}

struct FieldDefaults {
    rust_default_expr: Expr,
    ts_default: Option<String>,
}

/// Compute Rust and TS defaults for a field based on `#[default]`.
fn compute_field_defaults(
    field: &Field,
    infer_when_attribute_missing: bool,
) -> syn::Result<FieldDefaults> {
    let mut has_default_attribute = false;
    let mut rust_default_expr: Option<Expr> = None;
    let mut ts_default_value: Option<String> = None;

    for attribute in &field.attrs {
        if !attribute.path().is_ident("default") {
            continue;
        }
        has_default_attribute = true;

        let parsed_arguments: JsDefaultArguments = parse_default_arguments(attribute, "default")?;

        if let Some(rust_value) = parsed_arguments.rust {
            if rust_default_expr.is_some() {
                return Err(syn::Error::new_spanned(attribute, "duplicate `rust` value"));
            }

            rust_default_expr = Some(rust_value);
        }

        if let Some(ts_value) = parsed_arguments.ts {
            if ts_default_value.is_some() {
                return Err(syn::Error::new_spanned(attribute, "duplicate `ts` value"));
            }

            ts_default_value = Some(ts_value);
        }
    }

    let rust_default_expr_for_inference = rust_default_expr.clone();
    let rust_default_expr =
        rust_default_expr.unwrap_or_else(|| syn::parse_quote!(::core::default::Default::default()));

    let ts_default = if !has_default_attribute {
        if infer_when_attribute_missing {
            let Some(inferred_ts_default) = infer_ts_default_from_type_default(&field.ty) else {
                let field_name = field
                    .ident
                    .as_ref()
                    .map_or("<unnamed field>".to_string(), ToString::to_string);
                return Err(syn::Error::new_spanned(
                    &field.ty,
                    format!(
                        "could not infer `ts` default for public field `{field_name}`; add #[default(...)] (for example #[default(ts = \"...\")])"
                    ),
                ));
            };
            Some(inferred_ts_default)
        } else {
            None
        }
    } else if let Some(ts_default_value) = ts_default_value {
        Some(ts_default_value)
    } else if let Some(rust_default_expr) = &rust_default_expr_for_inference {
        let Some(inferred_ts_default) = infer_ts_default_from_expr(rust_default_expr) else {
            return Err(syn::Error::new_spanned(
                rust_default_expr,
                "could not infer `ts` default from `rust`; specify `ts = \"...\"` explicitly",
            ));
        };
        Some(inferred_ts_default)
    } else {
        let Some(inferred_ts_default) = infer_ts_default_from_type_default(&field.ty) else {
            return Err(syn::Error::new_spanned(
                &field.ty,
                "could not infer `ts` default from the field type; specify `ts = \"...\"` explicitly",
            ));
        };
        Some(inferred_ts_default)
    };

    Ok(FieldDefaults {
        rust_default_expr,
        ts_default,
    })
}

/// Infer a TS default from the Rust type default.
fn infer_ts_default_from_type_default(field_type: &Type) -> Option<String> {
    let type_path = match field_type {
        Type::Path(type_path) => type_path,
        Type::Paren(type_paren) => return infer_ts_default_from_type_default(&type_paren.elem),
        Type::Group(type_group) => return infer_ts_default_from_type_default(&type_group.elem),
        _ => return None,
    };

    let path_segment = type_path.path.segments.last()?;
    let type_name = path_segment.ident.to_string();
    let normalized_type_name = type_name
        .strip_prefix(RAW_IDENT_PREFIX)
        .unwrap_or(&type_name);

    match normalized_type_name {
        "bool" => Some("false".to_string()),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" => Some("0".to_string()),
        "f32" | "f64" => Some("0".to_string()),
        "Option" => Some("undefined".to_string()),
        "Vec" => Some("[]".to_string()),
        "String" => Some("\"\"".to_string()),
        _ => None,
    }
}

/// Infer a TS default string from a Rust expression.
fn infer_ts_default_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(expr_lit) => infer_ts_default_from_literal(expr_lit),
        Expr::Path(expr_path) => infer_ts_default_from_path(expr_path),
        Expr::Paren(expr_paren) => infer_ts_default_from_expr(&expr_paren.expr),
        Expr::Group(expr_group) => infer_ts_default_from_expr(&expr_group.expr),
        Expr::Unary(expr_unary) => infer_ts_default_from_unary(expr_unary),
        _ => None,
    }
}

/// Infer a TS default string from a literal expression.
fn infer_ts_default_from_literal(expr_lit: &ExprLit) -> Option<String> {
    match &expr_lit.lit {
        Lit::Bool(lit_bool) => Some(lit_bool.value.to_string()),
        Lit::Int(lit_int) => Some(normalize_integer_literal(lit_int)),
        Lit::Float(lit_float) => Some(normalize_float_literal(lit_float)),
        Lit::Str(lit_str) => serde_json::to_string(&lit_str.value()).ok(),
        _ => None,
    }
}

/// Infer a TS default string from a path expression.
fn infer_ts_default_from_path(expr_path: &ExprPath) -> Option<String> {
    if expr_path.qself.is_some() {
        return None;
    }

    if expr_path.path.is_ident("None") {
        return Some("undefined".to_string());
    }

    let mut path_parts = Vec::new();
    for path_segment in &expr_path.path.segments {
        if !matches!(path_segment.arguments, PathArguments::None) {
            return None;
        }

        let path_segment_string = path_segment.ident.to_string();
        let normalized_path_segment = path_segment_string
            .strip_prefix(RAW_IDENT_PREFIX)
            .unwrap_or(&path_segment_string);
        path_parts.push(normalized_path_segment.to_string());
    }

    if path_parts.len() < 2 {
        return None;
    }

    Some(path_parts.join("."))
}

/// Infer a TS default string from a unary expression.
fn infer_ts_default_from_unary(expr_unary: &ExprUnary) -> Option<String> {
    if !matches!(expr_unary.op, UnOp::Neg(_)) {
        return None;
    }

    let Expr::Lit(expr_lit) = expr_unary.expr.as_ref() else {
        return None;
    };
    let numeric_literal = match &expr_lit.lit {
        Lit::Int(lit_int) => normalize_integer_literal(lit_int),
        Lit::Float(lit_float) => normalize_float_literal(lit_float),
        _ => return None,
    };

    Some(format!("-{numeric_literal}"))
}

/// Normalize integer literal syntax to a plain string.
fn normalize_integer_literal(lit_int: &LitInt) -> String {
    let mut literal_string = lit_int.to_string();
    let literal_suffix = lit_int.suffix();
    if !literal_suffix.is_empty() {
        let literal_length_without_suffix = literal_string.len() - literal_suffix.len();
        literal_string.truncate(literal_length_without_suffix);
    }

    literal_string.replace('_', "")
}

/// Normalize float literal syntax to a plain string.
fn normalize_float_literal(lit_float: &LitFloat) -> String {
    let mut literal_string = lit_float.to_string();
    let literal_suffix = lit_float.suffix();
    if !literal_suffix.is_empty() {
        let literal_length_without_suffix = literal_string.len() - literal_suffix.len();
        literal_string.truncate(literal_length_without_suffix);
    }

    let mut normalized = literal_string.replace('_', "");
    if normalized.ends_with(".0") {
        normalized.truncate(normalized.len() - 2);
    }
    normalized
}
