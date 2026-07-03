use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, parse_macro_input};

pub(crate) fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let Data::Enum(data) = &ast.data else {
        return Error::new_spanned(ast, "ActionDefinitions can only be used on enums")
            .to_compile_error()
            .into();
    };

    // Each variant is a newtype over its action struct (`Click(Click)`), so the
    // field type is the action whose `DEFINITION` const we list.
    let definitions = data
        .variants
        .iter()
        .map(|variant| {
            let Fields::Unnamed(fields) = &variant.fields else {
                return Err(Error::new_spanned(
                    variant,
                    "ActionDefinitions requires newtype variants like `Click(Click)`",
                ));
            };
            let mut fields = fields.unnamed.iter();
            let (Some(field), None) = (fields.next(), fields.next()) else {
                return Err(Error::new_spanned(
                    variant,
                    "ActionDefinitions requires each variant to hold exactly one action type",
                ));
            };
            let ty = &field.ty;
            Ok(quote! { <#ty>::DEFINITION })
        })
        .collect::<Result<Vec<_>, Error>>();

    let definitions = match definitions {
        Ok(definitions) => definitions,
        Err(err) => return err.to_compile_error().into(),
    };

    quote! {
        pub const ACTION_DEFINITIONS: &[crate::actions::ActionDefinition] =
            &[#(#definitions),*];
    }
    .into()
}
