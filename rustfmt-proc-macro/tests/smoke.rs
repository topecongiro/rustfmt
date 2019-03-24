#[rustfmt_proc_macro::create_config]
#[derive(Debug)]
struct DummyConfig {
    /// This is doc comment.
    /// Multiline doc comment.
    dummy: u32,
}


#[test]
fn smoke_test() {
    let mut x = DummyConfig{
        dummy: 0
    };
    println!("{}", x.doc_dummy());
    x.set_dummy(1);
    assert_eq!(1, *x.dummy());
}
