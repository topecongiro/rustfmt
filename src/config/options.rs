use std::collections::HashSet;
use std::path::{Path, PathBuf};

use atty;
use config_proc_macro::config_type;

use crate::config::lists::*;
use crate::config::{Config, FileName};

#[config_type]
pub enum NewlineStyle {
    Auto,    // Auto-detect based on the raw source input
    Windows, // \r\n
    Unix,    // \n
    Native,  // \r\n in Windows, \n on other platforms
}

impl NewlineStyle {
    fn auto_detect(raw_input_text: &str) -> NewlineStyle {
        if let Some(pos) = raw_input_text.find('\n') {
            let pos = pos.saturating_sub(1);
            if let Some('\r') = raw_input_text.chars().nth(pos) {
                NewlineStyle::Windows
            } else {
                NewlineStyle::Unix
            }
        } else {
            NewlineStyle::Native
        }
    }

    fn native() -> NewlineStyle {
        if cfg!(windows) {
            NewlineStyle::Windows
        } else {
            NewlineStyle::Unix
        }
    }

    /// Apply this newline style to the formatted text. When the style is set
    /// to `Auto`, the `raw_input_text` is used to detect the existing line
    /// endings.
    ///
    /// If the style is set to `Auto` and `raw_input_text` contains no
    /// newlines, the `Native` style will be used.
    pub(crate) fn apply(self, formatted_text: &mut String, raw_input_text: &str) {
        use crate::NewlineStyle::*;
        let mut style = self;
        if style == Auto {
            style = Self::auto_detect(raw_input_text);
        }
        if style == Native {
            style = Self::native();
        }
        match style {
            Windows => {
                let mut transformed = String::with_capacity(2 * formatted_text.capacity());
                for c in formatted_text.chars() {
                    match c {
                        '\n' => transformed.push_str("\r\n"),
                        '\r' => continue,
                        c => transformed.push(c),
                    }
                }
                *formatted_text = transformed;
            }
            Unix => return,
            Native => unreachable!("NewlineStyle::Native"),
            Auto => unreachable!("NewlineStyle::Auto"),
        }
    }
}

#[config_type]
pub enum BraceStyle {
    AlwaysNextLine,
    PreferSameLine,
    // Prefer same line except where there is a where-clause, in which case force
    // the brace to the next line.
    SameLineWhere,
}

#[config_type]
pub enum ControlBraceStyle {
    // K&R style, Rust community default
    AlwaysSameLine,
    // Stroustrup style
    ClosingNextLine,
    // Allman style
    AlwaysNextLine,
}

#[config_type]
pub enum IndentStyle {
    // First line on the same line as the opening brace, all lines aligned with
    // the first line.
    Visual,
    // First line is on a new line and all lines align with block indent.
    Block,
}

#[config_type]
pub enum Density {
    // Fit as much on one line as possible.
    Compressed,
    // Use more lines.
    Tall,
    // Place every item on a separate line.
    Vertical,
}

#[config_type]
pub enum TypeDensity {
    // No spaces around "=" and "+"
    Compressed,
    // Spaces around " = " and " + "
    Wide,
}

#[config_type]
pub enum Heuristics {
    // Turn off any heuristics
    Off,
    // Turn on max heuristics
    Max,
    // Use Rustfmt's defaults
    Default,
}

impl Density {
    pub fn to_list_tactic(self, len: usize) -> ListTactic {
        match self {
            Density::Compressed => ListTactic::Mixed,
            Density::Tall => ListTactic::HorizontalVertical,
            Density::Vertical if len == 1 => ListTactic::Horizontal,
            Density::Vertical => ListTactic::Vertical,
        }
    }
}

#[config_type]
pub enum ReportTactic {
    Always,
    Unnumbered,
    Never,
}

// What Rustfmt should emit. Mostly corresponds to the `--emit` command line
// option.
#[config_type]
pub enum EmitMode {
    // Emits to files.
    Files,
    // Writes the output to stdout.
    Stdout,
    // Displays how much of the input file was processed
    Coverage,
    // Unfancy stdout
    Checkstyle,
    // Output the changed lines (for internal value only)
    ModifiedLines,
    // Checks if a diff can be generated. If so, rustfmt outputs a diff and quits with exit code 1.
    // This option is designed to be run in CI where a non-zero exit signifies non-standard code
    // formatting. Used for `--check`.
    Diff,
}

// Client-preference for coloured output.
#[config_type]
pub enum Color {
    // Always use color, whether it is a piped or terminal output
    Always,
    // Never use color
    Never,
    // Automatically use color, if supported by terminal
    Auto,
}

#[config_type]
pub enum Version {
    // 1.x.y
    One,
    // 2.x.y
    Two,
}

impl Color {
    /// Whether we should use a coloured terminal.
    pub fn use_colored_tty(self) -> bool {
        match self {
            Color::Always => true,
            Color::Never => false,
            Color::Auto => atty::is(atty::Stream::Stdout),
        }
    }
}

// How chatty should Rustfmt be?
#[config_type]
pub enum Verbosity {
    // Emit more.
    Verbose,
    Normal,
    // Emit as little as possible.
    Quiet,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct WidthHeuristics {
    // Maximum width of the args of a function call before falling back
    // to vertical formatting.
    pub fn_call_width: usize,
    // Maximum width of the args of a function-like attributes before falling
    // back to vertical formatting.
    pub attr_fn_like_width: usize,
    // Maximum width in the body of a struct lit before falling back to
    // vertical formatting.
    pub struct_lit_width: usize,
    // Maximum width in the body of a struct variant before falling back
    // to vertical formatting.
    pub struct_variant_width: usize,
    // Maximum width of an array literal before falling back to vertical
    // formatting.
    pub array_width: usize,
    // Maximum length of a chain to fit on a single line.
    pub chain_width: usize,
    // Maximum line length for single line if-else expressions. A value
    // of zero means always break if-else expressions.
    pub single_line_if_else_max_width: usize,
}

impl WidthHeuristics {
    // Using this WidthHeuristics means we ignore heuristics.
    pub fn null() -> WidthHeuristics {
        WidthHeuristics {
            fn_call_width: usize::max_value(),
            attr_fn_like_width: usize::max_value(),
            struct_lit_width: 0,
            struct_variant_width: 0,
            array_width: usize::max_value(),
            chain_width: usize::max_value(),
            single_line_if_else_max_width: 0,
        }
    }

    pub fn set(max_width: usize) -> WidthHeuristics {
        WidthHeuristics {
            fn_call_width: max_width,
            attr_fn_like_width: max_width,
            struct_lit_width: max_width,
            struct_variant_width: max_width,
            array_width: max_width,
            chain_width: max_width,
            single_line_if_else_max_width: max_width,
        }
    }

    // scale the default WidthHeuristics according to max_width
    pub fn scaled(max_width: usize) -> WidthHeuristics {
        const DEFAULT_MAX_WIDTH: usize = 100;
        let max_width_ratio = if max_width > DEFAULT_MAX_WIDTH {
            let ratio = max_width as f32 / DEFAULT_MAX_WIDTH as f32;
            // round to the closest 0.1
            (ratio * 10.0).round() / 10.0
        } else {
            1.0
        };
        WidthHeuristics {
            fn_call_width: (60.0 * max_width_ratio).round() as usize,
            attr_fn_like_width: (70.0 * max_width_ratio).round() as usize,
            struct_lit_width: (18.0 * max_width_ratio).round() as usize,
            struct_variant_width: (35.0 * max_width_ratio).round() as usize,
            array_width: (60.0 * max_width_ratio).round() as usize,
            chain_width: (60.0 * max_width_ratio).round() as usize,
            single_line_if_else_max_width: (50.0 * max_width_ratio).round() as usize,
        }
    }
}

impl ::std::str::FromStr for WidthHeuristics {
    type Err = &'static str;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Err("WidthHeuristics is not parsable")
    }
}

impl Default for EmitMode {
    fn default() -> EmitMode {
        EmitMode::Files
    }
}

/// A set of directories, files and modules that rustfmt should ignore.
#[derive(Default, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct IgnoreList(HashSet<PathBuf>);

impl IgnoreList {
    pub fn add_prefix(&mut self, dir: &Path) {
        self.0 = self
            .0
            .iter()
            .map(|s| {
                if s.has_root() {
                    s.clone()
                } else {
                    let mut path = PathBuf::from(dir);
                    path.push(s);
                    path
                }
            })
            .collect();
    }

    fn skip_file_inner(&self, file: &Path) -> bool {
        self.0.iter().any(|path| file.starts_with(path))
    }

    pub fn skip_file(&self, file: &FileName) -> bool {
        if let FileName::Real(ref path) = file {
            self.skip_file_inner(path)
        } else {
            false
        }
    }
}

impl ::std::str::FromStr for IgnoreList {
    type Err = &'static str;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Err("IgnoreList is not parsable")
    }
}

/// Maps client-supplied options to Rustfmt's internals, mostly overriding
/// values in a config with values from the command line.
pub trait CliOptions {
    fn apply_to(self, config: &mut Config);
    fn config_path(&self) -> Option<&Path>;
}

// The edition of the compiler (RFC 2052)
#[config_type]
pub enum Edition {
    #[value = "2015"]
    #[doc_hint = "2015"]
    Edition2015,
    #[value = "2018"]
    #[doc_hint = "2018"]
    Edition2018,
}

impl Default for Edition {
    fn default() -> Edition {
        Edition::Edition2015
    }
}

impl Edition {
    pub(crate) fn to_libsyntax_pos_edition(self) -> syntax_pos::edition::Edition {
        match self {
            Edition::Edition2015 => syntax_pos::edition::Edition::Edition2015,
            Edition::Edition2018 => syntax_pos::edition::Edition::Edition2018,
        }
    }
}

#[test]
fn test_newline_style_auto_detect() {
    let lf = "One\nTwo\nThree";
    let crlf = "One\r\nTwo\r\nThree";
    let none = "One Two Three";

    assert_eq!(NewlineStyle::Unix, NewlineStyle::auto_detect(lf));
    assert_eq!(NewlineStyle::Windows, NewlineStyle::auto_detect(crlf));
    assert_eq!(NewlineStyle::Native, NewlineStyle::auto_detect(none));
}

#[test]
fn test_newline_style_auto_apply() {
    let auto = NewlineStyle::Auto;

    let formatted_text = "One\nTwo\nThree";
    let raw_input_text = "One\nTwo\nThree";

    let mut out = String::from(formatted_text);
    auto.apply(&mut out, raw_input_text);
    assert_eq!("One\nTwo\nThree", &out, "auto should detect 'lf'");

    let formatted_text = "One\nTwo\nThree";
    let raw_input_text = "One\r\nTwo\r\nThree";

    let mut out = String::from(formatted_text);
    auto.apply(&mut out, raw_input_text);
    assert_eq!("One\r\nTwo\r\nThree", &out, "auto should detect 'crlf'");

    #[cfg(not(windows))]
    {
        let formatted_text = "One\nTwo\nThree";
        let raw_input_text = "One Two Three";

        let mut out = String::from(formatted_text);
        auto.apply(&mut out, raw_input_text);
        assert_eq!(
            "One\nTwo\nThree", &out,
            "auto-native-unix should detect 'lf'"
        );
    }

    #[cfg(windows)]
    {
        let formatted_text = "One\nTwo\nThree";
        let raw_input_text = "One Two Three";

        let mut out = String::from(formatted_text);
        auto.apply(&mut out, raw_input_text);
        assert_eq!(
            "One\r\nTwo\r\nThree", &out,
            "auto-native-windows should detect 'crlf'"
        );
    }
}
