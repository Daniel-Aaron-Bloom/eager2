# eager2
[![github](https://img.shields.io/badge/Daniel--Aaron--Bloom%2Feager2-8da0cb?style=for-the-badge&logo=github&label=github&labelColor=555555)](https://github.com/Daniel-Aaron-Bloom/eager2)
[![crates.io](https://img.shields.io/crates/v/eager2.svg?style=for-the-badge&color=fc8d62&logo=rust)](https://crates.io/crates/eager2)
[![docs.rs](https://img.shields.io/badge/docs.rs-eager2-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs)](https://docs.rs/eager2)
[![build status](https://img.shields.io/github/actions/workflow/status/Daniel-Aaron-Bloom/eager2/ci.yml?branch=master&style=for-the-badge)](https://github.com/Daniel-Aaron-Bloom/eager2/actions?query=branch%3Amaster)


This crate contains five core macros used to simulate eager macro expansion:

* [`eager!`]: Switches the macro-environment to eager-mode.
* [`lazy!`]: Switches the macro-environment to lazy-mode.
* [`suspend_eager!`]: Locks the macro-environment in lazy-mode.
* [`#[eager_macro]`][macro@eager_macro]: Declares an eager-enabled macro.
* [`eager_macro_rules!`]: The legacy way to declares one or more eager-enabled macro.

In addition to these primary macros, eager versions of standard library are provided (macros
only useful at runtime like `vec` are still lazy and `cfg!`, `column!`, `file!`, `line!`, and
`module_path!` are reserved for future implementation). If you wish to use the lazy versions of
the standard library, you can either insert a `lazy!{}` or `suspend_eager!{}` block around
them, or use the full path (e.g. `std::concat`). Please note the latter of which will only work
for macros from `std`, `core`, and `alloc`. All other lazy macro calls must be wrapped in `lazy`
or `suspend_eager`.

Some additional helpers are also provided:

* `ccase!`: Modifies the case of a string or ident.
* `eager_coalesce!`: Selects the first non-empty stream of tokens from a set of trees.
* `eager_if!`: Selects between two streams of tokens based on a third.
* `token_eq!`: Compares two trees of tokens for equality.
* `unstringify!`: Parses a string into a stream of tokens.

See each macro's documentation for details.

## Usage

```rust
use eager2::{eager_macro, eager};

//Declare an eager macro
#[eager_macro]
macro_rules! plus_1{
    ()=>{+ 1};
}

// Use the macro inside an eager! call to expand it eagerly
assert_eq!(4, eager!{2 plus_1!() plus_1!()});
```

## Environments

The ordinary macro environment of rust is `lazy`. This crate introduces a new environment which
is `eager`.

In the `lazy` environment the expression `a!(b!(c!()))` evaluates as follows:
* `a!` is given the tokens `b!(c!())` and produces some output
* if that output contains `b!(c!())` then `b!` is given the tokens `c!()`
* if that output contains `c!()` then `c!()` will be evaluated.

As we can see, the outer most macro is evaluated first. In the "eager" environment it's exactly
the opposite. The expression `a!(b!(c!()))` evaluates like:
* `c!()` is evaluated and expands to some tokens
* `b!` is given those expanded `c!` tokens and expands to some tokens.
* `a!` is given those expanded `b!` tokens and expands to some tokens.

An advantage of the eager approach that might not be immediately obvious is that in contrast to
the lazy approach, only the final expansion must be syntactically valid. This temporary
invalidity enables things like `concat_idents` to be trivially constructed in a much more
powerful way.

```rust
use eager2::{eager_macro, eager, unstringify};

#[eager_macro]
macro_rules! my_concat_idents {
    ($($e:ident),+ $(,)?) => { unstringify!(concat!(
        $(stringify!($e)),*
    )) };
}

// std::concat_idents can't do this
eager!{
    fn my_concat_idents!(foo, bar)() -> u32 { 23 }
}

let f = my_concat_idents!(foo, bar);

println!("{}", f());
```

One other thing to keep in mind is that the eager environment automatically includes this crate
in it's prelude, so calls within an eager environment should not need any explicit pathing.

### Switching Environments

The three macros which control the current environment are `eager!`, `lazy!`, and
`suspend_eager!`. They work as follows
```rust
// The default rust environment is lazy
eager!{

    // This environment is eager

    mod foo {

        // Still eager, `mod` and other declarations don't impact the environment

        lazy!{

            // Back to lazy

            eager!{
                // Eager again
            }

            // Back to lazy
        }

        // Back to eager

        suspend_eager!{

            // Back to lazy

            eager2::eager!{
                // Still lazy, but note that `eager!{...}` is going
                // to be a part of the final expansion, and so will
                // become eager during that evaluation.
                //
                // The use of `eager2::` prefix is because `mod foo` does not
                // import `eager`. This is something to be aware of when mixing
                // lazy and eager.
            }

        }
        // Back to eager
    }
}
```

So what's the difference between `lazy{eager!{...}}` and `suspend_eager{eager!{...}}` if the inner
part becomes eager eventually? Well, `suspend_eager!` forces eager evaluation to finish before
continuing, which means a syntactically valid expansion must occur. The difference can be seen
in this example:
```rust
use eager2::{eager, eager_macro};

#[eager_macro]
macro_rules! fn_body{
    ()=>{ foo() {} };
}

eager!{
    // This is legal
    fn lazy!{eager!{ fn_body!{} }}

    // This is not legal
    // error: missing parameters for function definition
    //fn suspend_eager!{eager!{ fn_body!{} }}
}
```

## Exporting Macros

When exporting a macro which calls into this crate, some care must be taken to deal with the
correct paths. The recommended convention is to add:
```rust
#[doc(hidden)]
pub extern crate eager2;
```

to the root of your crate and to utilize the `$crate::eager2::` prefix to call any macros when
in a lazy environment (eager environments should not require any prefix, but will identify macro
items with the `$crate::eager2::` as belonging to this crate).

## Documentation Convention

To make it clearly visible that a given macro is `eager!`-enabled, its short rustdoc description
must start with a pair of brackets, within which a link to the official `eager!` macro
documentation must be provided. The link's visible text must be 'eager!' and the brackets must
not be part of the link.

## Limitations

`eager!` is implemented using recursive macros, so the compiler's default macro recursion limit
can be exceeded. So occasionally you may have to use `#![recursion_limit="256"]` or higher. You
can also mitigate this by adding `suspend_eager!{eager!{...}}` blocks around code which calls
custom `eager!`-enabled macros (prelude macros are optimized and will not benefit from this).
This can impact the legality of your output, and so should only be done where it will not cause
breakage.

Debugging an macros can be quite difficult, eager macros especially so. [`compile_error!`] can
sometimes be helpful in this regard. This crate also has `trace_macros` feature which only
operates on `nightly`, and can be quite verbose. Contributions to improve the debugging
experience are very welcome.

Only `eager!`-enabled macros can be eagerly expanded, so existing macros do not gain much.
The `lazy!` block alleviates this a bit, by allowing the use of existing macros in it, while
eager expansion can be done around them. Luckily, `eager!`-enabling an existing macro should
not be too much trouble using `#[eager_macro]`.

## License

Licensed under MIT license ([LICENSE](LICENSE) or https://opensource.org/licenses/MIT)

[`eager!`]: https://docs.rs/eager2/latest/eager2/macro.eager.html "macro eager2::eager"
[`lazy!`]: https://docs.rs/eager2/latest/eager2/macro.lazy.html "macro eager2::lazy"
[`suspend_eager!`]: https://docs.rs/eager2/latest/eager2/macro.suspend_eager.html "macro eager2::lazy"
[macro@eager_macro]: https://docs.rs/eager2/latest/eager2/attr.eager_macro.html "attr eager2::eager_macro"
[`eager_macro_rules!`]: https://docs.rs/eager2/latest/eager2/macro.eager_macro_rules.html "macro eager2::eager_macro_rules"
