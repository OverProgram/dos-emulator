extern crate syn;
extern crate quote;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput, Data};

#[proc_macro_derive(FlagBits)]
pub fn enum_flags_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.clone().ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut flag_to_loc_tokens = Vec::new();

    match input.clone().data {
        Data::Enum(enum_token) => {
            let mut count = 0;
            for variant in enum_token.variants {
                let loc = match variant.discriminant {
                    Some((_, expr)) => expr.into_token_stream(),
                    None => quote! { #count }
                };
                let variant_name = variant.ident;
                flag_to_loc_tokens.push(quote! {
                    #variant_name => #loc
                });
                count += 1
            }
        }
        _ => panic!("FlagBits derive macro only works with enums")
    }

    let expanded = quote! {
        impl #impl_generics enumflags::FlagBits for #name #ty_generics #where_clause {
            type Container = usize;

            fn to_loc(&self) -> usize {
                match self {
                    #(#flag_to_loc_tokens),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
