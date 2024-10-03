use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Field};

#[proc_macro_derive(Build, attributes(state))]
pub fn derive_build(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;

    let generics = input
        .generics
        .type_params()
        .map(|x| &x.ident)
        .collect::<Vec<_>>();

    let syn::Data::Struct(data) = input.data else {
        return quote! { ::core::compile_error!("Build can only be derived for structs"); }.into();
    };

    fn is_state(field: &Field) -> bool {
        field
            .attrs
            .iter()
            .any(|x| x.path().get_ident().is_some_and(|x| x == "state"))
    }

    let bounds = data
        .fields
        .iter()
        .filter(|x| !is_state(x))
        .map(|Field { ty, .. }| quote! { #ty: ::academy_di::Build<__Provider> })
        .collect::<Vec<_>>();

    let build_expr = match data.fields {
        syn::Fields::Named(fields) => {
            let fields = fields
                .named
                .iter()
                .map(|f @ Field { ident, .. }| {
                    if is_state(f) {
                        quote! { #ident: ::core::default::Default::default() }
                    } else {
                        quote! { #ident: ::academy_di::Build::build(provider) }
                    }
                })
                .collect::<Vec<_>>();
            quote! { Self { #(#fields),* } }
        }
        syn::Fields::Unnamed(fields) => {
            let fields = fields
                .unnamed
                .iter()
                .map(|f| {
                    if is_state(f) {
                        quote! { ::core::default::Default::default() }
                    } else {
                        quote! { ::academy_di::Build::build(provider) }
                    }
                })
                .collect::<Vec<_>>();
            quote! { Self( #(#fields),* ) }
        }
        syn::Fields::Unit => quote! { Self },
    };

    quote! {
        impl<__Provider, #(#generics),*> ::academy_di::Build<__Provider> for #ident<#(#generics),*>
        where
            Self: ::core::clone::Clone + 'static,
            __Provider: ::academy_di::Provider,
            #(#bounds),*
        {
            fn build(provider: &mut __Provider) -> Self {
                if let ::core::option::Option::Some(cached) = ::academy_di::Provider::get(provider) {
                    cached
                } else {
                    #build_expr
                }
            }
        }
    }
    .into()
}
