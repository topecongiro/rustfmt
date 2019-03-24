extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, Ident, Item, ItemStruct, parse, MetaNameValue, Lit, Meta, Field};

#[proc_macro_attribute]
pub fn create_config(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Item);
    let config_struct = match input {
        Item::Struct(st) => st,
        _ => panic!("Expected struct"),
    };
    let config_impl = define_config_methods(&config_struct).expect("Failed to define methods on config");
    let output = quote! {
        #config_struct
        #config_impl
    };
    println!("{}", output);
    proc_macro::TokenStream::from(output)
}

// Getter, setter, default, doc, is_stable
fn define_config_methods(st: &ItemStruct) -> parse::Result<proc_macro2::TokenStream> {
    let mut methods = quote! {};
    for field in st.fields.iter() {
        let doc = define_methods_on_field(field).expect("Failed to define doc method");
        methods = quote! {
            #methods
            #doc
        };
    }

    let ident = &st.ident;

    Ok(quote! {
        impl #ident {
            #methods
        }
    })
}

fn define_methods_on_field(field: &Field) -> parse::Result<proc_macro2::TokenStream> {
    let doc_method = define_doc(field)?;
    let setter = define_setter(field)?;
    let getter = define_getter(field)?;
    Ok(quote! {
        #doc_method
        #getter
        #setter
    })
}

fn define_getter(field: &Field) -> parse::Result<proc_macro2::TokenStream> {
    let field_ident = &field.ident.as_ref().unwrap();
    let field_ty = &field.ty;
    let method_name = method_name(field, "");

    Ok(quote! {
        fn #method_name(&self) -> #field_ty {
            self.#field_ident.clone()
        }
    })
}

fn define_setter(field: &Field) -> parse::Result<proc_macro2::TokenStream> {
    let field_ident = &field.ident.as_ref().unwrap();
    let field_ty = &field.ty;
    let method_name = method_name(field, "set_");

    Ok(quote! {
        fn #method_name(&mut self, val: #field_ty) {
            self.#field_ident = val;
        }
    })
}

fn define_doc(field: &Field) -> parse::Result<proc_macro2::TokenStream> {
    let docs = field.attrs.iter().filter_map({|attr| {
        match attr.parse_meta() {
            Ok(Meta::NameValue(MetaNameValue { ref ident, lit: Lit::Str(ref doc), .. })) if ident == "doc" => {
                Some(doc.value())
            }
            _ => None,
        }
    }}).collect::<Vec<_>>().join("");
    let method_name = method_name(field, "doc_");
    Ok(quote! {
        fn #method_name(&self) -> &str {
            #docs
        }
    })
}

fn method_name(field: &Field, prefix: &str) -> Ident {
    let ident = field.ident.as_ref().unwrap();
    let s = format!("{}{}", prefix, ident);
    Ident::new(&s, ident.span())
}
