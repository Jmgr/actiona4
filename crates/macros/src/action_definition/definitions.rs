use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Error, Fields, GenericArgument, PathArguments, Type, parse_macro_input,
};

/// Each variant holds `WithCommon<Action>`; the `DEFINITION` const lives on the
/// inner `Action`, so unwrap the single generic argument of `WithCommon<_>`.
/// Any other type is returned unchanged.
fn inner_action_type(ty: &Type) -> &Type {
    let Type::Path(type_path) = ty else {
        return ty;
    };
    let Some(segment) = type_path.path.segments.last() else {
        return ty;
    };
    if segment.ident != "WithCommon" {
        return ty;
    }
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return ty;
    };
    match args.args.first() {
        Some(GenericArgument::Type(inner)) => inner,
        _ => ty,
    }
}

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
            let ty = inner_action_type(&field.ty);
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
