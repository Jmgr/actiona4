use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, parse_macro_input};

pub(crate) fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    if !matches!(ast.data, Data::Struct(_)) {
        return syn::Error::new_spanned(ast, "Parameter can only be used on structs")
            .to_compile_error()
            .into();
    };

    let name = &ast.ident;

    let name_str = name.to_string();
    let Some(kind_str) = name_str.strip_suffix("Parameter") else {
        return syn::Error::new_spanned(&ast, "Struct name should have a Parameter suffix")
            .to_compile_error()
            .into();
    };
    let kind_name = format_ident!("{kind_str}");

    // `#[parameter(storage = T)]` also emits `impl ParameterStorage for T`.
    // `T` is a type (e.g. `Scriptable<bool>`), which can't go through darling:
    // an attribute `name = value` value must be an expression, and a generic
    // type isn't one — so we parse the value tokens as a `syn::Type` directly.
    // Without the attribute, only `into_kind` is generated (e.g. `EnumParameter`,
    // whose storage impl is generated per-enum by `ActionEnum`).
    let mut storage: Option<syn::Type> = None;
    for attr in ast.attrs.iter().filter(|a| a.path().is_ident("parameter")) {
        let result = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("storage") {
                storage = Some(meta.value()?.parse()?);
                Ok(())
            } else {
                Err(meta.error("unknown `parameter` option (expected `storage`)"))
            }
        });
        if let Err(err) = result {
            return err.to_compile_error().into();
        }
    }

    let storage_impl = storage.map(|storage| {
        quote! {
            impl crate::parameters::ParameterStorage for #storage {
                type Settings = #name;
                const DEFAULT_SETTINGS: Self::Settings = #name::DEFAULT;
                const KIND: crate::parameters::ParameterKind = Self::DEFAULT_SETTINGS.into_kind();
            }
        }
    });

    let expanded = quote! {
        impl #name {
            pub const fn into_kind(self) -> crate::parameters::ParameterKind {
                crate::parameters::ParameterKind::#kind_name(self)
            }
        }

        #storage_impl
    };

    expanded.into()
}
