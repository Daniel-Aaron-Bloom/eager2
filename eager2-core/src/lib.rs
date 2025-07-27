#![cfg_attr(rustchan = "nightly", feature(proc_macro_span))]
#![recursion_limit = "1024"]

//! This library is not meant for public consumption
#![doc(hidden)]

use std::borrow::Cow;

#[cfg(not(feature = "testing"))]
extern crate proc_macro;

#[cfg(not(feature = "testing"))]
pub mod pm {
    pub use proc_macro::{
        token_stream, Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream,
        TokenTree,
    };
    pub trait ToTokens {
        fn to_tokens(&self, tokens: &mut TokenStream);
        fn to_token_stream(&self) -> TokenStream {
            let mut tokens = TokenStream::new();
            self.to_tokens(&mut tokens);
            tokens
        }
        fn into_token_stream(self) -> TokenStream
        where
            Self: Sized,
        {
            self.to_token_stream()
        }
    }
    impl ToTokens for Group {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend([TokenTree::from(self.clone())]);
        }
    }
    impl ToTokens for Ident {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend([TokenTree::from(self.clone())]);
        }
    }
    impl ToTokens for Punct {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend([TokenTree::from(self.clone())]);
        }
    }
    impl ToTokens for Literal {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend([TokenTree::from(self.clone())]);
        }
    }
    impl ToTokens for TokenTree {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend([self.clone()]);
        }
    }
    impl ToTokens for TokenStream {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            tokens.extend([self.clone()]);
        }
    }
    impl ToTokens for &str {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            Literal::string(self).to_tokens(tokens);
        }
    }
    impl<const N: usize, T: ToTokens> ToTokens for [T; N] {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            for item in self {
                item.to_tokens(tokens);
            }
        }
    }
    impl<T: ToTokens> ToTokens for &T {
        fn to_tokens(&self, tokens: &mut TokenStream) {
            (*self).to_tokens(tokens);
        }
    }
}

#[cfg(feature = "testing")]
pub mod pm {
    pub use proc_macro2::{
        token_stream, Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream,
        TokenTree,
    };
    pub use quote::ToTokens;
}

pub struct Error {
    pub span: pm::Span,
    pub msg: Cow<'static, str>,
    pub note: Option<Note>,
}

pub struct Note {
    pub span: Option<pm::Span>,
    pub msg: Cow<'static, str>,
}

impl pm::ToTokens for Error {
    fn to_tokens(&self, tokens: &mut pm::TokenStream) {
        fn ensure_lf(buf: &mut String, s: &str) {
            if s.ends_with('\n') {
                buf.push_str(s);
            } else {
                buf.push_str(s);
                buf.push('\n');
            }
        }

        let mut message = String::new();
        ensure_lf(&mut message, &self.msg);
        message.push('\n');

        if let Some(note) = self.note.as_ref() {
            message.push_str("  = note: ");
            ensure_lf(&mut message, &note.msg);
        }
        message.push('\n');

        let mut msg = pm::Literal::string(&message);
        msg.set_span(self.span);
        let mut group = pm::Group::new(pm::Delimiter::Brace, msg.into_token_stream());
        group.set_span(self.span);

        pm::Ident::new("compile_error", self.span).to_tokens(tokens);
        let mut exclamation = pm::Punct::new('!', pm::Spacing::Alone);
        exclamation.set_span(self.span);
        exclamation.to_tokens(tokens);
        group.to_tokens(tokens);
    }
}

pub mod consts;
pub mod egroup;
pub mod exec;
pub mod funcs;
pub mod parse;
pub mod rules;
pub mod state;
pub mod utils;
