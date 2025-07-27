#![cfg_attr(rustchan = "nightly", feature(decl_macro))]

#[cfg(all(rustchan = "nightly", test))]
mod does_it_compile {
    use eager2::{eager, eager_macro};

    #[eager_macro]
    macro test_macro($($tokens:tt)*) {
        $($tokens)*
    }

    eager! {
        test_macro!(
            fn return_input(input: usize) -> usize {
                input
            }
        )
    }

    #[test]
    fn it_does() {
        assert_eq!(return_input(4), 4);
    }
}

#[cfg(test)]
mod visibility {
    use eager2::eager;

    mod dummy {
        use eager2::eager_macro;

        #[eager_macro]
        pub(super) macro test_macro1($($tokens:tt)*) {
            $($tokens)*
        }

        #[eager_macro]
        pub(crate) macro test_macro2($($tokens:tt)*) {
            $($tokens)*
        }

        #[eager_macro]
        pub(in crate::visibility) macro test_macro3($($tokens:tt)*) {
            $($tokens)*
        }
    }

    use dummy::{test_macro1, test_macro2, test_macro3};

    eager! {
        test_macro1!(
            fn return_input1(input: usize) -> usize {
                input
            }
        )

        test_macro2!(
            fn return_input2(input: usize) -> usize {
                input
            }
        )

        test_macro3!(
            fn return_input3(input: usize) -> usize {
                input
            }
        )
    }

    #[test]
    fn it_works() {
        assert_eq!(return_input1(1), 1);
        assert_eq!(return_input2(2), 2);
        assert_eq!(return_input3(3), 3);
    }
}

#[cfg(test)]
mod multiple_rules {
    use eager2::{eager, eager_macro};

    #[eager_macro]
    macro make_array_impl
    {
        (0) => { },
        (1) => { 0 },
        (2) => { 0, make_array_impl!(1) },
        (3) => { 0, make_array_impl!(2) },
        (4) => { 0, make_array_impl!(3) },
        (5) => { 0, make_array_impl!(4) },
        (6) => { 0, make_array_impl!(5) },
        (7) => { 0, make_array_impl!(6) },
        (8) => { 0, make_array_impl!(7) },
    }

    #[eager_macro]
    macro make_array($size:tt) {
        [make_array_impl!($size)]
    }

    #[eager_macro]
    macro array_size
    {
        ([]) => { 0 },
        ([$first:expr $(, $others:expr)* $(,)?]) => {
            1 + array_size!([$($others,)*])
        }
    }

    #[test]
    fn array_size_normal() {
        assert_eq!(array_size!([]), 0);
        assert_eq!(array_size!([0]), 1);
        assert_eq!(array_size!([0, 0]), 2);
        assert_eq!(array_size!([0, 0, 0]), 3);
        assert_eq!(array_size!([0, 0, 0, 0]), 4);
        assert_eq!(array_size!([0, 0, 0, 0, 0]), 5);
    }

    #[test]
    fn array_size_eager() {
        assert_eq!(eager! { array_size!(make_array!(0)) }, 0);
        assert_eq!(eager! { array_size!(make_array!(1)) }, 1);
        assert_eq!(eager! { array_size!(make_array!(2)) }, 2);
        assert_eq!(eager! { array_size!(make_array!(3)) }, 3);
        assert_eq!(eager! { array_size!(make_array!(4)) }, 4);
        assert_eq!(eager! { array_size!(make_array!(5)) }, 5);
        assert_eq!(eager! { array_size!(make_array!(6)) }, 6);
        assert_eq!(eager! { array_size!(make_array!(7)) }, 7);
        assert_eq!(eager! { array_size!(make_array!(8)) }, 8);
    }
}
