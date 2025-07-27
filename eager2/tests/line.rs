#![cfg_attr(rustfmt, rustfmt_skip)]
use eager2::eager;

#[test]
fn test_1() {
    const A: u32 = line!();
    const B: u32 = eager2::line!();
    assert_eq!(A + 1, B);
}


#[test]
fn test_2() {
    const A: u32 = std::
line!();
    const B: u32 = eager2::
line!();
    assert_eq!(A + 2, B);
}

#[test]
fn test_3() {
    const A: u32 =          line!();
    const B: u32 = eager! {
        line!()
    };
    assert_eq!(A + 2, B);
}

#[test]
fn test_4() {
    const A: u32 = eager! {
                   eager2::
line!()
    };
    const B: u32 = std::line!();
    assert_eq!(A+3, B);
}
