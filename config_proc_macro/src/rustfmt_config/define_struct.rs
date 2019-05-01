use proc_macro2::TokenStream;
use quote::quote;

/// Define rustfmt `Config` struct.
pub fn define_rustfmt_config_on_struct(st: &syn::ItemStruct) -> syn::Result<TokenStream> {
    let syn::ItemStruct {
        vis,
        ident,
        generics,
        fields,
        ..
    } = st;

    let result = quote! {
        #vis struct #ident #generics {
            #fields
        }
    };

    Ok(result)
}

mod test {
    use crate::rustfmt_config::define_struct::define_rustfmt_config_on_struct;
    use quote::quote;

    #[test]
    fn smoke_test() {
        let dummy_struct = quote! {
            /// This is a doc comment.
            struct Foo {
                x: i32,
            }
        };
        let input = syn::parse2(dummy_struct).unwrap();
        define_rustfmt_config_on_struct(&input).unwrap();
    }
}
