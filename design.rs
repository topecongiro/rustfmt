#![feature(custom_attribute)]

enum Foo {
    Bar,
}

#[define_config]
pub struct Config {
    /// A width of indent.
    ///
    /// ### Example
    ///
    /// ```rust
    /// ```
    #[rustfmt::config(stable = "1.0", default(100))]
    indent_width: usize,
    /// Skip formatting an entire directory.
    #[rustfmt::config(deprecated = "2.0", msg = "Use `ignore = [.] instead.")]
    disable_all_formattings: bool,
    /// How do we indent exprressions and items. 
    #[rustfmt::config(stable = "1.0", default(Foo::Var))]
    indent_style: Foo,
}

fn main() {}
