use eager2::eager;

#[test]
fn test() {
    let v = eager! {vec![1, 2, 3]};
    assert_eq!(v, [1, 2, 3]);

    let v = eager! {std::vec![1, 2, 3]};
    assert_eq!(v, [1, 2, 3]);
}
