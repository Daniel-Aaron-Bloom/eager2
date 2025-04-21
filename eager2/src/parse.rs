use std::sync::OnceLock;

use eager2_core::{
    consts::EAGER_CALL_SIGIL,
    parse::{eat_zero_group, Param},
    pm::{self, Span, TokenTree},
    Error, Note,
};
use litrs::Literal;

pub fn expect_call_literal(tt: Result<TokenTree, Span>) -> bool {
    static EAGER_CALL_LITERAL: OnceLock<Literal<String>> = OnceLock::new();
    let lit =
        EAGER_CALL_LITERAL.get_or_init(|| Literal::parse(EAGER_CALL_SIGIL).unwrap().into_owned());
    expect_literal(tt, lit.clone()).ok().is_none()
}

pub fn expect_literal<'a>(
    tt: Result<TokenTree, Span>,
    s: impl Into<Param<'a, Literal<String>>>,
) -> Result<(Literal<String>, Span), Error> {
    expect_literal_impl(tt, s.into())
}

fn expect_literal_impl(
    tt: Result<TokenTree, Span>,
    s: Param<Literal<String>>,
) -> Result<(Literal<String>, Span), Error> {
    let (tt, s) = match (tt.map(eat_zero_group), s) {
        (Err(span), Param::ExactValue(s)) => Err(Error {
            span,
            msg: "unexpected end of macro invocation".into(),
            note: Some(Note {
                span: None,
                msg: format!("while trying to match literal `{s}`").into(),
            }),
        }),
        (Err(span), Param::Named(s)) => Err(Error {
            span,
            msg: "unexpected end of macro invocation".into(),
            note: Some(Note {
                span: None,
                msg: format!("while trying to match literal `${s}:literal`").into(),
            }),
        }),
        (Ok(tt), s) => Ok((tt, s)),
    }?;
    let span = tt.span();

    match (Literal::try_from(tt.clone()), s) {
        (Err(_), Param::Named(s)) => Err(Error {
            span: tt.span(),
            msg: format!("expected literal: `${s}:literal`").into(),
            note: None,
        }),
        (Ok(l), Param::Named(_)) => Ok((l, span)),
        (Ok(l), Param::ExactValue(s)) if l == s => Ok((l, span)),
        (Err(_) | Ok(_), Param::ExactValue(s)) => Err(Error {
            span: tt.span(),
            msg: format!("expected literal: `{s}`").into(),
            note: None,
        }),
    }
}

pub fn get_string_literal(l: pm::Literal) -> Option<String> {
    if let Literal::String(s) = Literal::from(l) {
        Some(s.value().to_string())
    } else {
        None
    }
}

pub fn expect_string_literal(
    tt: Result<TokenTree, Span>,
    s: Param<&str>,
) -> Result<(String, Span), Error> {
    let (tt, s) = match (tt.map(eat_zero_group), s) {
        (Err(span), Param::ExactValue(s)) => Err(Error {
            span,
            msg: "unexpected end of macro invocation".into(),
            note: Some(Note {
                span: None,
                msg: format!("while trying to match string literal `{s}`").into(),
            }),
        }),
        (Err(span), Param::Named(s)) => Err(Error {
            span,
            msg: "unexpected end of macro invocation".into(),
            note: Some(Note {
                span: None,
                msg: format!("while trying to match string literal `${s}:literal`").into(),
            }),
        }),
        (Ok(tt), s) => Ok((tt, s)),
    }?;
    let span = tt.span();

    let (l, s) = match (Literal::try_from(tt), s) {
        (Err(_), Param::ExactValue(s)) => Err(Error {
            span,
            msg: format!("expected string literal: `{s}`").into(),
            note: None,
        }),
        (Err(_), Param::Named(s)) => Err(Error {
            span,
            msg: format!("expected string literal: `${s}:literal`").into(),
            note: None,
        }),
        (Ok(l), s) => Ok((l, s)),
    }?;

    match (l, s) {
        (Literal::String(l), Param::Named(_)) => Ok((l.into_value().into_owned(), span)),
        (Literal::String(l), Param::ExactValue(s)) if l.value() == s => {
            Ok((l.into_value().into_owned(), span))
        }
        (_, Param::ExactValue(s)) => Err(Error {
            span,
            msg: format!("expected string literal: `{s}`").into(),
            note: None,
        }),
        (_, Param::Named(s)) => Err(Error {
            span,
            msg: format!("expected string literal: `${s}:literal`").into(),
            note: None,
        }),
    }
}
