//! Normalize doc comment on a struct field.

use std::str::FromStr;

use itertools::Itertools;

use crate::rustfmt_config::attrs;

/// Parse and normalize doc comments in attributes.
pub fn parse_doc_comment(attrs: &[syn::Attribute]) -> Result<DocComment, ParseDocCommentError> {
    let doc_comment = attrs::filter_doc_comments(attrs);
    DocComment::from_str(&doc_comment)
}

/// Normalized form of doc comment.
#[derive(Debug, Default)]
pub struct DocComment {
    description: String,
    example: ConfigOptionExample,
}

impl DocComment {
    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn example(&self) -> &ConfigOptionExample {
        &self.example
    }
}

impl FromStr for DocComment {
    type Err = ParseDocCommentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines().peekable();

        // A brief desciption of the config option.
        let description = lines
            .take_while_ref(|s| !s.starts_with("#"))
            .join("\n")
            .trim()
            .to_owned();

        // ### Example
        match lines.next() {
            Some(next_line) if next_line != "### Example" => {
                return Err(ParseDocCommentError::InvalidFormat(
                    "`### Example`",
                    next_line.to_owned(),
                ));
            }
            None => return Err(ParseDocCommentError::MissingExample),
            _ => (),
        }
        if description.is_empty() {
            return Err(ParseDocCommentError::MissingDescription);
        }

        skip_until_next_header(&mut lines);

        // #### Input
        match lines.next() {
            None => return Err(ParseDocCommentError::MissingInput),
            Some(next_line) if next_line != "#### Input" => {
                return Err(ParseDocCommentError::InvalidFormat(
                    "`#### Input`",
                    next_line.to_owned(),
                ));
            }
            _ => (),
        }
        let input = take_code_block(&mut lines)?;

        skip_until_next_header(&mut lines);

        // #### Output
        match lines.next() {
            None => return Err(ParseDocCommentError::MissingOutput),
            Some(next_line) if next_line != "#### Output" => {
                return Err(ParseDocCommentError::MissingOutput);
            }
            _ => (),
        }

        skip_until_next_header(&mut lines);

        // ##### Option values
        let mut output = vec![];
        while lines.peek().is_some() {
            let option_value = match lines.next() {
                None => unreachable!(),
                Some(next_line) if next_line.starts_with("##### ") => {
                    next_line.trim_start_matches("##### ").to_owned()
                }
                Some(next_line) => {
                    return Err(ParseDocCommentError::InvalidFormat(
                        "`##### `",
                        next_line.to_owned(),
                    ));
                }
            };
            let code_block = take_code_block(&mut lines)?;
            output.push((option_value, code_block));

            skip_until_next_header(&mut lines);
        }
        if output.is_empty() {
            return Err(ParseDocCommentError::MissingOutput);
        }

        let example = ConfigOptionExample { input, output };

        Ok(DocComment {
            description,
            example,
        })
    }
}

#[derive(Debug, Default)]
pub struct ConfigOptionExample {
    input: String,
    output: Vec<(String, String)>,
}

impl ConfigOptionExample {
    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn output(&self) -> &[(String, String)] {
        &self.output
    }
}

/// `DocCommentError` represents errors encountered while parsing doc comment.
#[derive(Debug, Fail)]
pub enum ParseDocCommentError {
    #[fail(display = "Doc comment does not have a brief description of the configuration option.")]
    MissingDescription,
    #[fail(display = "Doc comment does not have an example section.")]
    MissingExample,
    #[fail(display = "Doc comment does not have an input section of example.")]
    MissingInput,
    #[fail(display = "Doc comment does not have an output section of example.")]
    MissingOutput,
    #[fail(display = "Doc comment does not have a coce block.")]
    MissingCodeBlock,
    #[fail(display = "Expected {}, found {}", _0, _1)]
    InvalidFormat(&'static str, String),
}

fn skip_until_next_header<I, S>(i: &mut I)
where
    I: Iterator<Item = S> + Clone,
    S: AsRef<str>,
{
    i.take_while_ref(|s| !s.as_ref().starts_with('#')).count();
}

fn take_code_block<I, S>(i: &mut I) -> Result<String, ParseDocCommentError>
where
    I: Iterator<Item = S> + Clone,
    S: AsRef<str> + std::fmt::Display,
{
    i.take_while_ref(|s| !s.as_ref().starts_with("```")).count();
    match i.next() {
        Some(ref s) if s.as_ref().starts_with("```") => (),
        _ => return Err(ParseDocCommentError::MissingCodeBlock),
    }
    Ok(i.by_ref()
        .take_while(|s| !s.as_ref().starts_with("```"))
        .join("\n"))
}

mod test {
    use super::parse_doc_comment;
    use quote::quote;

    #[test]
    fn parse_doc_comment_test() {
        let dummy_struct = quote! {
            struct Foo {
                /// A description.
                ///
                /// ### Example
                ///
                /// #### Input
                ///
                /// ```rust
                /// fn main() {
                ///     println!("Hello world.");
                /// }
                /// ```
                ///
                /// #### Output
                ///
                /// ##### Option value 1
                /// ```rust
                /// fn main() {
                ///     println!("Hello world.");
                /// }
                /// ```
                ///
                ///
                ///
                /// ##### Option value 1
                ///
                ///
                /// ```rust
                /// fn main() {
                ///   println!("Hello world.");
                /// }
                /// ```
                ///
                ///
                max_width: usize,
            }
        };
        let input: syn::ItemStruct = syn::parse2(dummy_struct).unwrap();
        let field = input.fields.iter().next().unwrap();
        let doc_comment = parse_doc_comment(&field.attrs).unwrap();
        assert_eq!(doc_comment.description(), "A description.");
        let input_s = r#"fn main() {
    println!("Hello world.");
}"#;
        assert_eq!(doc_comment.example().input(), input_s);
        let output_1 = r#"fn main() {
    println!("Hello world.");
}"#;
        let output_2 = r#"fn main() {
  println!("Hello world.");
}"#;
        assert_eq!(doc_comment.example().output().len(), 2);
        assert_eq!(doc_comment.example().output()[0].1, output_1);
        assert_eq!(doc_comment.example().output()[1].1, output_2);
    }
}
