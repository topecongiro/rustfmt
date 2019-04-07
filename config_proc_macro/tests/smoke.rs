pub mod config {
    pub trait ConfigType: Sized {
        fn doc_hint() -> String;
    }
}

#[allow(dead_code)]
#[allow(unused_imports)]
mod tests {
    use config_proc_macro::{config_type, define_config};

    #[config_type]
    enum Bar {
        Foo,
        Bar,
        #[doc_hint = "foo_bar"]
        FooBar,
        FooFoo(i32),
    }

    enum Foo {}

    #[define_config]
    pub struct Config {
        /// A width of indent.
        ///
        /// ### Example
        ///
        /// ```rust
        /// ```
        #[config(stable = "1.0", default(100))]
        indent_width: usize,
        /// Skip formatting an entire directory.
        #[config(deprecated = "2.0", msg = "Use `ignore = [.] instead.")]
        disable_all_formattings: bool,
        /// How do we indent exprressions and items.
        #[config(stable = "1.0", default(Foo::Var))]
        indent_style: Foo,
    }
}
