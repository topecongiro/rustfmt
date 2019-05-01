//! Filter usable attributes on fields of `Config` struct.

use std::convert::TryFrom;
use std::str::FromStr;

use semver::Version;

/// Normalized form of `config_option` attribute.
#[derive(Debug)]
pub struct ConfigOptionAttribute {
    default_value: String,
    stable: Option<Version>,
    deprecated: Option<(Version, String)>,
}

impl ConfigOptionAttribute {
    pub fn default_value(&self) -> &str {
        self.default_value.as_str()
    }

    pub fn stable(&self) -> Option<&Version> {
        self.stable.as_ref()
    }

    pub fn deprecated(&self) -> Option<&(Version, String)> {
        self.deprecated.as_ref()
    }
}

#[derive(Debug, Fail)]
pub enum TryFromConfigOptionAttributeError {
    /// `#[config_option]` is not available.
    #[fail(display = "#[config_option] not found.")]
    NotFound,
    /// `#[config_option]` is found but written in an unexpected format.
    #[fail(display = "Invalid #[config_option].")]
    Invalid,
}

impl TryFrom<&[syn::Attribute]> for ConfigOptionAttribute {
    type Error = TryFromConfigOptionAttributeError;

    fn try_from(attrs: &[syn::Attribute]) -> Result<Self, Self::Error> {
        let config_option = attrs
            .iter()
            .filter_map(syn::Attribute::interpret_meta)
            .find(|meta| match meta {
                syn::Meta::List(ref meta_list) if meta_list.ident == "config_option" => true,
                _ => false,
            });
        let config_option = match config_option {
            Some(config_option) => config_option,
            None => return Err(TryFromConfigOptionAttributeError::NotFound),
        };

        match config_option {
            syn::Meta::List(ref meta_list) => {
                let default_value = find_map_meta_list(meta_list, extract_default)
                    .ok_or(TryFromConfigOptionAttributeError::Invalid)?;
                let stable = find_map_meta_list(meta_list, extract_stable);
                let deprecated = find_map_meta_list(meta_list, extract_deprecated);
                if stable.is_some() && deprecated.is_some() {
                    return Err(TryFromConfigOptionAttributeError::Invalid);
                }
                Ok(ConfigOptionAttribute {
                    default_value,
                    stable,
                    deprecated,
                })
            }
            _ => return Err(TryFromConfigOptionAttributeError::Invalid),
        }
    }
}

fn find_map_meta_list<F, T>(meta_list: &syn::MetaList, f: F) -> Option<T>
where
    F: Fn(&syn::NestedMeta) -> Option<T>,
{
    meta_list.nested.iter().find_map(f)
}

fn extract_str_value(meta: &syn::NestedMeta, name: &str) -> Option<String> {
    match meta {
        syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
            ident,
            lit: syn::Lit::Str(lit_str),
            ..
        })) if ident == name => Some(lit_str.value()),
        _ => None,
    }
}

fn extract_default(meta: &syn::NestedMeta) -> Option<String> {
    extract_str_value(meta, "default")
}

fn extract_stable(meta: &syn::NestedMeta) -> Option<Version> {
    extract_str_value(meta, "stable").and_then(|s| Version::from_str(s.as_ref()).ok())
}

fn extract_version(meta: &syn::NestedMeta) -> Option<Version> {
    extract_str_value(meta, "version").and_then(|s| Version::from_str(s.as_ref()).ok())
}

fn extract_alternative(meta: &syn::NestedMeta) -> Option<String> {
    extract_str_value(meta, "alternative")
}

fn extract_deprecated(meta: &syn::NestedMeta) -> Option<(Version, String)> {
    match meta {
        syn::NestedMeta::Meta(syn::Meta::List(ref meta_list))
            if meta_list.ident == "deprecated" =>
        {
            let version = find_map_meta_list(meta_list, extract_version)?;
            let alternative = find_map_meta_list(meta_list, extract_alternative)?;
            Some((version, alternative))
        }
        _ => None,
    }
}

mod test {
    use quote::quote;
    use syn::parse2;

    use super::*;

    #[test]
    fn extract_config_option_test() {
        let tokens = quote! {
            struct Foo {
                #[config_option(stable = "1.0.0",
                                default = "100",
                                deprecated(version = "1.2.0", alternative = "Use the other config option."))]
                field: u32,
            }
        };
        let st: syn::ItemStruct = parse2(tokens).expect("Failed to parse");
        let field = st.fields.iter().next().expect("No field");
        let config_option_attr = ConfigOptionAttribute::try_from(field.attrs.as_slice()).unwrap();
        println!("{:?}",config_option_attr);

        assert_eq!(config_option_attr.stable, Version::from_str("1.0.0").ok());
        assert_eq!(config_option_attr.default_value, "100");
        assert_eq!(config_option_attr.deprecated.unwrap(),
                   (Version::from_str("1.2.0").unwrap(), "Use the other config option.".to_owned()));
    }
}
