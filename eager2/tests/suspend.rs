use eager2::{eager, eager_macro_rules};

eager_macro_rules! {
    macro_rules! fn_body{
        ()=>{ foo() {} };
    }
}

#[test]
fn build_fn() {
    eager! {
        #[doc = "just a function"]
        suspend_eager!{eager!{fn fn_body!()}}
    }
    foo();
}
