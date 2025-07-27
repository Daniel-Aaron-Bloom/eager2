use crate::{
    parse::IdentOrString,
    pm::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, ToTokens, TokenStream},
};
pub trait NextOr: Iterator {
    fn next_or<E>(&mut self, e: E) -> Result<Self::Item, E>;
}

impl<I: Iterator> NextOr for I {
    fn next_or<E>(&mut self, e: E) -> Result<Self::Item, E> {
        self.next().ok_or(e)
    }
}

pub trait PopNext {
    type Item;
    fn pop_next(&mut self) -> Option<Self::Item>;
}

impl<T> PopNext for Vec<T>
where
    T: Iterator,
{
    type Item = T::Item;
    fn pop_next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(n) = self.last_mut()?.next() {
                return Some(n);
            }
            self.pop();
        }
    }
}

#[must_use]
pub(crate) fn lifetime(name: IdentOrString) -> TokenStream {
    let mut stream = TokenStream::new();
    Punct::new('\'', Spacing::Joint).to_tokens(&mut stream);
    Ident::new(
        match name {
            IdentOrString::Ident(i) => i.to_string(),
            IdentOrString::String(s) => s,
        }
        .as_str(),
        Span::call_site(),
    )
    .to_tokens(&mut stream);
    stream
}

#[must_use]
pub fn eager_data(
    eager_call_sigil: &Literal,
    state: TokenStream,
    tail: TokenStream,
) -> TokenStream {
    let mut tokens = TokenStream::new();
    eager_call_sigil.to_tokens(&mut tokens);
    Group::new(Delimiter::Bracket, state).to_tokens(&mut tokens);
    tokens.extend(tail);
    tokens
}

/// Roughly the same as `quote::quote!`, but faster, without repetitions,
/// and working directly with proc_macro. There might exist some valid Rust
/// code which could not be recognised by this macro, but it's enough for
/// our use case
#[macro_export]
macro_rules! interpolate {
    {} => {
        ::proc_macro::TokenStream::new()
    };
    {$($tokens:tt)+} => {{
        let mut interpolated = $crate::interpolate! {};
        interpolated.extend(
            $crate::interpolate_inner! {
                @already_interpolated[::proc_macro::TokenStream::new()]
                $($tokens)+
            }
        );
        interpolated
    }};
}

#[macro_export]
macro_rules! interpolate_inner {
    {@already_interpolated[$($interpolated:tt)*]} => {[$($interpolated)*]};

    // interpolated
    {@already_interpolated[$($interpolated:tt)*] #$ident:ident $($rest:tt)*} => {
        $crate::interpolate_inner! {
            @already_interpolated[
                $($interpolated)*,
                $crate::pm::ToTokens::to_token_stream(
                    &$ident
                )
            ]
            $($rest)*
        }
    };

    // ident
    {@already_interpolated[$($interpolated:tt)*] $ident:ident $($rest:tt)*} => {
        $crate::interpolate_inner! {
            @already_interpolated[
                $($interpolated)*,
                $crate::pm::ToTokens::into_token_stream(
                    ::proc_macro::Ident::new(
                        stringify!($ident),
                        ::proc_macro::Span::call_site()
                    )
                )
            ]
            $($rest)*
        }
    };
    {@already_interpolated[$($interpolated:tt)*] _ $($rest:tt)*} => {
        $crate::interpolate_inner! {
            @already_interpolated[
                $($interpolated)*,
                $crate::pm::ToTokens::into_token_stream(
                    ::proc_macro::Ident::new(
                        "_",
                        ::proc_macro::Span::call_site()
                    )
                )
            ]
            $($rest)*
        }
    };

    // literal
    {@already_interpolated[$($interpolated:tt)*] $lit:literal $($rest:tt)*} => {
        $crate::interpolate_inner! {
            @already_interpolated[
                $($interpolated)*,
                {
                    use ::core::str::FromStr;

                    $crate::pm::ToTokens::into_token_stream(
                        ::proc_macro::Literal::from_str(
                            stringify!($lit)
                        ).expect(
                            concat!(
                                "`interpolate!` could not create a literal with ",
                                stringify!($lit)
                            )
                        )
                    )
                }
            ]

            $($rest)*
        }
    };

    // group
    {@already_interpolated[$($interpolated:tt)*] { $($tokens:tt)* } $($rest:tt)*} => {
        $crate::interpolate_inner! {
            @already_interpolated[
                $($interpolated)*,
                $crate::pm::ToTokens::into_token_stream(
                    ::proc_macro::Group::new(
                        ::proc_macro::Delimiter::Brace,
                        $crate::interpolate! { $($tokens)* }
                    )
                )
            ]
            $($rest)*
        }
    };
    {@already_interpolated[$($interpolated:tt)*] [ $($tokens:tt)* ] $($rest:tt)*} => {
        $crate::interpolate_inner! {
            @already_interpolated[
                $($interpolated)*,
                $crate::pm::ToTokens::into_token_stream(
                    ::proc_macro::Group::new(
                        ::proc_macro::Delimiter::Bracket,
                        $crate::interpolate! { $($tokens)* }
                    )
                )
            ]
            $($rest)*
        }
    };
    {@already_interpolated[$($interpolated:tt)*] ( $($tokens:tt)* ) $($rest:tt)*} => {
        $crate::interpolate_inner! {
            @already_interpolated[
                $($interpolated)*,
                $crate::pm::ToTokens::into_token_stream(
                    ::proc_macro::Group::new(
                        ::proc_macro::Delimiter::Parenthesis,
                        $crate::interpolate! { $($tokens)* }
                    )
                )
            ]
            $($rest)*
        }
    };

    // special tokens
    {@already_interpolated[$($interpolated:tt)*] $lifetime:lifetime $($rest:tt)*} => {
        $crate::interpolate_inner! {
            @already_interpolated[
                $($interpolated)*,
                $crate::pm::ToTokens::into_token_stream(
                    $crate::utils::quote_lifetime(
                        stringify!($lifetime)
                    )
                )
            ]
            $($rest)*
        }
    };

    // punct (or related)
    // this matches a [`Token`](https://doc.rust-lang.org/nightly/reference/tokens.html#grammar-Token),
    // which, at this point must be a [`PUNCTUATION`](https://doc.rust-lang.org/nightly/reference/tokens.html#grammar-PUNCTUATION)
    {@already_interpolated[$($interpolated:tt)*] $punct:tt $($rest:tt)*} => {
        $crate::interpolate_inner! {
            @already_interpolated[
                $($interpolated)*,
                $crate::pm::ToTokens::into_token_stream(
                    $crate::utils::quote_punct(
                        stringify!($punct)
                    )
                )
            ]
            $($rest)*
        }
    };
}

#[allow(dead_code)]
pub(crate) fn quote_lifetime(lifetime: &str) -> TokenStream {
    let trimmed = lifetime.trim();
    debug_assert_eq!(trimmed.chars().next(), Some('\''));

    let mut stream = TokenStream::new();

    // SAFETY: « ' » needs one byte to be encoded as UTF-8,
    // so removing it still yields valid UTF-8
    let name = unsafe { str::from_utf8_unchecked(&lifetime.as_bytes()[1..]) };

    Punct::new('\'', Spacing::Joint).to_tokens(&mut stream);
    Ident::new(name, Span::call_site()).to_tokens(&mut stream);

    stream
}

#[allow(dead_code)]
pub(crate) fn quote_punct(punct: &str) -> TokenStream {
    let trimmed = punct.trim();
    debug_assert!(trimmed.chars().count() <= 3);

    let mut stream = TokenStream::new();
    let mut chars = trimmed.chars().peekable();

    while let Some(ch) = chars.next() {
        if chars.peek().is_some() {
            Punct::new(ch, Spacing::Joint).to_tokens(&mut stream);
        } else {
            Punct::new(ch, Spacing::Alone).to_tokens(&mut stream);
        }
    }

    stream
}

#[cfg(rustchan = "nightly")]
pub(crate) fn join_spans(s1: Span, s2: Span) -> Span {
    s1.join(s2).unwrap_or(s1)
}

#[cfg(not(rustchan = "nightly"))]
pub(crate) fn join_spans(s1: Span, _: Span) -> Span {
    s1
}
