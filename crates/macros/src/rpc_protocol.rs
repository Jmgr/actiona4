use std::collections::HashSet;

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Attribute, FnArg, Ident, ItemTrait, Pat, ReturnType, TraitItem, Type, parse_macro_input,
};

pub(crate) fn expand(arguments: TokenStream, item: TokenStream) -> TokenStream {
    if !proc_macro2::TokenStream::from(arguments).is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "`rpc_protocol` takes no arguments",
        )
        .to_compile_error()
        .into();
    }

    let item_trait = parse_macro_input!(item as ItemTrait);
    match expand_trait(item_trait) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Identifiers for the four wire enums of a protocol. Variable names match
/// the `Protocol::*` associated type each enum implements, so that the call
/// sites below read in the obvious direction (host responses go into
/// `host_response_ident`, extension requests into `extension_request_ident`,
/// etc.).
struct ProtocolIdents {
    host_request: Ident,
    host_response: Ident,
    extension_request: Ident,
    extension_response: Ident,
}

impl ProtocolIdents {
    fn new(protocol_base: &str) -> Self {
        Self {
            host_request: format_ident!("{protocol_base}HostRequest"),
            host_response: format_ident!("{protocol_base}HostResponse"),
            extension_request: format_ident!("{protocol_base}ExtensionRequest"),
            extension_response: format_ident!("{protocol_base}ExtensionResponse"),
        }
    }
}

fn expand_trait(item_trait: ItemTrait) -> syn::Result<TokenStream2> {
    if !item_trait.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            item_trait.generics,
            "`rpc_protocol` does not support generic protocols",
        ));
    }

    let visibility = item_trait.vis;
    let protocol_ident = item_trait.ident;
    let protocol_base = protocol_base_name(&protocol_ident);
    let idents = ProtocolIdents::new(&protocol_base);
    let host_trait_ident = format_ident!("{protocol_ident}Host");
    let extension_trait_ident = format_ident!("{protocol_ident}Extension");

    let mut methods = Vec::new();
    let mut seen_names: HashSet<String> = HashSet::new();
    for item in item_trait.items {
        match item {
            TraitItem::Fn(method) => {
                let parsed = ProtocolMethod::from_trait_method(method)?;
                if !seen_names.insert(parsed.method_ident.to_string()) {
                    return Err(syn::Error::new_spanned(
                        &parsed.method_ident,
                        "`rpc_protocol` methods must have unique names",
                    ));
                }
                methods.push(parsed);
            }
            unsupported => {
                return Err(syn::Error::new_spanned(
                    unsupported,
                    "`rpc_protocol` only supports methods",
                ));
            }
        }
    }

    let host_call_methods: Vec<_> = methods
        .iter()
        .filter(|method| method.direction == CallDirection::Host)
        .collect();
    let extension_call_methods: Vec<_> = methods
        .iter()
        .filter(|method| method.direction == CallDirection::Extension)
        .collect();

    // host_call: host sends Request, extension answers with Response.
    // extension_call: extension sends Request, host answers with Response.
    let host_request_variants = host_call_methods
        .iter()
        .copied()
        .map(ProtocolMethod::request_variant);
    let extension_response_variants = host_call_methods
        .iter()
        .copied()
        .map(ProtocolMethod::response_variant);
    let extension_request_variants = extension_call_methods
        .iter()
        .copied()
        .map(ProtocolMethod::request_variant);
    let host_response_variants = extension_call_methods
        .iter()
        .copied()
        .map(ProtocolMethod::response_variant);

    let host_methods = host_call_methods
        .iter()
        .copied()
        .map(|method| method.call_method(&idents.host_request, &idents.extension_response));
    let extension_methods = extension_call_methods
        .iter()
        .copied()
        .map(|method| method.call_method(&idents.extension_request, &idents.host_response));

    let has_host_calls = !host_call_methods.is_empty();
    let has_extension_calls = !extension_call_methods.is_empty();

    let host_request_type = wire_type(&idents.host_request, has_host_calls);
    let host_response_type = wire_type(&idents.host_response, has_extension_calls);
    let extension_request_type = wire_type(&idents.extension_request, has_extension_calls);
    let extension_response_type = wire_type(&idents.extension_response, has_host_calls);

    let host_request_enum = enum_definition(
        &visibility,
        &idents.host_request,
        has_host_calls,
        host_request_variants,
    );
    let extension_response_enum = enum_definition(
        &visibility,
        &idents.extension_response,
        has_host_calls,
        extension_response_variants,
    );
    let extension_request_enum = enum_definition(
        &visibility,
        &idents.extension_request,
        has_extension_calls,
        extension_request_variants,
    );
    let host_response_enum = enum_definition(
        &visibility,
        &idents.host_response,
        has_extension_calls,
        host_response_variants,
    );
    let host_trait = side_trait(
        &visibility,
        &host_trait_ident,
        &idents.extension_request,
        &idents.host_response,
        has_extension_calls,
        extension_call_methods.iter().copied(),
    );
    let extension_trait = side_trait(
        &visibility,
        &extension_trait_ident,
        &idents.host_request,
        &idents.extension_response,
        has_host_calls,
        host_call_methods.iter().copied(),
    );
    let host_handler_impl = host_handler_impl(&host_trait_ident, has_extension_calls);
    let extension_handler_impl = extension_handler_impl(&extension_trait_ident, has_host_calls);

    Ok(quote! {
        #[derive(Debug, Clone)]
        #visibility struct #protocol_ident;

        #host_request_enum
        #host_response_enum
        #extension_request_enum
        #extension_response_enum
        #host_trait
        #extension_trait

        impl crate::protocol::Protocol for #protocol_ident {
            type HostRequest = #host_request_type;
            type HostResponse = #host_response_type;
            type ExtensionRequest = #extension_request_type;
            type ExtensionResponse = #extension_response_type;
        }

        impl crate::Host<#protocol_ident> {
            #(#host_methods)*
            #host_handler_impl
        }

        impl crate::Extension<#protocol_ident> {
            #(#extension_methods)*
            #extension_handler_impl
        }
    })
}

fn wire_type(ident: &Ident, has_methods: bool) -> TokenStream2 {
    if has_methods {
        quote!(#ident)
    } else {
        quote!(crate::protocol::Never)
    }
}

fn side_trait<'a>(
    visibility: &syn::Visibility,
    trait_ident: &Ident,
    request_ident: &Ident,
    response_ident: &Ident,
    has_methods: bool,
    methods: impl Iterator<Item = &'a ProtocolMethod>,
) -> TokenStream2 {
    let methods: Vec<_> = methods.collect();
    let trait_methods = methods.iter().map(|method| method.trait_method());
    let dispatch_arms = methods
        .iter()
        .map(|method| method.dispatch_arm(request_ident, response_ident));
    let dispatch_method = if has_methods {
        quote! {
            fn handle_request(
                &self,
                request: #request_ident,
            ) -> impl ::std::future::Future<
                Output = ::std::result::Result<#response_ident, ::std::string::String>,
            > + ::std::marker::Send + '_
            where
                Self: ::std::marker::Sync,
            {
                async move {
                    match request {
                        #(#dispatch_arms)*
                    }
                }
            }
        }
    } else {
        quote!()
    };

    quote! {
        #[allow(async_fn_in_trait)]
        #visibility trait #trait_ident {
            #(#trait_methods)*
            #dispatch_method
        }
    }
}

fn host_handler_impl(host_trait_ident: &Ident, should_generate: bool) -> TokenStream2 {
    if !should_generate {
        return quote!();
    }

    quote! {
        pub async fn with_handler_impl<H>(
            executable_path: &::std::path::Path,
            task_tracker: ::tokio_util::task::TaskTracker,
            token: ::tokio_util::sync::CancellationToken,
            timeout: ::std::time::Duration,
            handler: H,
        ) -> color_eyre::Result<Self>
        where
            H: #host_trait_ident + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            let handler = ::std::sync::Arc::new(handler);
            Self::with_handler(
                executable_path,
                task_tracker,
                token,
                timeout,
                move |request| {
                    let handler = ::std::sync::Arc::clone(&handler);
                    async move {
                        <H as #host_trait_ident>::handle_request(&*handler, request).await
                    }
                },
            )
            .await
        }
    }
}

fn extension_handler_impl(extension_trait_ident: &Ident, should_generate: bool) -> TokenStream2 {
    if !should_generate {
        return quote!();
    }

    quote! {
        pub fn with_handler_impl<H>(
            key: ::ipc_rpc::ConnectionKey,
            task_tracker: ::tokio_util::task::TaskTracker,
            token: ::tokio_util::sync::CancellationToken,
            timeout: ::std::time::Duration,
            handler: H,
        ) -> Self
        where
            H: #extension_trait_ident + ::std::marker::Send + ::std::marker::Sync + 'static,
        {
            let handler = ::std::sync::Arc::new(handler);
            Self::with_handler(
                key,
                task_tracker,
                token,
                timeout,
                move |request| {
                    let handler = ::std::sync::Arc::clone(&handler);
                    async move {
                        <H as #extension_trait_ident>::handle_request(&*handler, request).await
                    }
                },
            )
        }
    }
}

fn enum_definition(
    visibility: &syn::Visibility,
    ident: &Ident,
    should_generate: bool,
    variants: impl Iterator<Item = TokenStream2>,
) -> TokenStream2 {
    if should_generate {
        quote! {
            #[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
            #visibility enum #ident {
                #(#variants,)*
            }
        }
    } else {
        quote!()
    }
}

fn protocol_base_name(protocol_ident: &Ident) -> String {
    let name = protocol_ident.to_string();
    name.strip_suffix("Protocol").unwrap_or(&name).to_string()
}

struct ProtocolMethod {
    attrs: Vec<Attribute>,
    direction: CallDirection,
    method_ident: Ident,
    /// Variant name used in both the request and response enums.
    variant_ident: Ident,
    args: Vec<MethodArg>,
    return_type: Type,
    returns_unit: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum CallDirection {
    Host,
    Extension,
}

impl ProtocolMethod {
    fn from_trait_method(mut method: syn::TraitItemFn) -> syn::Result<Self> {
        if method.sig.asyncness.is_none() {
            return Err(syn::Error::new_spanned(
                method.sig.fn_token,
                "`rpc_protocol` methods must be async",
            ));
        }

        if method.default.is_some() {
            return Err(syn::Error::new_spanned(
                method.sig.ident,
                "`rpc_protocol` methods cannot have a default body",
            ));
        }

        let direction = parse_call_direction(&mut method.attrs)?;
        let variant_ident = method_variant_ident(&method.sig.ident);

        let mut args = Vec::with_capacity(method.sig.inputs.len());
        for input in method.sig.inputs {
            args.push(MethodArg::from_fn_arg(input)?);
        }

        let return_type = return_type(method.sig.output);
        let returns_unit = is_unit(&return_type);

        Ok(Self {
            attrs: method.attrs,
            direction,
            method_ident: method.sig.ident,
            variant_ident,
            args,
            return_type,
            returns_unit,
        })
    }

    fn arg_signature(&self) -> impl Iterator<Item = TokenStream2> + '_ {
        self.args.iter().map(|arg| {
            let ident = &arg.ident;
            let ty = &arg.ty;
            quote!(#ident: #ty)
        })
    }

    fn request_variant(&self) -> TokenStream2 {
        let attrs = &self.attrs;
        let variant = &self.variant_ident;
        if self.args.is_empty() {
            quote! {
                #(#attrs)*
                #variant
            }
        } else {
            let fields = self.arg_signature();
            quote! {
                #(#attrs)*
                #variant { #(#fields),* }
            }
        }
    }

    fn response_variant(&self) -> TokenStream2 {
        let attrs = &self.attrs;
        let variant = &self.variant_ident;
        let return_type = &self.return_type;
        if self.returns_unit {
            quote! {
                #(#attrs)*
                #variant
            }
        } else {
            quote! {
                #(#attrs)*
                #variant(#return_type)
            }
        }
    }

    /// Constructs a request-enum value (for sending) or matches a request-enum
    /// pattern (for dispatch). Both forms are textually identical, so a single
    /// builder serves both call sites.
    fn request_construction(&self, request_ident: &Ident) -> TokenStream2 {
        let variant = &self.variant_ident;
        if self.args.is_empty() {
            quote!(#request_ident::#variant)
        } else {
            let fields = self.args.iter().map(|arg| &arg.ident);
            quote!(#request_ident::#variant { #(#fields),* })
        }
    }

    fn call_method(&self, request_ident: &Ident, response_ident: &Ident) -> TokenStream2 {
        let attrs = &self.attrs;
        let method_ident = &self.method_ident;
        let return_type = &self.return_type;
        let args = self.arg_signature();
        let request_expr = self.request_construction(request_ident);
        let response_match = self.response_match(response_ident);

        quote! {
            #(#attrs)*
            pub async fn #method_ident(&self, #(#args),*) -> color_eyre::Result<#return_type> {
                match self.send(#request_expr).await? {
                    #response_match
                    Err(err) => Err(color_eyre::eyre::eyre!("{err}")),
                    Ok(response) => Err(color_eyre::eyre::eyre!(
                        "unexpected response: {response:?}"
                    )),
                }
            }
        }
    }

    fn trait_method(&self) -> TokenStream2 {
        let attrs = &self.attrs;
        let method_ident = &self.method_ident;
        let return_type = &self.return_type;
        let args = self.arg_signature();

        quote! {
            #(#attrs)*
            fn #method_ident(
                &self,
                #(#args),*
            ) -> impl ::std::future::Future<
                Output = color_eyre::Result<#return_type>,
            > + ::std::marker::Send + '_;
        }
    }

    fn dispatch_arm(&self, request_ident: &Ident, response_ident: &Ident) -> TokenStream2 {
        let method_ident = &self.method_ident;
        let request_pattern = self.request_construction(request_ident);
        let variant = &self.variant_ident;
        let args = self.args.iter().map(|arg| &arg.ident);

        if self.returns_unit {
            quote! {
                #request_pattern => {
                    self.#method_ident(#(#args),*)
                        .await
                        .map(|()| #response_ident::#variant)
                        .map_err(|err| err.to_string())
                }
            }
        } else {
            quote! {
                #request_pattern => {
                    self.#method_ident(#(#args),*)
                        .await
                        .map(#response_ident::#variant)
                        .map_err(|err| err.to_string())
                }
            }
        }
    }

    fn response_match(&self, response_ident: &Ident) -> TokenStream2 {
        let variant = &self.variant_ident;
        if self.returns_unit {
            quote!(Ok(#response_ident::#variant) => Ok(()),)
        } else {
            quote!(Ok(#response_ident::#variant(value)) => Ok(value),)
        }
    }
}

struct MethodArg {
    ident: Ident,
    ty: Type,
}

impl MethodArg {
    fn from_fn_arg(arg: FnArg) -> syn::Result<Self> {
        let FnArg::Typed(arg) = arg else {
            return Err(syn::Error::new_spanned(
                arg,
                "`rpc_protocol` methods must not take `self`",
            ));
        };

        let Pat::Ident(pat_ident) = *arg.pat else {
            return Err(syn::Error::new_spanned(
                arg.pat,
                "`rpc_protocol` method arguments must be simple identifiers",
            ));
        };

        if matches!(&*arg.ty, Type::Reference(_)) {
            return Err(syn::Error::new_spanned(
                arg.ty,
                "`rpc_protocol` method arguments must be owned types; use `String` instead of `&str`",
            ));
        }

        Ok(Self {
            ident: pat_ident.ident,
            ty: *arg.ty,
        })
    }
}

fn method_variant_ident(method_ident: &Ident) -> Ident {
    format_ident!("{}", method_ident.to_string().to_case(Case::Pascal))
}

fn return_type(output: ReturnType) -> Type {
    match output {
        ReturnType::Default => syn::parse_quote!(()),
        ReturnType::Type(_, ty) => *ty,
    }
}

fn is_unit(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(tuple) if tuple.elems.is_empty())
}

fn parse_call_direction(attrs: &mut Vec<Attribute>) -> syn::Result<CallDirection> {
    let mut direction = None;
    let mut remaining_attrs = Vec::with_capacity(attrs.len());

    for attr in attrs.drain(..) {
        if attr.path().is_ident("host_call") {
            if direction.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "`rpc_protocol` methods must have exactly one call direction attribute",
                ));
            }
            direction = Some(CallDirection::Host);
        } else if attr.path().is_ident("extension_call") {
            if direction.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "`rpc_protocol` methods must have exactly one call direction attribute",
                ));
            }
            direction = Some(CallDirection::Extension);
        } else {
            remaining_attrs.push(attr);
        }
    }

    *attrs = remaining_attrs;
    direction.ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "`rpc_protocol` methods must be marked with `#[host_call]` or `#[extension_call]`",
        )
    })
}
