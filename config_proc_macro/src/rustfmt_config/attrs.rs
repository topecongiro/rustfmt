use itertools::join;

/// Extract doc comments out of attributes. Doc comments are returned
/// as a paragraph in groups. A leading space is removed from each line.
pub fn filter_doc_comments(attrs: &[syn::Attribute]) -> String {
    join(attrs.iter().filter_map(attr_to_doc_text), "\n")
}

/// Returns the documentation text if the given attribute is a `#[doc]` attribute.
fn attr_to_doc_text(attr: &syn::Attribute) -> Option<String> {
    match attr.interpret_meta()? {
        syn::Meta::NameValue(ref meta_name_value) if meta_name_value.ident == "doc" => {
            match meta_name_value.lit {
                syn::Lit::Str(ref lit_str) => {
                    let s = lit_str.value();
                    Some(if s.starts_with(' ') {
                        (&s[1..]).to_owned()
                    } else {
                        s
                    })
                }
                _ => None,
            }
        }
        _ => None,
    }
}

mod test {
    use super::filter_doc_comments;
    use quote::quote;

    #[test]
    fn filter_doc_comments_test() {
        let dummy_struct = quote! {
            /// first line.
            ///
            /// third line.
            #[hello]
            /// second group.
            struct Foo {}
        };
        let input: syn::ItemStruct = syn::parse2(dummy_struct).unwrap();
        let doc_comments = filter_doc_comments(&input.attrs);
        assert_eq!(doc_comments, "first line.\n\nthird line.\nsecond group.");
    }
}
