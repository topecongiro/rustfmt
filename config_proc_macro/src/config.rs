use proc_macro2::TokenStream;
use quote::quote;
use syn::{self, visit::Visit};

pub fn define_config(input: &syn::Item) -> TokenStream {
    let st = match input {
        syn::Item::Struct(st) => st,
        _ => panic!("Expected struct"),
    };

    match define_config_on_struct(st) {
        Ok(ts) => ts,
        Err(e) => panic!(e),
    }
}

fn define_config_on_struct(st: &syn::ItemStruct) -> syn::Result<TokenStream> {
    let mut visitor = ConfigVisitor{};
    visitor.visit_item_struct(st);
    Ok(quote!{#st})
}

struct ConfigVisitor {}

impl<'ast> Visit<'ast> for ConfigVisitor {
    fn visit_field(&mut self, f: &'ast syn::Field) {
        eprintln!("{}", quote!(#f));
    }
}
