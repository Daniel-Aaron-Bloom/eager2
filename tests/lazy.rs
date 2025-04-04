#![allow(dead_code)]
mod test_lazy_block_in_eager_is_lazy {
    use eager2::eager;
    /*
    Tests that a non-eager!-enabled macro can be used inside a 'lazy!' block
    */
    macro_rules! lazy_macro {
        () => {
            1 + 1
        };
    }

    #[test]
    fn test() {
        let x = eager! {
            lazy!{
                lazy_macro!{}
            }
        };
        assert_eq!(2, x);
    }
}
mod test_lazy_block_without_eager {
    use eager2::lazy;
    /*
    Tests that can invoke `lazy!` without `eager!`.
    */
    macro_rules! lazy_macro {
        () => {
            1 + 1
        };
    }

    #[test]
    fn test() {
        let x = lazy! {
            lazy_macro!{}
        };
        assert_eq!(2, x);
    }
}
mod test_multiple_lazy_blocks {
    use eager2::eager;
    /*
    Tests that can use multiple lazy blocks in eager.
    */
    macro_rules! lazy_macro {
        () => {
            1
        };
    }

    #[test]
    fn test() {
        let x = eager! {
            lazy!{lazy_macro!{}}
            +
            lazy!{lazy_macro!{}}
            +
            lazy!{lazy_macro!{}}
        };
        assert_eq!(3, x);
    }
}
mod test_nested_lazy {
    use eager2::eager;
    /*
    Tests that a lazy blocks can be nested without having an effect.
    */
    macro_rules! lazy_macro {
        () => {
            1 + 1
        };
    }

    #[test]
    fn test() {
        let x = eager! {
            lazy!{
                lazy!{
                    lazy!{
                        lazy_macro!{}
                    }
                }
            }
        };
        assert_eq!(2, x);
    }
}
mod test_eager_in_lazy {
    use eager2::{eager, eager_macro_rules};
    /*
    Tests that an eager block can be inserted in a lazy block, and the it is eagerly expanded.
    */
    eager_macro_rules! { $eager_1
        macro_rules! eager_macro{
            ()=> {success}
        }
    }
    macro_rules! lazy_macro {
        (success) => {
            1
        };
    }

    #[test]
    fn test() {
        let x = eager! {
            lazy!{
                lazy_macro!{
                    eager!{
                        eager_macro!{}
                    }
                }
            }
        };
        assert_eq!(1, x);
    }
}
mod test_deep_nested_eager_and_lazy {
    use eager2::{eager, eager_macro_rules};
    /*
    Tests that eager and lazy blocks can be deeply nested
    */
    eager_macro_rules! { $eager_1
        macro_rules! eager_macro{
            ()=> {1}
        }
    }
    macro_rules! lazy_macro {
        // We use the rule grammar to confirm lazy/eager expansions of the test
        (1 uncallable_macro !{} 1) => {
            1
        };
    }

    #[test]
    fn test() {
        let x = eager! {
            lazy!{
                lazy_macro!{
                    eager!{
                        eager_macro!{}
                        lazy!{
                            uncallable_macro!{} // Shouldn't be called, since its in lazy mode
                            eager!{eager_macro!{}}
                        }
                    }
                }
            }
        };
        assert_eq!(1, x);
    }
}
// Same tests as above, but with the '()' block type
mod paren_test_lazy_block_in_eager_is_lazy {
    use eager2::eager;
    /*
    Tests that a non-eager!-enabled macro can be used inside a 'lazy!' block
    */
    macro_rules! lazy_macro {
        () => {
            1 + 1
        };
    }

    #[test]
    fn test() {
        let x = eager! {
            lazy!(
                lazy_macro!()
            )
        };
        assert_eq!(2, x);
    }
}
mod paren_test_lazy_block_without_eager {
    use eager2::lazy;
    /*
    Tests that can invoke `lazy!` without `eager!`.
    */
    macro_rules! lazy_macro {
        () => {
            1 + 1
        };
    }

    #[test]
    fn test() {
        let x = lazy! {
            lazy_macro!()
        };
        assert_eq!(2, x);
    }
}
mod paren_test_multiple_lazy_blocks {
    use eager2::eager;
    /*
    Tests that can use multiple lazy blocks in eager.
    */
    macro_rules! lazy_macro {
        () => {
            1
        };
    }

    #[test]
    fn test() {
        let x = eager! {
            lazy!(lazy_macro!())
            +
            lazy!(lazy_macro!())
            +
            lazy!(lazy_macro!())
        };
        assert_eq!(3, x);
    }
}
mod paren_test_nested_lazy {
    use eager2::eager;
    /*
    Tests that a lazy blocks can be nested without having an effect.
    */
    macro_rules! lazy_macro {
        () => {
            1 + 1
        };
    }

    #[test]
    fn test() {
        let x = eager! {
            lazy!(
                lazy!(
                    lazy!(
                        lazy_macro!()
                    )
                )
            )
        };
        assert_eq!(2, x);
    }
}
mod paren_test_eager_in_lazy {
    use eager2::{eager, eager_macro_rules};
    /*
    Tests that an eager block can be inserted in a lazy block, and the it is eagerly expanded.
    */
    eager_macro_rules! { $eager_1
        macro_rules! eager_macro{
            ()=> {success}
        }
    }
    macro_rules! lazy_macro {
        (success) => {
            1
        };
    }

    #[test]
    fn test() {
        let x = eager! {
            lazy!(
                lazy_macro!(
                    eager!(
                        eager_macro!()
                    )
                )
            )
        };
        assert_eq!(1, x);
    }
}
mod paren_test_deep_nested_eager_and_lazy {
    use eager2::{eager, eager_macro_rules};
    /*
    Tests that eager and lazy blocks can be deeply nested
    */
    eager_macro_rules! { $eager_1
        macro_rules! eager_macro{
            ()=> {1}
        }
    }
    macro_rules! lazy_macro {
        // We use the rule grammar to confirm lazy/eager expansions of the test
        (1 uncallable_macro !() 1) => {
            1
        };
    }

    #[test]
    fn test() {
        let x = eager! {
            lazy!(
                lazy_macro!(
                    eager!(
                        eager_macro!()
                        lazy!(
                            uncallable_macro!() // Shouldn't be called, since its in lazy mode
                            eager!(eager_macro!())
                        )
                    )
                )
            )
        };
        assert_eq!(1, x);
    }
}
