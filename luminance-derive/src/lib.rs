extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MyMacro)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = parse_macro_input!(input);
    let struct_name = derive_input.ident;

    let expanded = quote! {
      struct #struct_name {
      }
    };

    TokenStream::from(expanded)
}
