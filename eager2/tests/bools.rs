#![allow(clippy::nonminimal_bool, clippy::assertions_on_constants)]

use eager2::{eager, eager_macro};

#[eager_macro]
macro_rules! test_macro{
    {1} =>{ 1 };
}

#[test]
fn token_eq() {
    assert!(eager! {token_eq!{{a}, {a}}});
    assert!(!eager! {token_eq!{{a}, {b}}});
    assert!(eager! {token_eq!{{lazy!{1}}, {test_macro!{1}}}});
    assert!(!eager! {token_eq!{{lazy!{test_macro!{1}}}, {test_macro!{1}}}});
}

#[test]
fn eager_if() {
    let v = 1;
    eager! {eager_if![token_eq!{{a}, {a}}]{let v = v*10;}{asdf * / 4}};
    assert_eq!(v, 10);
    eager! {eager_if![token_eq!{{a}, {b}}]{asdf * / 4}{let v = v*10;}};
    assert_eq!(v, 100);
}

#[test]
fn eager_coalesce() {
    let mut v = 1;
    assert_eq!(v, 1);
    eager! {eager_coalesce!{{}, {v = 10}, {v = 100}}};
    assert_eq!(v, 10);
    eager! {eager_coalesce!{{}, {}, {}}};
    assert_eq!(v, 10);
}
