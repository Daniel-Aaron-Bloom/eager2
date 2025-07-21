#![cfg_attr(rustfmt, rustfmt_skip)]
use eager2::eager;

#[test]
fn test_1() {
    const A: u32 = column!();
    const B: u32 = eager2::column!();
    assert_eq!(A, B);
}

#[test]
fn test_2() {
    const A: u32 = std::
column!();
    const B: u32 = eager2::
column!();
    assert_eq!(A, B);
}

#[test]
fn test_3() {
    const A: u32 =          column!();
    const B: u32 = eager! { column!() };
    assert_eq!(A, B);
}

#[test]
fn test_4() {
    const A: u32 = eager! {
                   eager2::column!()
    };
    const B: u32 = column!();
    assert_eq!(A, B);
}

#[test]
fn test_5() {
    const A: u32 = eager! {
                   eager2::column!()
    };
    const B: u32 = std::column!();
    assert_eq!(A, B);
}

#[test]
fn test_6() {
    const A: u32 = eager! {
                   column!()
    };
    const B: u32 = std::column!();
    assert_eq!(A, B);
}

#[test]
fn test_7() {
    const A: u32 = eager! {
                   eager2::
column!()
    };
    const B: u32 = std::column!();
    assert_eq!(A, B);
}
