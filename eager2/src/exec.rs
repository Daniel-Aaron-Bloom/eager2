use std::{fmt::Write, str::FromStr};

use convert_case::{pattern::Pattern, Boundary, Case, Casing, Converter};
use eager2_core::{
    egroup::EfficientGroupV,
    parse::{expect_ident, expect_ident_or_string, expect_punct, IdentOrString, Param},
    pm::{self, Ident, Span, TokenStream, TokenTree},
    utils::NextOr,
    Error, Note,
};
use litrs::{BoolLit, Literal};

use crate::parse::{expect_literal, expect_string_literal};

pub fn execute_concat(
    span: Span,
    mut args: &mut dyn Iterator<Item = TokenTree>,
    processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    let mut buffer = String::new();
    while let Some(tt) = args.next() {
        let (tt, neg) = match expect_punct(Ok(tt.clone()), '-') {
            Ok(_) => (args.next_or(span), true),
            Err(_) => (Ok(tt), false),
        };

        let (l, span) = expect_literal(tt, Param::Named("arg"))?;
        if neg {
            buffer.push('-');
        }

        match (neg, l) {
            (_, Literal::Byte(_) | Literal::ByteString(_)) => {
                return Err(Error {
                    span,
                    msg: "cannot concatenate a byte string literal".into(),
                    note: None,
                })
            }
            (false, Literal::Bool(BoolLit::False)) => buffer.push_str("false"),
            (false, Literal::Bool(BoolLit::True)) => buffer.push_str("true"),
            (_, Literal::Float(f)) => {
                for s in f.number_part().split('_') {
                    buffer.push_str(s);
                }
            }
            (false, Literal::Char(c)) => buffer.push(c.value()),
            (false, Literal::String(s)) => buffer.push_str(s.value()),
            (_, Literal::Integer(i)) => {
                use litrs::IntegerBase::*;
                match (i.base(), i.suffix()) {
                    (
                        Decimal,
                        "f32" | "f64" | "" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8"
                        | "i16" | "i32" | "i64" | "i128",
                    ) => {
                        for s in i.raw_main_part().split('_') {
                            buffer.push_str(s);
                        }
                    }
                    (Binary, "f32" | "f64") => {
                        return Err(Error {
                            span,
                            msg: "binary float literal is not supported".into(),
                            note: None,
                        })
                    }
                    (Octal, "f32" | "f64") => {
                        return Err(Error {
                            span,
                            msg: "octal float literal is not supported".into(),
                            note: None,
                        })
                    }

                    (
                        _,
                        "" | "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32"
                        | "i64" | "i128",
                    ) => {
                        let Some(v) = i.value::<u128>() else {
                            return Err(Error {
                                span,
                                msg: "integer literal is too large".into(),
                                note: Some(Note {
                                    span: None,
                                    msg: format!("value `{i}` exceeds limit of `0xffffffffffffffffffffffffffffffff`").into(),
                                }),
                            });
                        };
                        write!(&mut buffer, "{v}").unwrap();
                    }
                    (_, s) => {
                        return Err(Error {
                            span,
                            msg: format!("invalid suffix `{s}` for number literal").into(),
                            note: None,
                        });
                    }
                }
            }
            _ => {
                return Err(Error {
                    span,
                    msg: "expected a literal".into(),
                    note: Some(Note {
                        span: None,
                        msg: r#"only literals (like `"foo"`, `-42` and `3.14`) can be passed to `concat!()`"#.into(),
                    }),
                });
            }
        }
        if let Some(tt) = args.next() {
            expect_punct(Ok(tt), ',')?;
        }
    }
    processed_out.push(TokenTree::Literal(pm::Literal::string(&buffer)));
    Ok(())
}

fn case_value_parser(span: Span, s: &str) -> Result<Case<'_>, Error> {
    const ALL_CASES: &[(&str, Case)] = &{
        [
            ("snake", Case::Snake),
            ("constant", Case::Constant),
            ("uppersnake", Case::UpperSnake),
            ("ada", Case::Ada),
            ("kebab", Case::Kebab),
            ("cobol", Case::Cobol),
            ("upperkebab", Case::UpperKebab),
            ("train", Case::Train),
            ("flat", Case::Flat),
            ("upperflat", Case::UpperFlat),
            ("pascal", Case::Pascal),
            ("uppercamel", Case::UpperCamel),
            ("camel", Case::Camel),
            ("lower", Case::Lower),
            ("upper", Case::Upper),
            ("title", Case::Title),
            ("sentence", Case::Sentence),
            ("alternating", Case::Alternating),
            ("toggle", Case::Toggle),
            ("screaming", Case::Constant),
            ("alternate", Case::Alternating),
        ]
    };

    let case_str = s.to_case(Case::Flat);
    for (name, case) in ALL_CASES {
        if case_str == *name {
            return Ok(*case);
        }
    }
    Err(Error {
        span,
        msg: format!("'{s}' is not a valid case.  See documentation for a list of cases.").into(),
        note: None,
    })
}

fn pattern_value_parser(span: Span, s: &str) -> Result<Pattern, Error> {
    const ALL_PATTERNS: &[(&str, Pattern)] = &{
        use convert_case::pattern;
        [
            ("uppercase", pattern::uppercase),
            ("lowercase", pattern::lowercase),
            ("capital", pattern::capital),
            ("camel", pattern::camel),
            ("toggle", pattern::toggle),
            ("alternating", pattern::alternating),
            ("sentence", pattern::sentence),
        ]
    };

    let pattern_str = s.to_case(Case::Flat);
    for pattern in ALL_PATTERNS {
        let pattern_in_flat = pattern.0.to_case(Case::Flat);
        if pattern_str == pattern_in_flat {
            return Ok(pattern.1);
        }
    }
    Err(Error {
        span,
        msg: format!("'{s}' is not a valid pattern.  See documentation for list of patterns.")
            .into(),
        note: None,
    })
}

#[allow(clippy::too_many_lines)]
pub fn execute_ccase(
    span: Span,
    mut args: &mut dyn Iterator<Item = TokenTree>,
    processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    const NONE: Option<(Ident, Span, String)> = None;

    let input = expect_ident_or_string(args.next_or(span))?;
    expect_punct(args.next_or(span), ',')?;

    let [mut from, mut boundaries, mut to, mut pattern, mut delimiter] = [NONE; 5];
    while let Some(tt) = args.next() {
        let arg_name = expect_ident(Ok(tt), Param::Named("arg_name"))?;
        let dest = match arg_name.to_string().as_str() {
            "f" | "from" => &mut from,
            "b" | "boundaries" => &mut boundaries,
            "t" | "to" => &mut to,
            "p" | "pattern" => &mut pattern,
            "d" | "delimiter" => &mut delimiter,
            _ => {
                return Err(Error{
                    span: arg_name.span(),
                    msg: "expected ident one of [`f`, `from`, `b`, `boundaries`, `t`, `to`, `p`, `pattern`, `d`, `delimiter`]".into(),
                    note: None,
                });
            }
        };
        if let Some((dest, _, _)) = dest.as_ref() {
            return Err(Error {
                span: arg_name.span(),
                msg: "duplicate arg_name".into(),
                note: Some(Note {
                    span: Some(dest.span()),
                    msg: "previous found here".into(),
                }),
            });
        }

        expect_punct(args.next_or(span), ':')?;
        let (arg_val, arg_span) = expect_string_literal(args.next_or(span), Param::Named("val"))?;
        if let Some(tt) = args.next() {
            expect_punct(Ok(tt), ',')?;
        }

        *dest = Some((arg_name, arg_span, arg_val));
    }

    match (
        from.as_ref(),
        boundaries.as_ref(),
        to.as_ref(),
        pattern.as_ref(),
        delimiter.as_ref(),
    ) {
        (Some((a, _, _)), Some((b, _, _)), _, _, _)
        | (_, _, Some((a, _, _)), Some((b, _, _)), _)
        | (_, _, Some((a, _, _)), _, Some((b, _, _))) => {
            return Err(Error {
                span: a.span(),
                msg: "arg conflicts with other arg".into(),
                note: Some(Note {
                    span: Some(b.span()),
                    msg: "other arg".into(),
                }),
            });
        }
        (_, _, _, None, Some((d, _, _))) => {
            return Err(Error {
                span: d.span(),
                msg: "`delimiter` requires missing argument `pattern`".into(),
                note: Some(Note {
                    span: Some(span),
                    msg: "missing arg `pattern`".into(),
                }),
            });
        }
        (_, _, None, None, _) => {
            return Err(Error {
                span,
                msg: "missing argument `to` or `pattern`".into(),
                note: None,
            });
        }
        _ => {}
    }

    // TODO: check args
    let mut conv = Converter::new();
    if let Some((_, _, boundaries)) = boundaries {
        debug_assert!(from.is_none());
        let boundaries = Boundary::defaults_from(boundaries.as_str());
        conv = conv.set_boundaries(&boundaries);
    }
    if let Some((_, span, from)) = from {
        conv = conv.from_case(case_value_parser(span, &from)?);
    }
    if let Some((_, _, delimiter)) = delimiter {
        debug_assert!(pattern.is_some());
        // --delimiter
        conv = conv.set_delim(delimiter);
    }
    if let Some((_, span, pattern)) = pattern {
        debug_assert!(to.is_none());
        conv = conv.set_pattern(pattern_value_parser(span, &pattern)?);
    }
    if let Some((_, span, to)) = to {
        conv = conv.to_case(case_value_parser(span, &to)?);
    }
    let result = match input {
        IdentOrString::Ident(ident) => {
            let result = conv.convert(ident.to_string());
            // Parse to error check
            let tokens: Vec<_> = TokenStream::from_str(&result)
                .unwrap_or_default()
                .into_iter()
                .collect();
            match tokens.as_slice() {
                [ident @ TokenTree::Ident(_)] => ident.clone(),
                _ => {
                    return Err(Error {
                        span,
                        msg: format!("`{result}` is not a valid identifier.").into(),
                        note: None,
                    });
                }
            }
        }
        IdentOrString::String(string) => {
            let result = conv.convert(&string);
            pm::Literal::string(&result).into()
        }
    };
    processed_out.push(result);

    Ok(())
}
