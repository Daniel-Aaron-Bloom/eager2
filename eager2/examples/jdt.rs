use eager2::{eager, eager_macro};
use std::time::Instant;

#[eager_macro]
macro_rules! parse_jdt {
    ($file:literal) => {eager! {
        struct ccase!(unstringify!($file), t:"UpperCamel") {
            __parse_jdt!(include!(concat!(env!("CARGO_MANIFEST_DIR"), "/includes/", $file, ".jtd.json")))
        }
    }};
}

#[eager_macro]
macro_rules! __parse_jdt {
    ({ "properties": {$($name:literal: $value:tt),*} }) => {eager!{
        $(ccase!(unstringify!($name), t:"snake"): __parse_jdt_field!{$value},)*
    }};
    ($($content:tt)*) => {
        compile_error!(stringify!("jdt failed to parse: " $($content)*));
    };
}

#[eager_macro]
macro_rules! __parse_jdt_field {
    ({ "type": "string" }) => { String };
    ({ "type": "timestamp" }) => { Instant };
    ({ "type": "int32" }) => { i32 };
    ({ "type": "boolean" }) => { bool };
    ($($content:tt)*) => {
        compile_error!(stringify!("jdt type failed to parse: " $($content)*));
    };
}

parse_jdt! {"some_type"}

fn main() {
    let _v = SomeType {
        created_at: Instant::now(),
        id: "hello".into(),
        karma: -100,
        is_admin: false,
    };
}
