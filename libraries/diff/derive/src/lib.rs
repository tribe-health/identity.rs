mod impls;
mod internal;

#[macro_use]
extern crate quote;

#[macro_use]
extern crate syn;

extern crate proc_macro;
extern crate proc_macro2;

use proc_macro2::TokenStream;
use syn::DeriveInput;

#[proc_macro_derive(Diff, attributes(patch))]
pub fn patch_diff_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input: DeriveInput = parse_macro_input!(input as DeriveInput);

  impls::expand_derive_diff(&input)
    .unwrap_or_else(to_compile_errors)
    .into()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> TokenStream {
  let errors = errors.iter().map(syn::Error::to_compile_error);
  quote!(#(#errors)*)
}
