//!
//! This crate contains five core macros used to simulate eager macro expansion:
//!
//! * [`eager!`]: Switches the macro-environment to eager-mode.
//! * [`lazy!`]: Switches the macro-environment to lazy-mode.
//! * [`suspend_eager!`]: Locks the macro-environment in lazy-mode.
//! * [`#[eager_macro]`][macro@eager_macro]: Declares an eager-enabled macro.
//! * [`eager_macro_rules!`]: The legacy way to declares one or more eager-enabled macro.
//!
//! In addition to these primary macros, eager versions of standard library are provided (macros
//! only useful at runtime like `vec` are still lazy and `cfg!`, `column!`, `file!`, `line!`, and
//! `module_path!` are reserved for future implementation). If you wish to use the lazy versions of
//! the standard library, you can either insert a `lazy!{}` or `suspend_eager!{}` block around
//! them, or use the full path (e.g. `std::concat`). Please note the latter of which will only work
//! for macros from `std`, `core`, and `alloc`. All other lazy macro calls must be wrapped in `lazy`
//! or `suspend_eager`.
//!
//! Some additional helpers are also provided:
//!
//! * `ccase!`: Modifies the case of a string or ident.
//! * `eager_coalesce!`: Selects the first non-empty stream of tokens from a set of trees.
//! * `eager_if!`: Selects between two streams of tokens based on a third.
//! * `token_eq!`: Compares two trees of tokens for equality.
//! * `unstringify!`: Parses a string into a stream of tokens.
//!
//! See each macro's documentation for details.
//!
//! # Usage
//!
//! ```
//! use eager2::{eager, eager_macro};
//!
//! //Declare an eager macro
//! #[eager_macro]
//! macro_rules! plus_1{
//!     ()=>{+ 1};
//! }
//!
//! // Use the macro inside an eager! call to expand it eagerly
//! assert_eq!(4, eager! {2 plus_1!() plus_1!()});
//! ```
//!
//! # Environments
//!
//! The ordinary macro environment of rust is `lazy`. This crate introduces a new environment which
//! is `eager`.
//!
//! In the `lazy` environment the expression `a!(b!(c!()))` evaluates as follows:
//! * `a!` is given the tokens `b!(c!())` and produces some output
//! * if that output contains `b!(c!())` then `b!` is given the tokens `c!()`
//! * if that output contains `c!()` then `c!()` will be evaluated.
//!
//! As we can see, the outer most macro is evaluated first. In the "eager" environment it's exactly
//! the opposite. The expression `a!(b!(c!()))` evaluates like:
//! * `c!()` is evaluated and expands to some tokens
//! * `b!` is given those expanded `c!` tokens and expands to some tokens.
//! * `a!` is given those expanded `b!` tokens and expands to some tokens.
//!
//! An advantage of the eager approach that might not be immediately obvious is that in contrast to
//! the lazy approach, only the final expansion must be syntactically valid. This temporary
//! invalidity enables things like `concat_idents` to be trivially constructed in a much more
//! powerful way.
//!
//! ```
//! use eager2::{eager, eager_macro, unstringify};
//!
//! #[eager_macro]
//! macro_rules! my_concat_idents {
//!     ($($e:ident),+ $(,)?) => { unstringify!(concat!(
//!         $(stringify!($e)),*
//!     )) };
//! }
//!
//! // std::concat_idents can't do this
//! eager! {
//!     fn my_concat_idents!(foo, bar)() -> u32 { 23 }
//! }
//!
//! let f = my_concat_idents!(foo, bar);
//!
//! println!("{}", f());
//! ```
//!
//! One other thing to keep in mind is that the eager environment automatically includes this crate
//! in it's prelude, so calls within an eager environment should not need any explicit pathing.
//!
//! ## Switching Environments
//!
//! The three macros which control the current environment are `eager!`, `lazy!`, and
//! `suspend_eager!`. They work as follows
//! ```
//! // The default rust environment is lazy
//! # use eager2::eager;
//! eager! {
//!
//!     // This environment is eager
//!
//!     mod foo {
//!
//!         // Still eager, `mod` and other declarations don't impact the environment
//!
//!         lazy!{
//!
//!             // Back to lazy
//!
//!             eager!{
//!                 // Eager again
//!             }
//!
//!             // Back to lazy
//!         }
//!
//!         // Back to eager
//!
//!         suspend_eager!{
//!
//!             // Back to lazy
//!
//!             eager2::eager!{
//!                 // Still lazy, but note that `eager!{...}` is going
//!                 // to be a part of the final expansion, and so will
//!                 // become eager during that evaluation.
//!                 //
//!                 // The use of `eager2::` prefix is because `mod foo` does not
//!                 // import `eager`. This is something to be aware of when mixing
//!                 // lazy and eager.
//!             }
//!
//!         }
//!         // Back to eager
//!     }
//! }
//! ```
//!
//! So what's the difference between `lazy{eager!{...}}` and `suspend_eager{eager!{...}}` if the inner
//! part becomes eager eventually? Well, `suspend_eager!` forces eager evaluation to finish before
//! continuing, which means a syntactically valid expansion must occur. The difference can be seen
//! in this example:
//! ```
//! use eager2::{eager, eager_macro};
//!
//! #[eager_macro]
//! macro_rules! fn_body{
//!     ()=>{ foo() {} };
//! }
//!
//! eager! {
//!     // This is legal
//!     fn lazy!{eager!{ fn_body!{} }}
//!
//!     // This is not legal
//!     // error: missing parameters for function definition
//!     //fn suspend_eager!{eager!{ fn_body!{} }}
//! }
//! ```
//!
//! # Exporting Macros
//!
//! When exporting a macro which calls into this crate, some care must be taken to deal with the
//! correct paths. The recommended convention is to add:
//! ```ignore
//! #[doc(hidden)]
//! pub extern crate eager2;
//! ```
//!
//! to the root of your crate and to utilize the `$crate::eager2::` prefix to call any macros when
//! in a lazy environment (eager environments should not require any prefix, but will identify macro
//! items with the `$crate::eager2::` as belonging to this crate).
//!
//! # Documentation Convention
//!
//! To make it clearly visible that a given macro is `eager!`-enabled, its short rustdoc description
//! must start with a pair of brackets, within which a link to the official `eager!` macro
//! documentation must be provided. The link's visible text must be 'eager!' and the brackets must
//! not be part of the link.
//!
//! # Limitations
//!
//! `eager!` is implemented using recursive macros, so the compiler's default macro recursion limit
//! can be exceeded. So occasionally you may have to use `#![recursion_limit="256"]` or higher. You
//! can also mitigate this by adding `suspend_eager!{eager!{...}}` blocks around code which calls
//! custom `eager!`-enabled macros (prelude macros are optimized and will not benefit from this).
//! This can impact the legality of your output, and so should only be done where it will not cause
//! breakage.
//!
//! Debugging an macros can be quite difficult, eager macros especially so. [`compile_error!`] can
//! sometimes be helpful in this regard. This crate also has `trace_macros` feature which only
//! operates on `nightly`, and can be quite verbose. Contributions to improve the debugging
//! experience are very welcome.
//!
//! Only `eager!`-enabled macros can be eagerly expanded, so existing macros do not gain much.
//! The `lazy!` block alleviates this a bit, by allowing the use of existing macros in it, while
//! eager expansion can be done around them. Luckily, `eager!`-enabling an existing macro should
//! not be too much trouble using `#[eager_macro]`.

#![cfg_attr(docsrs, deny(rustdoc::broken_intra_doc_links))]
#![deny(missing_docs, unsafe_code)]
#![allow(clippy::enum_glob_use)]

use proc_macro::TokenStream;

mod exec;
mod impls;
mod init;
mod parse;
mod rules;

/// [[eager!](macro.eager.html)] Declares [eager!](macro.eager.html)-enabled macros.
///
/// ## Syntax
/// ```
/// # use eager2::eager_macro;
/// #[eager_macro]
/// macro_rules! eager_macro_rules {
///     ($($dollar:tt $id_1:ident)?
///     $(
///         $(#[$($metas:meta)*])*
///         macro_rules! $macro_name:ident {
///             $($rules:tt => $expansions:tt);+ $(;)?
///         }
///     )*) => { /* proc-macro */ };
/// }
/// ```
/// # Usage
///
/// Where possible, users are encourage to switch to [`#[eager_macro]`][macro@eager_macro].
///
/// Wraps the usual `macro_rules!` syntax. Prior to any `macros_rules` an identifier may be given,
/// preceded by '$'. Then any number of macro declarations can be given using the usual
/// `macro_rules!` syntax. Documentation and attributes are also given in the
/// usual way just before each `macro_rules!`, i.e. inside `eager_macro_rules!`.
///
/// Some restrictions apply to the `macro_rules!` declarations:
///
/// * `macro_rules!` should avoid using identifiers that start with `__eager2_ident_`. This is
///   because in order to make macros eager, `eager_macro_rules` needs to add it's own unique
///   fragment identifier that does conflict with any of those present in the `macro_rules`. If
///   absolutely necessary, it is possible to override this identifier by providing an alternative,
///   prefixed with the token `$` as the first tokens to `eager_macro_rules`.
/// * No macros should use (either in calling or expansion) the literal `0𓊆eager2𓊇` as their
///   first token, as this could conflict with the way eager expansion state is managed.
///
/// # Examples
///
/// [eager!](macro.eager.html)-enabling the following macro:
/// ```
/// /// Some documentation
/// #[macro_export]
/// macro_rules! some_macro {
///     () => {};
/// }
/// ```
/// is done by wrapping it in `eager_macro_rules!` as follows:
/// ```
/// use eager2::eager_macro_rules;
///
/// eager_macro_rules! {
///     /// Some documentation
///     #[macro_export]
///     macro_rules! some_macro{
///         ()=>{};
///     }
/// }
/// ```
/// where `()=>{};` is the list of rules that comprise the macro.
#[proc_macro]
pub fn eager_macro_rules(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    rules::eager_macro_rules(stream.into()).into()
}

/// Declares an [eager!](macro.eager.html)-enabled macro.
///
/// # Usage
///
/// Should only be applied to `macro_rules!` items. Can be applied anywhere in the sequence of
/// attributes.
///
/// Some restrictions apply to the `macro_rules!` declarations:
///
/// * `macro_rules!` should avoid using identifiers that start with `__eager2_ident_`. This is
///   because in order to make macros eager, `eager_macro` needs to add it's own unique
///   fragment identifier that does conflict with any of those present in the `macro_rules`. If
///   absolutely necessary, it is possible to override this identifier by providing an alternative as
///   a parameter, i.e. `#[eager_macro(__my_unused_ident_name)]`.
/// * No macros should use (either in calling or expansion) the literal `0𓊆eager2𓊇` as their
///   first token, as this could conflict with the way eager expansion state is managed.
///
/// # Examples
///
/// [eager!](macro.eager.html)-enabling the following macro:
/// ```
/// // attr can be added here
/// /// Some documentation
/// // or here
/// #[macro_export]
/// // or here
/// macro_rules! some_macro {
///     () => {};
/// }
/// ```
/// is done by adding the `#[eager_macro]` attribute:
/// ```
/// /// Some documentation
/// #[macro_export]
/// #[eager2::eager_macro] // aesthetically here looks best, but you do you
/// macro_rules! some_macro {
///     () => {};
/// }
/// ```
///
/// # Implementation Details
///
/// The `macro_rules!` being modified by this attribute gets some special rules inserted at the head
/// of the macro rules list. These rules are slightly modified duplicates of the user-defined rules,
/// in that they look for an "eager sigil" being passed, followed by a serialization of the eager
/// environment state, followed by the original match expression of the rule. These rules expand to
/// a macro call into this crate's environment evaluator with the environment state followed by the
/// original expansion of the rule.
#[proc_macro_attribute]
pub fn eager_macro(attr: TokenStream, stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    rules::eager_macro(attr.into(), stream.into()).into()
}

/// [[eager!](macro.eager.html)] Emulates eager expansion of macros.
///
/// # Examples
///
/// ```
/// use eager2::{eager, eager_macro};
///
/// //Declare an eager macro
/// #[eager_macro]
/// macro_rules! plus_1{
///     ()=>{+ 1};
/// }
///
/// // Use the macro inside an eager! call to expand it eagerly
/// assert_eq!(4, eager! {2 plus_1!() plus_1!()});
/// ```
///
/// # Usage
///
/// `eager!` can wrap any code. If that code contains a macro call, it will be expanded before its
/// consumer. This means:
///
/// * If a macro call `b!{..}` is given as an argument to another macro `a!` (i.e. `a!(b!{...})`),
///   the inner macro (`b!`) will be expanded first.
/// * All macros will be fully expanded before `eager!` expands. Therefore, otherwise illegal
///   intermediate expansion steps are possible.
///
/// `eager!` does not work with any macro; only macros declared using
/// [`#[eager_macro]`][macro@eager_macro] or [`eager_macro_rules!`] may be used. Such macros are
/// said to be `eager!`-enabled.
///
/// To facilitate the use of non-`eager!`-enabled macros inside an `eager!` call,
/// a [`lazy!`] block can be inserted. Everything inside the `lazy!` block will be lazily expanded,
/// while everything outside it will continue to be eagerly expanded. Since, `lazy!` reverts
/// to the usual rules for macro expansion, an `eager!` block can be inserted inside the `lazy!`
/// block, to re-enable eager expansion for some subset of it.
///
/// ---
/// # Macro expansions
///
/// Rust is lazy when it comes to macro expansion. When the compiler sees a macro call, it will
/// try to expand the macro without looking at its arguments or what the expansion becomes.
/// Using `eager!`, previously illegal macro expansions can be made possible.
///
/// The following is a non-exhaustive list of illegal macro patterns that can be used with `eager!`.
///
/// ### The arguments to a macro usually cannot be the resulting expansion of another macro call:
///
/// ```compile_fail
/// // Say you have a macro that adds two numbers:
/// macro_rules! add{
///     ($e1:expr, $e2:expr)=> {$e1 + $e2}
/// }
///
/// // And a macro that expands to two comma-separated numbers:
/// macro_rules! two_and_three{
///     ()=>{2,3}
/// }
///
/// // You cannot use the expansion of `two_and_three!` as an argument to `add!`:
/// let x = add!(two_and_three!()); // error
/// ```
///
/// The compiler will complain about no rule in `add!` accepting `two_and_three`, since it does not
/// get expanded before the `add!`, who requires two expressions and not just one.
///
/// With eager expansion, this can be made possible:
///
/// ```
/// use eager2::{eager, eager_macro};
///
/// #[eager_macro]
/// macro_rules! add {
///     ($e1:expr, $e2:expr) => { $e1 + $e2 };
/// }
///
/// #[eager_macro]
/// macro_rules! two_and_three{
///     ()=>{2,3}
/// }
///
/// let x = eager! {add!(two_and_three!())};
/// assert_eq!(5, x);
/// ```
///
/// This is even possible if `add` cannot be made eager (e.g. it is defined in a dependency) using
/// `lazy!`:
///
/// ```
/// # /*
/// extern crate dependency {
/// # */
/// # mod dependency {
/// # /*
///     #[macro_export]
/// # */
///     macro_rules! add {
///         ($e1:expr, $e2:expr)=> {$e1 + $e2}
///     }
///     # pub(crate) use add;
/// }
///
/// use eager2::{eager_macro, lazy};
///
/// #[eager_macro]
/// macro_rules! two_and_three {
///     ()=>{2,3}
/// }
///
/// let x = lazy!{dependency::add!(eager!(two_and_three!()))};
/// assert_eq!(5, x);
/// ```
///
/// ### Macros are illegal in some contexts (e.g. as an identifier)
///
/// ```compile_fail
/// // Say you have a macro that expands to an identifier:
/// macro_rules! id{
///     ()=> {SomeStruct}
/// }
///
/// // And want to use it to declare a struct:
/// struct id!(){
///     v: u32
/// }
/// ```
///
/// This will not compile since macros are illegal in identifier position. The compiler does
/// not check whether the expansion of the macro will result in valid Rust code.
///
/// With eager expansion, `id!` will expand before the `eager!` block , making it possible to use it
/// in an identifier position:
/// ```
/// use eager2::{eager, eager_macro};
///
/// #[eager_macro]
/// macro_rules! id {
///     () => { SomeStruct };
/// }
///
/// eager! {
///     struct id!(){
///         v: u32
///     }
/// }
///
/// let some_struct = SomeStruct { v: 4 };
/// assert_eq!(4, some_struct.v);
/// ```
/// To circumvent any restriction on where macros can be used, we can therefore just wrap
/// the code surrounding the macro call with `eager!`. The `eager!` must still be in a valid position,
/// but in the worst case it can be put around the whole item
/// (struct, trait, implement, function, etc.).
///
///
/// ### No intermediate expansion step can include invalid syntax
///
/// Say we want to create a macro that interprets natural language, converting it into an
/// expression.
///
/// ```compile_fail
/// // We start by declaring a macro that interprets operator words:
/// macro_rules! op{
///     ( plus ) => { + };
///     ( minus ) => { - };
/// }
///
/// // We then declare a macro that interprets integer words:
/// macro_rules! integer{
///     ( one ) => { 1 };
///     ( two ) => { 2 };
/// }
///
/// // Lastly, we declare the top-level macro that uses the previous two macros to
/// // expand into an expression:
/// macro_rules! calculate{
///     ( $lhs:tt $op:tt $rhs:tt ) => {
///          integer!{$lhs} op!{$op} integer!{$rhs}
///     };
/// }
///
/// // Using this macro will fail to compile:
/// let x = calculate!(one plus two); //Error
/// ```
///
/// Looking at the first expansion step we can see that three macro calls in a sequence
/// are not a valid expression:
/// ```ignore
/// let x = integer!(one) op!{plus} integer!(two); //Error
/// ```
///
/// We can circumvent this restriction, by having `calculate!` wrap its output in an `eager!`:
///
/// ```
/// use eager2::{eager_macro, eager};
///
/// #[eager_macro]
/// macro_rules! op{
///     ( plus ) => { + };
///     ( minus ) => { - };
/// }
///
/// #[eager_macro]
/// macro_rules! integer{
///     ( one ) => { 1 };
///     ( two ) => { 2 };
/// }
///
/// macro_rules! calculate{
///     ( $lhs:tt $op:tt $rhs:tt ) => {
///          eager!{integer!{$lhs} op!{$op} integer!{$rhs}}
///     };
/// }
///
/// let x = calculate!(one plus two);
/// assert_eq!(3, x);
/// ```
///
/// In this case, `calculate!` does not actually have to be `eager!`-enabled, since it is never used
/// inside an `eager!` block. That said, we probably should enable it so that others may later use
/// it in eager environments.
#[proc_macro]
pub fn eager(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eval(stream.into(), true).into()
}

/// [[eager!](macro.eager.html)] Switches the macro-environment to lazy-mode.
///
/// If this macro is called independently of `eager!`, it is equivalent to `eager!{lazy!{...}}`.
///
/// For more information on environments, see [the crate-level documentation][environments].
///
/// [environments]: ./index.html#environments
#[proc_macro]
pub fn lazy(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eval(stream.into(), false).into()
}

/// [[eager!](macro.eager.html)] Locks the macro-environment in lazy-mode.
///
/// If this macro is called independently of `eager!`, it is equivalent to
/// `eager!{suspend_eager!{...}}`.
///
/// For more information on environments, see [the crate-level documentation][environments].
///
/// [environments]: ./index.html#environments
#[proc_macro]
pub fn suspend_eager(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "suspend_eager").into()
}

/// 🚧 Not yet implemented!
///
/// [[eager!](macro.eager.html)] Evaluates boolean combinations of configuration flags at compile-time.
///
/// The syntax given to this macro is the same syntax as the [`cfg`] attribute.
///
/// This is an `eager!`-enabled version of [`std::cfg`].
///
/// [`cfg`]: https://doc.rust-lang.org/reference/conditional-compilation.html#the-cfg-attribute
#[proc_macro]
pub fn cfg(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "cfg").into()
}

/// 🚧 Not yet implemented!
///
/// [[eager!](macro.eager.html)] Expands to the column number at which it was invoked.
///
/// With [`line!`] and [`file!`], these macros provide debugging information for
/// developers about the location within the source.
///
/// The expanded expression has type `u32` and is 1-based, so the first column
/// in each line evaluates to 1, the second to 2, etc. This is consistent
/// with error messages by common compilers or popular editors.
/// The returned column is *not necessarily* the line of the `column!` invocation itself,
/// but rather the first macro invocation leading up to the invocation
/// of the `column!` macro.
///
/// This is an `eager!`-enabled version of [`std::column`].
#[proc_macro]
pub fn column(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "column").into()
}

/// [[eager!](macro.eager.html)] Causes compilation to fail with the given error message when encountered.
///
/// This macro should be used when a crate uses a conditional compilation strategy to provide
/// better error messages for erroneous conditions. It's the compiler-level form of [`panic!`],
/// but emits an error during *compilation* rather than at *runtime*.
///
/// This is an `eager!`-enabled version of [`std::compile_error`].
#[proc_macro]
pub fn compile_error(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "compile_error").into()
}

/// [[eager!](macro.eager.html)] Concatenates literals into a static string slice.
///
/// This macro takes any number of comma-separated literals, yielding an
/// expression of type `&'static str` which represents all of the literals
/// concatenated left-to-right.
///
/// Integer and floating point literals are [stringified](core::stringify) in order to be
/// concatenated.
#[proc_macro]
pub fn concat(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "concat").into()
}

/// [[eager!](macro.eager.html)] Inspects an environment variable at compile time.
///
/// This macro will expand to the value of the named environment variable at
/// compile time, yielding an expression of type `&'static str`
#[proc_macro]
pub fn env(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "env").into()
}

/// [[eager!](macro.eager.html)] Optionally inspects an environment variable at compile time.
///
/// If the named environment variable is present at compile time, this will expand into an
/// expression of type `Option<&'static str>` whose value is `Some` of the value of the environment
/// variable (a compilation error will be emitted if the environment variable is not a valid Unicode
/// string). If the environment variable is not present, then this will expand to `None`.
#[proc_macro]
pub fn option_env(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "option_env").into()
}

/// 🚧 Not yet implemented!
///
/// [[eager!](macro.eager.html)] Expands to the file name in which it was invoked.
///
/// With [`line!`] and [`column!`], these macros provide debugging information for
/// developers about the location within the source.
///
/// The expanded expression has type `&'static str`, and the returned file
/// is not the invocation of the `file!` macro itself, but rather the
/// first macro invocation leading up to the invocation of the `file!`
/// macro.
///
/// This is an `eager!`-enabled version of [`std::file`].
#[proc_macro]
pub fn file(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "file").into()
}

/// [[eager!](macro.eager.html)] Parses a file as an expression or an item according to the context.
///
/// The included file is placed in the surrounding code
/// [unhygienically](https://doc.rust-lang.org/reference/macros-by-example.html#hygiene). If
/// the included file is parsed as an expression and variables or functions share names across
/// both files, it could result in variables or functions being different from what the
/// included file expected.
///
/// The included file is located relative to the current file (similarly to how modules are
/// found). The provided path is interpreted in a platform-specific way at compile time. So,
/// for instance, an invocation with a Windows path containing backslashes `\` would not
/// compile correctly on Unix.
#[proc_macro]
pub fn include(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "include").into()
}
/// [[eager!](macro.eager.html)] Includes a file as a reference to a byte array.
///
/// The file is located relative to the current file (similarly to how
/// modules are found). The provided path is interpreted in a platform-specific
/// way at compile time. So, for instance, an invocation with a Windows path
/// containing backslashes `\` would not compile correctly on Unix.
///
/// This macro will yield an expression of type &'static [u8; N] which is the contents of the file.
#[proc_macro]
pub fn include_bytes(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "include_bytes").into()
}

/// [[eager!](macro.eager.html)] Includes a UTF-8 encoded file as a string.
///
/// The file is located relative to the current file (similarly to how
/// modules are found). The provided path is interpreted in a platform-specific
/// way at compile time. So, for instance, an invocation with a Windows path
/// containing backslashes `\` would not compile correctly on Unix.
///
/// This macro will yield an expression of type `&'static str` which is the
/// contents of the file.
#[proc_macro]
pub fn include_str(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "include_str").into()
}

/// 🚧 Not yet implemented!
///
/// [[eager!](macro.eager.html)] Expands to the line number on which it was invoked.
///
/// With [`column!`] and [`file!`], these macros provide debugging information for
/// developers about the location within the source.
///
/// The expanded expression has type `u32` and is 1-based, so the first line
/// in each file evaluates to 1, the second to 2, etc. This is consistent
/// with error messages by common compilers or popular editors.
/// The returned line is *not necessarily* the line of the `line!` invocation itself,
/// but rather the first macro invocation leading up to the invocation
/// of the `line!` macro.
#[proc_macro]
pub fn line(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "line").into()
}

/// 🚧 Not yet implemented!
///
/// [[eager!](macro.eager.html)] Expands to a string that represents the current module path.
///
/// The current module path can be thought of as the hierarchy of modules
/// leading back up to the crate root. The first component of the path
/// returned is the name of the crate currently being compiled.
#[proc_macro]
pub fn module_path(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "module_path").into()
}

/// [[eager!](macro.eager.html)] Stringifies its arguments.
///
/// This macro will yield an expression of type `&'static str` which is the
/// stringification of all the tokens passed to the macro. No restrictions
/// are placed on the syntax of the macro invocation itself.
///
/// Note that the expanded results of the input tokens may change in the
/// future. You should be careful if you rely on the output.
#[proc_macro]
pub fn stringify(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "stringify").into()
}

/// [[eager!](macro.eager.html)] Convert strings between snake case, kebab case, camel case,
/// title case, pascal case, and so many more.
///
/// ## Syntax
/// ```
/// # use eager2::eager_macro;
/// #[eager_macro]
/// macro_rules! ccase {
///     (
///         $arg_1:ident,
///         $($param:ident : $val:literal),+ $(,)?
///     ) => { /* proc-macro */ };
///     (
///         $arg_1:literal,
///         $($param:ident : $val:literal),+ $(,)?
///     ) => { /* proc-macro */ };
/// }
/// ```
///
/// The first argument can either be a string literal or an identifier which determines the output
/// type. Afterward one or more named parameters should be supplied with string literal value. Valid
/// parameter names are:
/// * `t` or `to`: Transform a string in**to** a certain string case.
/// * `f` or `from`: Splits string based on the boundaries associated with that case.  For example, `"snake"` case splits on underscores `_` and `"camel"` splits based on lowercase characters followed by uppercase characters `aA`.
/// * `b` or `boundaries`: Specify precisely what conditions should be used for splitting a string into words.  Whatever boundaries are present in the boundary string will be used against the input. Any example can do, but a nice way to specify is to separate boundaries with a `:`.  For example, `aA:a1:1a` will split based on lowercase followed by uppercase, lowercase followed by and preceded by a digit.
/// * `p` or `pattern`: Transforms the words of the input based on a pattern.  A pattern describes the order of "word cases".  For example, the camel pattern is a lowercase word followed by capitalized words, like in camel case, while the lower pattern is just lowercased words, like in snake case.
/// * `d` or `delimiter`: Join the words using a delimiter. By default, the delimiter is an empty string, as used in camel case.
///
/// In general, you only need to use `t: "snake"`, `t: "UpperCamelCase"`, or `t: "CONSTANT"`.
///
/// See the [ccase] utility for a more detailed explanation of the options.
///
/// # Examples
///
/// ```
/// use eager2::eager;
///
/// eager! { let ccase!(myVarName, t: "snake") = 10; }
/// assert_eq!(my_var_name, 10);
/// ```
///
/// [ccase]: https://github.com/rutrum/ccase
#[proc_macro]
pub fn ccase(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "ccase").into()
}

/// [[eager!](macro.eager.html)] Expands into the first non-empty block.
///
/// ## Syntax
/// ```
/// # use eager2::eager_macro;
/// #[eager_macro]
/// macro_rules! eager_coalesce {
///     (
///         $($($arg:tt),+ $(,)?)?
///     ) => { /* proc-macro */ };
/// }
/// ```
///
/// Takes a series of blocks and expands into the first non-empty one. Useful for setting default
/// values. Each argument must be enclosed in `{}`, `[]`, or `()`.
///
/// # Examples
///
/// ```
/// use eager2::eager;
///
/// eager! {
///     eager_coalesce!{{}, {let v = 10;}, {let v = 100;}}
/// }
/// assert_eq!(v, 10);
/// ```
#[proc_macro]
pub fn eager_coalesce(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "eager_coalesce").into()
}

/// [[eager!](macro.eager.html)] Conditionally expands into one of two blocks.
///
/// `eager_if` should never be called from outside an eager environment.
///
/// ## Syntax
///
/// `eager_if!` should be followed by 3 token groups.
/// * The first group should contain a single boolean literal the value of which determines which
///   branch the expression will expand to. By convention this first tree uses `[]` delimiter.
/// * The second group contains the tokens for the `true` case. By convention it uses `{}`
///   delimiter.
/// * The third group contains the tokens for the `false` case. By convention it uses `{}`
///   delimiter.
///
/// # Examples
/// ```
/// use eager2::eager;
///
/// eager! {
///     eager_if![true]{let v = 10;}{^ invalid syntax code ^}
/// }
/// assert_eq!(v, 10);
/// ```
#[proc_macro]
pub fn eager_if(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "eager_if").into()
}

/// [[eager!](macro.eager.html)] Expands into the token `true` if all arguments are token-wise
/// equivalent.
///
/// ## Syntax
/// ```
/// # use eager2::eager_macro;
/// #[eager_macro]
/// macro_rules! token_eq {
///     (
///         $($arg:tt),+ $(,)?
///     ) => { /* proc-macro */ };
/// }
/// ```
///
/// Takes a series of blocks and expands the literal `true` if all tokens are equal. Each argument
/// must be enclosed in `{}`, `[]`, or `()`. This enclosure is ignored for purposes of comparison.
///
/// # Examples
/// ```
/// use eager2::eager;
///
/// let r = eager! {token_eq!{{a}, {a}}};
/// assert!(r);
/// let r = eager! {token_eq!{{a}, {a}, {b}}};
/// assert!(!r);
/// ```
///
/// ```compile_fail
/// use eager2::eager;
///
/// let r = eager! {token_eq!{}}; // At least one argument needs to be provided
/// assert!(r);
/// ```
#[proc_macro]
pub fn token_eq(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "token_eq").into()
}

/// [[eager!](macro.eager.html)] Parses a string as tokens.
///
/// ## Syntax
/// ```
/// # use eager2::eager_macro;
/// #[eager_macro]
/// macro_rules! unstringify {
///     ($arg:literal $(,)?) => { /* proc-macro */ };
/// }
/// ```
///
/// # Examples
/// ```
/// use eager2::eager;
///
/// eager! {unstringify!("let v = 10;")}
/// assert_eq!(v, 10);
/// ```
#[proc_macro]
pub fn unstringify(stream: TokenStream) -> TokenStream {
    #[allow(clippy::useless_conversion)]
    impls::eager_wrap(stream.into(), "unstringify").into()
}
