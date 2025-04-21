#![allow(dead_code)]

mod test_noop_lazy {
    use eager2::lazy;

    #[test]
    fn test() {
        assert_eq!("Success", lazy! {stringify!(Success)});
    }
}

mod test_noop_eager {
    use eager2::eager;

    #[test]
    fn test() {
        assert_eq!("Success", eager! {stringify!(Success)});
    }
}

mod test_eager {
    use eager2::{eager, eager_macro_rules};

    eager_macro_rules! {
        macro_rules! test_macro{
            {1} =>{ Hello };
            (2) => ( Hola );
            [3] => [ Bonjour ];
            {4} => ( Hallo );
            (5) => { Ciao };
            [6] => [ 你好 ];
            {7} => [ こんにちは ];
            (8) => { Olá };
            [9] => ( 안녕하세요 );
        }
    }

    #[test]
    fn test() {
        assert_eq!("Hello", eager! {stringify!(test_macro!(1))});
        assert_eq!("Hola", eager! {stringify!(test_macro!(2))});
        assert_eq!("Bonjour", eager! {stringify!(test_macro!(3))});
        assert_eq!("Hallo", eager! {stringify!(test_macro!(4))});
        assert_eq!("Ciao", eager! {stringify!(test_macro!(5))});
        assert_eq!("你好", eager! {stringify!(test_macro!(6))});
        assert_eq!("こんにちは", eager! {stringify!(test_macro!(7))});
        assert_eq!("Olá", eager! {stringify!(test_macro!(8))});
        assert_eq!("안녕하세요", eager! {stringify!(test_macro!(9))});
    }
}

mod test_unstringify {
    use eager2::eager;

    #[test]
    fn test() {
        eager! {unstringify!("let v = 10;")}
        assert_eq!(v, 10);
    }
}
