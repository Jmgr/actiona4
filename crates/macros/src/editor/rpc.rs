use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, FnArg, ItemTrait, Pat, ReturnType, TraitItem, parse_macro_input};

struct CallMethod {
    ident: syn::Ident,
    module: syn::Ident,
    name: String,
    arg_idents: Vec<syn::Ident>,
    arg_types: Vec<syn::Type>,
    ret: proc_macro2::TokenStream,
}

/// One declaration → a typed client, a host dispatcher, and shared wire types;
/// see [`crate::rpc`].
pub fn expand(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_def = parse_macro_input!(item as ItemTrait);
    let vis = trait_def.vis.clone();
    let trait_ident = trait_def.ident.clone();
    let client_ident = format_ident!("{trait_ident}Client");
    let serve_ident = format_ident!("{}_serve", trait_ident.to_string().to_case(Case::Snake));

    let mut methods = Vec::new();
    for item in &trait_def.items {
        let TraitItem::Fn(method) = item else {
            return Error::new_spanned(item, "#[rpc] traits may only contain methods")
                .to_compile_error()
                .into();
        };

        let mut arg_idents = Vec::new();
        let mut arg_types = Vec::new();
        for input in &method.sig.inputs {
            match input {
                FnArg::Receiver(_) => {} // `&self`
                FnArg::Typed(arg) => {
                    let Pat::Ident(pat) = &*arg.pat else {
                        return Error::new_spanned(
                            &arg.pat,
                            "#[rpc] arguments must be plain identifiers",
                        )
                        .to_compile_error()
                        .into();
                    };
                    arg_idents.push(pat.ident.clone());
                    arg_types.push((*arg.ty).clone());
                }
            }
        }

        let ret = match &method.sig.output {
            ReturnType::Default => quote! { () },
            ReturnType::Type(_, ty) => quote! { #ty },
        };

        let ident = method.sig.ident.clone();
        methods.push(CallMethod {
            module: format_ident!("__rpc_{ident}"),
            name: ident.to_string(),
            ident,
            arg_idents,
            arg_types,
            ret,
        });
    }

    // The server-side trait: `async fn` rewritten to an explicit `Send` future,
    // so the boxed dispatch future can cross the threadpool boundary.
    let trait_fns = methods.iter().map(|m| {
        let (ident, ai, at, ret) = (&m.ident, &m.arg_idents, &m.arg_types, &m.ret);
        quote! {
            fn #ident(&self, #(#ai: #at),*)
                -> impl ::std::future::Future<Output = #ret> + ::std::marker::Send;
        }
    });

    // Per-method module shared by client and server: the by-name argument
    // struct (serde) plus the wire name.
    let modules = methods.iter().map(|m| {
        let (module, name, ai, at) = (&m.module, &m.name, &m.arg_idents, &m.arg_types);
        quote! {
            #[doc(hidden)]
            #vis mod #module {
                use super::*;
                pub const NAME: &str = #name;
                #[derive(::serde::Serialize, ::serde::Deserialize)]
                pub struct Args { #(pub #ai: #at),* }
            }
        }
    });

    // Client methods: serialize args, hand them to the transport, deserialize.
    let client_fns = methods.iter().map(|m| {
        let (ident, module, ai, at, ret) =
            (&m.ident, &m.module, &m.arg_idents, &m.arg_types, &m.ret);
        quote! {
            pub async fn #ident(&self, #(#ai: #at),*)
                -> ::core::result::Result<#ret, RpcError<T::Error>> {
                let __input = ::serde_json::to_value(&#module::Args { #(#ai),* })
                    .map_err(RpcError::Serialize)?;
                let __output = self.transport.request(#module::NAME, __input).await
                    .map_err(RpcError::Transport)?;
                ::serde_json::from_value(__output).map_err(RpcError::Deserialize)
            }
        }
    });

    // Server dispatch arms: deserialize args, call the impl, serialize result.
    let serve_arms = methods.iter().map(|m| {
        let (ident, module, ai) = (&m.ident, &m.module, &m.arg_idents);
        quote! {
            __cmd if __cmd == #module::NAME => {
                let __args: #module::Args = ::serde_json::from_value(input)
                    .expect("RPC: failed to deserialize request");
                ::std::option::Option::Some(::std::boxed::Box::pin(async move {
                    let __out = api.#ident(#(__args.#ai),*).await;
                    ::serde_json::to_value(&__out)
                        .expect("RPC: failed to serialize response")
                }))
            }
        }
    });

    quote! {
        #vis trait #trait_ident {
            #(#trait_fns)*
        }

        #(#modules)*

        #vis struct #client_ident<T> {
            transport: T,
        }

        impl<T: Transport> #client_ident<T> {
            pub fn new(transport: T) -> Self {
                Self { transport }
            }
            #(#client_fns)*
        }

        #vis fn #serve_ident<__A>(
            api: ::std::sync::Arc<__A>,
            cmd: &str,
            input: ::serde_json::Value,
        ) -> ::std::option::Option<::std::pin::Pin<::std::boxed::Box<
            dyn ::std::future::Future<Output = ::serde_json::Value> + ::std::marker::Send,
        >>>
        where
            __A: #trait_ident + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            match cmd {
                #(#serve_arms)*
                _ => ::std::option::Option::None,
            }
        }
    }
    .into()
}
