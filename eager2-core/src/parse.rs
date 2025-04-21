use std::{fmt::Write, iter::Peekable, ops::RangeInclusive, sync::OnceLock};

use crate::{
    consts::{EAGER_SIGIL, LAZY_SIGIL},
    pm::{token_stream, Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenTree},
    state::Mode,
    Error, Note,
};

pub type ExpectCallLiteral = fn(Result<TokenTree, Span>) -> bool;
pub type ExpectStringLiteral =
    fn(Result<TokenTree, Span>, Param<&str>) -> Result<(String, Span), Error>;
pub type GetStringLiteral = fn(Literal) -> Option<String>;

pub struct Fns {
    pub expect_call_literal: ExpectCallLiteral,
    pub expect_string_literal: ExpectStringLiteral,
    pub get_string_literal: GetStringLiteral,
}

pub static FNS: OnceLock<Fns> = OnceLock::new();

pub fn expect_call_literal(tt: Result<TokenTree, Span>) -> bool {
    (FNS.get().unwrap().expect_call_literal)(tt)
}

pub enum Param<'a, V> {
    Named(&'a str),
    ExactValue(V),
}

impl<V> From<V> for Param<'_, V> {
    fn from(v: V) -> Self {
        Self::ExactValue(v)
    }
}
impl<'a> From<&'a str> for Param<'a, Delimiter> {
    fn from(v: &'a str) -> Self {
        Self::Named(v)
    }
}
impl<'a> From<&'a str> for Param<'a, Mode> {
    fn from(v: &'a str) -> Self {
        Self::Named(v)
    }
}
pub trait IsPunct {
    fn as_char(&self) -> char;
    fn is_punct(&self, p: &Punct) -> bool;
}

impl IsPunct for char {
    fn as_char(&self) -> char {
        *self
    }
    fn is_punct(&self, p: &Punct) -> bool {
        p.as_char() == *self
    }
}

impl IsPunct for (char, Spacing) {
    fn as_char(&self) -> char {
        self.0
    }
    fn is_punct(&self, p: &Punct) -> bool {
        p.as_char() == self.0 && p.spacing() == self.1
    }
}

#[must_use]
pub fn eat_zero_group(tt: TokenTree) -> TokenTree {
    let orig_tt = tt;
    let mut tt = orig_tt.clone();
    loop {
        match tt {
            TokenTree::Group(g) if g.delimiter() == Delimiter::None => {
                let mut stream = g.stream().into_iter();
                let Some(next_tt) = stream.next() else {
                    return orig_tt;
                };
                if stream.next().is_some() {
                    return orig_tt;
                }
                tt = next_tt;
            }
            tt => return tt,
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
pub fn expect_punct(tt: Result<TokenTree, Span>, c: impl IsPunct) -> Result<Punct, Error> {
    expect_punct_impl(tt, &c)
}
fn expect_punct_impl(tt: Result<TokenTree, Span>, c: &dyn IsPunct) -> Result<Punct, Error> {
    match tt.map(eat_zero_group) {
        Err(span) => Err(Error {
            span,
            msg: "unexpected end of macro invocation".into(),
            note: Some(Note {
                span: None,
                msg: format!("while trying to match token `{}`", c.as_char()).into(),
            }),
        }),
        Ok(TokenTree::Punct(p)) if c.is_punct(&p) => Ok(p),
        Ok(tt) => Err(Error {
            span: tt.span(),
            msg: format!("expected token: `{}`", c.as_char()).into(),
            note: None,
        }),
    }
}

pub fn expect_ident<'a, 'b>(
    tt: Result<TokenTree, Span>,
    s: impl Into<Param<'a, &'b str>>,
) -> Result<Ident, Error> {
    expect_ident_impl(tt, s.into())
}

fn expect_ident_impl(tt: Result<TokenTree, Span>, s: Param<&str>) -> Result<Ident, Error> {
    match (tt.map(eat_zero_group), s) {
        (Err(span), Param::ExactValue(s)) => Err(Error {
            span,
            msg: "unexpected end of macro invocation".into(),
            note: Some(Note {
                span: None,
                msg: format!("while trying to match ident `{s}`").into(),
            }),
        }),
        (Err(span), Param::Named(s)) => Err(Error {
            span,
            msg: "unexpected end of macro invocation".into(),
            note: Some(Note {
                span: None,
                msg: format!("while trying to match ident `${s}:ident`").into(),
            }),
        }),
        (Ok(TokenTree::Ident(i)), Param::ExactValue(s)) if i.to_string() == s => Ok(i),
        (Ok(TokenTree::Ident(i)), Param::Named(_)) => Ok(i),
        (Ok(tt), Param::ExactValue(s)) => Err(Error {
            span: tt.span(),
            msg: format!("expected ident: `{s}`").into(),
            note: None,
        }),
        (Ok(tt), Param::Named(s)) => Err(Error {
            span: tt.span(),
            msg: format!("expected ident: `${s}:ident`").into(),
            note: None,
        }),
    }
}

pub fn expect_group<'a>(
    tt: Result<TokenTree, Span>,
    d: impl Into<Param<'a, Delimiter>>,
) -> Result<Group, Error> {
    expect_group_impl(tt, d.into())
}

fn expect_group_impl(tt: Result<TokenTree, Span>, d: Param<Delimiter>) -> Result<Group, Error> {
    fn to_char(d: Delimiter) -> char {
        match d {
            Delimiter::Parenthesis => '(',
            Delimiter::Brace => '{',
            Delimiter::Bracket => '[',
            Delimiter::None => panic!("∅ should never be used"),
        }
    }
    match (tt.map(eat_zero_group), d) {
        (Err(span), Param::ExactValue(d)) => Err(Error {
            span,
            msg: "unexpected end of macro invocation".into(),
            note: Some(Note {
                span: None,
                msg: format!("while trying to match ident `{}`", to_char(d)).into(),
            }),
        }),
        (Err(span), Param::Named(g)) => Err(Error {
            span,
            msg: "unexpected end of macro invocation".into(),
            note: Some(Note {
                span: None,
                msg: format!("while trying to match ident `${g}:group`").into(),
            }),
        }),
        (Ok(TokenTree::Group(g)), Param::ExactValue(d)) if g.delimiter() == d => Ok(g),
        (Ok(TokenTree::Group(g)), Param::Named(_)) if g.delimiter() != Delimiter::None => Ok(g),
        (Ok(tt), Param::ExactValue(d)) => Err(Error {
            span: tt.span(),
            msg: format!("expected token: `{}`", to_char(d)).into(),
            note: None,
        }),
        (Ok(tt), Param::Named(g)) => Err(Error {
            span: tt.span(),
            msg: format!("expected one of `(`, `[`, or `{{` for `${g}:group`").into(),
            note: None,
        }),
    }
}

pub enum IdentOrString {
    Ident(Ident),
    String(String),
}

pub fn expect_ident_or_string(tt: Result<TokenTree, Span>) -> Result<IdentOrString, Error> {
    let tt = match tt.map(eat_zero_group) {
        Err(span) => {
            return Err(Error {
                span,
                msg: "unexpected end of macro invocation".into(),
                note: Some(Note {
                    span: None,
                    msg: "while trying to match ident or string literal `$input`".into(),
                }),
            })
        }
        Ok(TokenTree::Ident(i)) => return Ok(IdentOrString::Ident(i)),
        Ok(TokenTree::Literal(l)) => {
            if let Some(s) = (FNS.get().unwrap().get_string_literal)(l.clone()) {
                return Ok(IdentOrString::String(s));
            }
            TokenTree::Literal(l)
        }

        Ok(tt) => tt,
    };
    Err(Error {
        span: tt.span(),
        msg: "expected ident or string literal".into(),
        note: None,
    })
}

pub fn expect_string_literal<'a, 'b>(
    tt: Result<TokenTree, Span>,
    s: impl Into<Param<'a, &'b str>>,
) -> Result<(String, Span), Error> {
    (FNS.get().unwrap().expect_string_literal)(tt, s.into())
}

pub fn expect_mode<'a>(
    tt: Result<TokenTree, Span>,
    s: impl Into<Param<'a, Mode>>,
) -> Result<Mode, Error> {
    expect_mode_impl(tt, s.into())
}

fn expect_mode_impl(tt: Result<TokenTree, Span>, s: Param<Mode>) -> Result<Mode, Error> {
    match (tt.map(eat_zero_group), s) {
        (Err(span), Param::ExactValue(s)) => Err(Error {
            span,
            msg: "unexpected end of macro invocation".into(),
            note: Some(Note {
                span: None,
                msg: format!("while trying to match ident `{s:?}`").into(),
            }),
        }),
        (Err(span), Param::Named(s)) => Err(Error {
            span,
            msg: "unexpected end of macro invocation".into(),
            note: Some(Note {
                span: None,
                msg: format!("while trying to match ident `${s}:ident`").into(),
            }),
        }),
        (Ok(TokenTree::Ident(i)), Param::ExactValue(s)) if i.to_string() == s.sigil() => Ok(s),
        (Ok(TokenTree::Ident(i)), Param::Named(n)) => match i.to_string().as_str() {
            EAGER_SIGIL => Ok(Mode::Eager),
            LAZY_SIGIL => Ok(Mode::Lazy),
            _ => Err(Error {
                span: i.span(),
                msg: format!("expected ident `{EAGER_SIGIL}` or `{LAZY_SIGIL}` for `${n}:ident`")
                    .into(),
                note: None,
            }),
        },
        (Ok(tt), Param::ExactValue(s)) => Err(Error {
            span: tt.span(),
            msg: format!("expected ident: `{s:?}`").into(),
            note: None,
        }),
        (Ok(tt), Param::Named(s)) => Err(Error {
            span: tt.span(),
            msg: format!("expected ident: `${s}:ident`").into(),
            note: None,
        }),
    }
}

#[derive(Debug)]
pub enum MacroPathType {
    /// `macro_name + "!"`
    UnpathedMacro { macro_name: String },

    /// `"$crate::" + macro_name + "!"`
    DollarCrateRootMacro { macro_name: String },

    /// `"::" | "" + "eager2::" + macro_name + "!"`
    Eager2CrateRootMacro { global: bool, macro_name: String },

    /// `"::" | "" + crate_name + "::" + macro_name + "!"`
    CrateRootMacro {
        global: bool,
        /// Can be `"$crate"` or the crate name, but can't be `"eager2"`
        crate_name: String,
        macro_name: String,
    },

    /// `"$crate::eager2::" + macro_name + "!"`
    ReExportMacro { macro_name: String },
}

pub struct MacroPathSegments<'a> {
    pub tokens: &'a [TokenTree],
    pub ty: Option<MacroPathType>,
}

fn is_empty(t: &mut token_stream::IntoIter) -> bool {
    for tt in t {
        match tt {
            TokenTree::Group(g) if is_empty(&mut g.stream().into_iter()) => {}
            _ => return false,
        }
    }
    true
}

/// `None` for not a path group
/// `Some(true)` for path group and found the ident
/// `Some(false)` for path group but no ident (only `$` and empty groups)
fn is_path_group(mut g: Group) -> Option<bool> {
    loop {
        if g.delimiter() != Delimiter::None {
            return None;
        }
        let mut tokens = g.stream().into_iter();
        loop {
            match tokens.next() {
                Some(TokenTree::Ident(_)) if is_empty(&mut tokens) => return Some(true),
                Some(TokenTree::Group(g)) if g.delimiter() != Delimiter::None => return None,
                Some(TokenTree::Group(g2)) if is_empty(&mut g2.stream().into_iter()) => {}
                Some(TokenTree::Group(g2)) if is_empty(&mut tokens) => {
                    g = g2;
                    break;
                }
                Some(TokenTree::Punct(p)) if p.as_char() == '$' => {}
                None => return Some(false),
                _ => return None,
            }
        }
    }
}

fn is_all_dollar(tt: TokenTree) -> bool {
    match tt {
        TokenTree::Punct(p) => p.as_char() == '$',
        TokenTree::Group(g) => {
            g.delimiter() == Delimiter::None && g.stream().into_iter().all(is_all_dollar)
        }
        _ => false,
    }
}

fn get_next_path_segment<'a>(
    iter: &mut Peekable<impl Iterator<Item = (usize, &'a TokenTree)>>,
) -> Option<RangeInclusive<usize>> {
    let end = iter.peek().map_or(0, |(i, _)| *i);
    match iter.next()?.1 {
        TokenTree::Ident(_) => {}
        TokenTree::Group(g) if is_path_group(g.clone())? => {}
        // Path segments have to end in an ident-like
        _ => return None,
    }

    // We allow any number of dollar signs before that
    loop {
        match iter.peek().map(|&(i, tt)| (i, is_all_dollar(tt.clone()))) {
            Some((_, true)) => {}
            Some((i, false)) => return Some(i + 1..=end),
            None => return Some(0..=end),
        }
        iter.next();
    }
}

fn path_segment_to_string_helper(buffer: &mut String, tt: TokenTree) {
    match tt {
        TokenTree::Punct(p) => buffer.push(p.as_char()),
        TokenTree::Ident(i) => write!(buffer, "{i}").unwrap(),
        TokenTree::Group(g) => {
            debug_assert_eq!(g.delimiter(), Delimiter::None);
            for tt in g.stream() {
                path_segment_to_string_helper(buffer, tt);
            }
        }
        TokenTree::Literal(_) => unreachable!(),
    }
}

fn path_segment_to_string(tokens: &[TokenTree]) -> String {
    let mut buffer = String::new();
    for tt in tokens {
        path_segment_to_string_helper(&mut buffer, tt.clone());
    }
    buffer
}

impl<'a> MacroPathSegments<'a> {
    #[allow(clippy::too_many_lines)]
    #[must_use]
    pub fn try_new(tokens: &'a [TokenTree]) -> Option<Self> {
        let mut iter = tokens.iter().enumerate().rev();
        match iter.next()?.1 {
            TokenTree::Punct(p) if p.as_char() == '!' => {}
            // No exclamation
            _ => return None,
        }

        let mut iter = iter.peekable();

        // Get macro name
        let macro_name_i = get_next_path_segment(&mut iter)?;
        let macro_name = &tokens[macro_name_i.clone()];

        // Get colons
        match (iter.next(), iter.next()) {
            (Some((_, TokenTree::Punct(p2))), Some((_, TokenTree::Punct(p1))))
                if p1.as_char() == ':' && p1.spacing() == Spacing::Joint && p2.as_char() == ':' => {
            }
            _ => {
                return Some(MacroPathSegments {
                    tokens: &tokens[*macro_name_i.start()..],
                    ty: Some(MacroPathType::UnpathedMacro {
                        macro_name: path_segment_to_string(macro_name),
                    }),
                })
            }
        }

        // Get the first scope name
        let Some(scope_name_i) = get_next_path_segment(&mut iter) else {
            // Extern prelude with no crate name so we default to unpathed
            // and ignore the leading `::`
            return Some(MacroPathSegments {
                tokens: macro_name,
                ty: Some(MacroPathType::UnpathedMacro {
                    macro_name: path_segment_to_string(macro_name),
                }),
            });
        };
        let scope_name = &tokens[scope_name_i.clone()];

        // Get colons
        let global_i = match (iter.next(), iter.next()) {
            (Some((_, TokenTree::Punct(p2))), Some((i, TokenTree::Punct(p1))))
                if p1.as_char() == ':' && p1.spacing() == Spacing::Joint && p2.as_char() == ':' =>
            {
                i
            }
            _ => {
                return Some(MacroPathSegments {
                    tokens: &tokens[*scope_name_i.start()..],
                    ty: Some({
                        let crate_name = path_segment_to_string(scope_name);
                        let macro_name = path_segment_to_string(macro_name);

                        match crate_name.as_str() {
                            "$crate" => MacroPathType::DollarCrateRootMacro { macro_name },
                            "eager2" => MacroPathType::Eager2CrateRootMacro {
                                global: false,
                                macro_name,
                            },
                            _ => MacroPathType::CrateRootMacro {
                                global: false,
                                crate_name,
                                macro_name,
                            },
                        }
                    }),
                })
            }
        };

        // Get the second scope
        let Some(crate_name_i) = get_next_path_segment(&mut iter) else {
            return Some(MacroPathSegments {
                tokens: &tokens[global_i..],
                ty: Some({
                    let crate_name = path_segment_to_string(scope_name);
                    let macro_name = path_segment_to_string(macro_name);

                    if crate_name == "eager2" {
                        MacroPathType::Eager2CrateRootMacro {
                            global: true,
                            macro_name,
                        }
                    } else {
                        MacroPathType::CrateRootMacro {
                            global: true,
                            crate_name,
                            macro_name,
                        }
                    }
                }),
            });
        };
        let crate_name = &tokens[crate_name_i.clone()];

        // Get colons
        let mut global_i = match (iter.next(), iter.next()) {
            (Some((_, TokenTree::Punct(p2))), Some((i, TokenTree::Punct(p1))))
                if p1.as_char() == ':' && p1.spacing() == Spacing::Joint && p2.as_char() == ':' =>
            {
                i
            }
            _ => {
                let start = *crate_name_i.start();
                let crate_name = path_segment_to_string(crate_name);
                let scope_name = path_segment_to_string(scope_name);
                if crate_name == "$crate" && scope_name == "eager2" {
                    return Some(MacroPathSegments {
                        tokens: &tokens[start..],
                        ty: Some(MacroPathType::ReExportMacro {
                            macro_name: path_segment_to_string(macro_name),
                        }),
                    });
                }
                return Some(MacroPathSegments {
                    tokens: &tokens[start..],
                    ty: None,
                });
            }
        };

        // Continue getting scopes and colons
        loop {
            let Some(crate_name_i) = get_next_path_segment(&mut iter) else {
                return Some(MacroPathSegments {
                    tokens: &tokens[global_i..],
                    ty: None,
                });
            };

            global_i = match (iter.next(), iter.next()) {
                (Some((_, TokenTree::Punct(p2))), Some((i, TokenTree::Punct(p1))))
                    if p1.as_char() == ':'
                        && p1.spacing() == Spacing::Joint
                        && p2.as_char() == ':' =>
                {
                    i
                }
                _ => {
                    return Some(MacroPathSegments {
                        tokens: &tokens[*crate_name_i.start()..],
                        ty: None,
                    })
                }
            };
        }
    }
}
