mod attrs;
mod item_enum;
mod item_struct;

use proc_macro2::TokenStream;

use item_enum::define_config_type_on_enum;
use item_struct::define_config_type_on_struct;

/// Defines `config_type` on enum or struct.
// FIXME: Implement this on struct.
pub fn define_config_type(input: &syn::Item) -> TokenStream {
    match input {
        syn::Item::Struct(st) => define_config_type_on_struct(st),
        syn::Item::Enum(en) => define_config_type_on_enum(en),
        _ => panic!("Expected enum or struct"),
    }
    .unwrap()
}
