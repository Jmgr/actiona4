use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Fields, ItemStruct, parse_macro_input};

use crate::action_definition::action::process_parameter_field;

/// Like `#[action]`'s field processing, but for a plain struct of
/// `#[parameter]` fields that isn't itself an action — no icon, effect,
/// category, `DEFINITION`, or `ActionInstance` variant. Used for
/// `CommonParameters`, the fields every action carries via `WithCommon`.
///
/// Each field gets the same marker + `Param<T, Marker>` treatment `#[action]`
/// gives its own fields, plus a `<FIELD>_PARAMETER` const holding the
/// generated `Parameter` definition, so an action's own `#[action(...)]`
/// expansion can splice specific common fields into its parameter list (e.g.
/// `timeout`, gated on `supports_timeout`).
pub(crate) fn expand(item: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(item as ItemStruct);

    let Fields::Named(fields) = &mut item.fields else {
        return syn::Error::new_spanned(
            item,
            "common_parameters can only be used on structs with named fields",
        )
        .to_compile_error()
        .into();
    };

    let name = &item.ident;
    let visibility = &item.vis;

    let mut markers = Vec::new();
    let mut consts = Vec::new();

    for field in &mut fields.named {
        let processed = match process_parameter_field(name, "common", visibility, field) {
            Ok(Some(processed)) => processed,
            Ok(None) => continue,
            Err(err) => return err.to_compile_error().into(),
        };

        let const_name = format_ident!(
            "{}_PARAMETER",
            processed.field_name.to_case(Case::UpperSnake)
        );
        let parameter = &processed.parameter;
        markers.push(processed.markers);
        consts.push(quote! {
            pub const #const_name: crate::parameters::Parameter = #parameter;
        });
    }

    quote! {
        #(#markers)*

        #item

        impl #name {
            #(#consts)*
        }
    }
    .into()
}
