#![allow(dead_code)]

mod test_produces_at_least_the_same {
    use eager2::eager_macro;
    /*
    Test that a declared macro will work as if it was produced with 'macro_rules'
    when not called through'eager'
    */
    #[eager_macro]
    macro_rules! test_macro{
        {1} =>{ 1 };
        (2) => ( 2 );
        [3] => [ 3 ];
        {4} => ( 4 );
        (5) => { 5 };
        [6] => [ 6 ];
        {7} => [ 7 ];
        (8) => { 8 };
        [9] => ( 9 );
    }
    #[test]
    fn test() {
        assert_eq!(1, test_macro! {1});
        assert_eq!(2, test_macro! {2});
        assert_eq!(3, test_macro! {3});
        assert_eq!(4, test_macro! {4});
        assert_eq!(5, test_macro! {5});
        assert_eq!(6, test_macro! {6});
        assert_eq!(7, test_macro! {7});
        assert_eq!(8, test_macro! {8});
        assert_eq!(9, test_macro! {9});
    }
}

mod test_eager {
    use eager2::{eager, eager_macro};
    /*
    Test that a declared macro will work as if it was produced with 'macro_rules'
    when not called through'eager'
    */
    eager! {
    #[eager_macro]
    macro_rules! test_macro {
        {1} =>{ 1 };
        (2) => ( 2 );
        [3] => [ 3 ];
        {4} => ( 4 );
        (5) => { 5 };
        [6] => [ 6 ];
        {7} => [ 7 ];
        (8) => { 8 };
        [9] => ( 9 );
    }}

    #[test]
    fn test() {
        assert_eq!(1, test_macro! {1});
        assert_eq!(2, test_macro! {2});
        assert_eq!(3, test_macro! {3});
        assert_eq!(4, test_macro! {4});
        assert_eq!(5, test_macro! {5});
        assert_eq!(6, test_macro! {6});
        assert_eq!(7, test_macro! {7});
        assert_eq!(8, test_macro! {8});
        assert_eq!(9, test_macro! {9});
    }
}

mod test_produces_eager_macro {
    use eager2::{eager, eager_macro};
    /*
    Test that a declared macro will work with eager!
    */
    #[eager_macro]
    macro_rules! test_macro{
        {1} => { + 1 };
        {2} => {eager!{1 test_macro!(1)}};
        {3} => {eager!{1 test_macro!(1) test_macro!(1)}};
        {4} => {test_macro!(2) + test_macro!(2)};
    }
    #[test]
    fn test() {
        assert_eq!(2, test_macro! {2});
        assert_eq!(3, test_macro! {3});
        assert_eq!(4, test_macro! {4});
    }
}
mod test_eager_vs_non_eager_expansion_order {
    use eager2::{eager, eager_macro};
    /*
    Test that the expanded macro has the eager versions of each rule first.
    This is required because the other way around may result in the eager
    calls not using the correct rule.
    For example, if 'mac1' below is expanded to:

    macro_rules! mac1{
        {
            $($to_encapsulate:tt)*
        }=>{
            {$($to_encapsulate)*}
        }

        <and then the eager version>
    }

    In this case eager! would not work because when it calls the macro (mac1), the pure
    rule will match the initial '@eager', which is not intended.
    */

    #[eager_macro]
    macro_rules! mac1 {
        {
            $($to_encapsulate:tt)*
        }=>{
            {$($to_encapsulate)*}
        }
    }
    macro_rules! mac2 {
        ($some:ident) => {
            eager! {
                struct $some
                mac1!{x: u32}
            }
        };
    }
    mac2! {SomeStruct}
}
mod test_attributes {
    /*
    Tests that can assign attributes to declared macros.
    */

    #[macro_use]
    mod test_mod {
        use eager2::eager_macro;

        #[eager_macro]
        #[macro_export]
        #[doc(hidden)] // Whether this gets the correct effect cannot be tested
        macro_rules! test_macro_1 {
            () => {
                1
            };
        }

        #[macro_export]
        #[doc(hidden)] // Whether this gets the correct effect cannot be tested
        #[eager_macro]
        macro_rules! test_macro_2 {
            () => {
                2
            };
        }
    }

    #[test]
    fn test() {
        assert_eq!(1, test_macro_1!());
        assert_eq!(2, test_macro_2!());
    }
}
mod test_rustdoc {
    /*
    Tests that can assign rustdoc to the declared macros.
    Whether the docs are generated correctly cannot be tested through usual
    rust testing methods. But we can at least test that the docs may be present.
    */

    #[macro_use]
    mod test_mod {
        use eager2::eager_macro;

        #[eager_macro]
        #[macro_export]
        ///
        /// Some docs
        ///
        macro_rules! test_macro_3 {
            () => {
                3
            };
        }

        #[macro_export]
        ///
        /// Some docs
        ///
        #[eager_macro]
        macro_rules! test_macro_4 {
            () => {
                4
            };
        }
    }

    #[test]
    fn test() {
        assert_eq!(3, test_macro_3!());
        assert_eq!(4, test_macro_4!());
    }
}
