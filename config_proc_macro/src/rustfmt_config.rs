mod attrs;
mod define_struct;
mod field;

use proc_macro2::TokenStream;

use define_struct::define_rustfmt_config_on_struct;

pub fn define_rustfmt_config(input: &syn::Item) -> TokenStream {
    match input {
        syn::Item::Struct(st) => define_rustfmt_config_on_struct(st),
        _ => panic!("Expected struct"),
    }
    .unwrap()
}
