mod attrs;
mod doc_comment;

use std::convert::TryFrom;
use std::str::FromStr;

use config_trait::ConfigType;
use semver::Version;

use crate::rustfmt_config::attrs::filter_doc_comments;
use crate::rustfmt_config::field::attrs::ConfigOptionAttribute;
use crate::rustfmt_config::field::doc_comment::{DocComment, ParseDocCommentError};
use crate::utils::ty_to_str;

/// A configuration option of rustfmt.
#[derive(Debug, Default)]
pub struct RustfmtConfigOption<T> {
    /// A name of this config option.
    name: String,
    /// A type of this config option.
    type_name: String,
    /// A default value of this config option.
    default: T,
    /// Doc comment of this config option.
    doc_comment: DocComment,
    /// The version since this config option has become stable.
    stable_version: Option<Version>,
    /// The version since this config option has been deprecated, and its alternative.
    deprecated: Option<(Version, String)>,
}

impl<T> RustfmtConfigOption<T>
where
    T: ConfigType + Clone,
{
    pub fn is_stable(&self) -> bool {
        self.stable_version.is_some()
    }

    pub fn is_deprecated(&self) -> bool {
        self.deprecated.is_some()
    }

    pub fn stable_version(&self) -> Option<&Version> {
        self.stable_version.as_ref()
    }

    pub fn default_value(&self) -> T {
        self.default.clone()
    }
}

impl<T> TryFrom<&syn::Field> for RustfmtConfigOption<T>
where
    T: ConfigType + Clone,
{
    type Error = failure::Error;

    fn try_from(field: &syn::Field) -> Result<Self, Self::Error> {
        let name = field
            .ident
            .as_ref()
            .ok_or(TryFromRustfmtConfigOptionError::Invalid)?
            .to_string();
        let doc_comment = DocComment::from_str(&filter_doc_comments(&field.attrs))?;
        let type_name =
            ty_to_str(&field.ty).ok_or(TryFromRustfmtConfigOptionError::InvalidConfigOtpionType)?;
        let config_option_attr = ConfigOptionAttribute::try_from(field.attrs.as_slice())?;
        let default = T::from_str(config_option_attr.default_value())
            .map_err(|_| TryFromRustfmtConfigOptionError::Invalid)?;
        let deprecated = config_option_attr
            .deprecated()
            .cloned();
        let stable_version = config_option_attr.stable().cloned();

        Ok(RustfmtConfigOption {
            name,
            type_name,
            default,
            doc_comment,
            stable_version,
            deprecated,
        })
    }
}

#[derive(Debug, Fail)]
pub enum TryFromRustfmtConfigOptionError {
    #[fail(display = "Invalid config option type")]
    InvalidConfigOtpionType,
    #[fail(display = "Invalid format")]
    Invalid,
}
