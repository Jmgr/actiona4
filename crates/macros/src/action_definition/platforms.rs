use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Expr, Ident};

/// Parse the `only`/`not` platform expressions from `#[action(...)]`, `#[parameter(...)]`,
/// or `#[action_enum(...)]` into `types::platform::PlatformConstraint` tokens for a
/// `Platforms(&[...])` literal.
///
/// Accepts a single platform variant (`only = Linux`) or a bracketed list of
/// them (`only = [Linux, Windows]`), mirroring `types::platform::PlatformKind`'s
/// variant names. The variant name isn't validated here; an unknown one simply
/// fails to compile in the generated `PlatformKind::#name` reference, the same
/// way `icon`/`effect`/`category` are handled.
pub fn platform_constraints(
    only: Option<&Expr>,
    not: Option<&Expr>,
) -> syn::Result<Vec<TokenStream>> {
    let mut constraints = Vec::new();
    if let Some(expr) = only {
        for kind in platform_kinds(expr)? {
            constraints
                .push(quote! { ::types::platform::PlatformConstraint::Only(::types::platform::PlatformKind::#kind) });
        }
    }
    if let Some(expr) = not {
        for kind in platform_kinds(expr)? {
            constraints
                .push(quote! { ::types::platform::PlatformConstraint::Not(::types::platform::PlatformKind::#kind) });
        }
    }
    Ok(constraints)
}

fn platform_kinds(expr: &Expr) -> syn::Result<Vec<Ident>> {
    match expr {
        Expr::Array(array) => array.elems.iter().map(platform_kind).collect(),
        _ => Ok(vec![platform_kind(expr)?]),
    }
}

fn platform_kind(expr: &Expr) -> syn::Result<Ident> {
    let Expr::Path(path) = expr else {
        return Err(Error::new_spanned(
            expr,
            "expected a platform variant, e.g. `Linux` or `[Linux, Windows]`",
        ));
    };
    path.path
        .segments
        .last()
        .map(|segment| segment.ident.clone())
        .ok_or_else(|| Error::new_spanned(path, "expected a platform variant"))
}
