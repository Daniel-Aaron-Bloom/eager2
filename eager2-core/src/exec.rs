use std::{
    env, fs, io,
    path::{self, Path, PathBuf},
    str::FromStr,
    sync::OnceLock,
};

use crate::{
    consts::eager_call_sigil,
    egroup::EfficientGroupV,
    parse::{
        eat_zero_group, expect_group, expect_ident, expect_punct, expect_string_literal, Param,
    },
    pm::{token_stream, Delimiter, Group, Ident, Literal, Span, TokenStream, TokenTree},
    rules::expand_rules_legacy,
    utils::{NextOr, PopNext},
    Error,
};

pub type ExecProccessed =
    fn(Span, &mut dyn Iterator<Item = TokenTree>, &mut EfficientGroupV) -> Result<(), Error>;

pub struct Fns {
    pub execute_concat: ExecProccessed,
    pub execute_ccase: ExecProccessed,
}

pub static FNS: OnceLock<Fns> = OnceLock::new();

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExecutableMacroType {
    Cfg,
    Column,
    CompileError,
    Concat,
    Env,
    File,
    Include,
    IncludeBytes,
    IncludeStr,
    Line,
    ModulePath,
    OptionEnv,
    Stringify,

    CCase,
    EagerCoalesce,
    EagerIf,
    EagerMacroRules,
    TokenEq,
    Unstringify,
}

impl TryFrom<&str> for ExecutableMacroType {
    type Error = ();
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "cfg" => Self::Cfg,
            "column" => Self::Column,
            "compile_error" => Self::CompileError,
            "concat" => Self::Concat,
            "env" => Self::Env,
            "file" => Self::File,
            "include" => Self::Include,
            "include_bytes" => Self::IncludeBytes,
            "include_str" => Self::IncludeStr,
            "line" => Self::Line,
            "module_path" => Self::ModulePath,
            "option_env" => Self::OptionEnv,
            "stringify" => Self::Stringify,

            "ccase" => Self::CCase,
            "eager_coalesce" => Self::EagerCoalesce,
            "eager_if" => Self::EagerIf,
            "eager_macro_rules" => Self::EagerMacroRules,
            "token_eq" => Self::TokenEq,
            "unstringify" => Self::Unstringify,

            _ => return Err(()),
        })
    }
}

impl ExecutableMacroType {
    pub fn execute(
        self,
        span: Span,
        stream: impl IntoIterator<Item = TokenTree>,
        processed: &mut EfficientGroupV,
        unprocessed: &mut Vec<token_stream::IntoIter>,
    ) -> Result<(), Error> {
        self.execute_impl(span, &mut stream.into_iter(), processed, unprocessed)
    }

    fn execute_impl(
        self,
        span: Span,
        args: &mut dyn Iterator<Item = TokenTree>,
        processed: &mut EfficientGroupV,
        unprocessed: &mut Vec<token_stream::IntoIter>,
    ) -> Result<(), Error> {
        match self {
            Self::Cfg => execute_cfg(span, args, processed),
            Self::CompileError => Err(execute_compile_error(span, args)?),
            Self::Concat => {
                let fexecs = FNS.get().unwrap();
                (fexecs.execute_concat)(span, args, processed)
            }
            Self::Column => execute_column(span, args, processed),
            Self::Env => execute_env(span, args, processed),
            Self::File => execute_file(span, args, processed),
            Self::Stringify => {
                execute_stringify(args, processed);
                Ok(())
            }
            Self::Include => execute_include(span, args, unprocessed),
            Self::IncludeBytes => execute_include_bytes(span, args, processed),
            Self::IncludeStr => execute_include_str(span, args, processed),
            Self::Line => execute_line(span, args, processed),
            Self::ModulePath => execute_module_path(span, args, processed),
            Self::OptionEnv => execute_option_env(span, args, processed),

            Self::CCase => {
                let fexecs = FNS.get().unwrap();
                (fexecs.execute_ccase)(span, args, processed)
            }
            Self::EagerCoalesce => execute_eager_coalesce(span, args, processed),
            Self::EagerIf => execute_eager_if(span, args, unprocessed),
            Self::EagerMacroRules => execute_eager_macro_rules(span, args, processed),
            Self::TokenEq => execute_token_eq(span, args, processed),
            Self::Unstringify => execute_unstringify(span, args, unprocessed),
        }
    }
}

fn execute_cfg(
    span: Span,
    mut _args: &mut dyn Iterator<Item = TokenTree>,
    _processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    Err(Error {
        span,
        msg: "eager `cfg!` is not implemented yet.".into(),
        note: None,
    })
}

fn execute_compile_error(
    span: Span,
    mut args: &mut dyn Iterator<Item = TokenTree>,
) -> Result<Error, Error> {
    let (msg, _) = expect_string_literal(args.next_or(span), Param::Named("msg"))?;
    if let Some(tt) = args.next() {
        expect_punct(Ok(tt), ',')?;
    }

    if args.next().is_some() {
        return Err(Error {
            span,
            msg: "`compile_error!()` takes 1 arguments".into(),
            note: None,
        });
    }

    Ok(Error {
        span,
        msg: msg.into(),
        note: None,
    })
}

fn execute_column(
    span: Span,
    mut _args: &mut dyn Iterator<Item = TokenTree>,
    _processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    Err(Error {
        span,
        msg: "eager `column!` is not implemented yet.".into(),
        note: None,
    })
}

fn execute_env(
    span: Span,
    mut args: &mut dyn Iterator<Item = TokenTree>,
    processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    let (key, _) = expect_string_literal(args.next_or(span), Param::Named("key"))?;
    let error = if let Some(tt) = args.next() {
        expect_punct(Ok(tt), ',')?;
        if let Some(tt) = args.next() {
            let error = expect_string_literal(Ok(tt), Param::Named("error"))?;
            if let Some(tt) = args.next() {
                expect_punct(Ok(tt), ',')?;
            }
            Some(error)
        } else {
            None
        }
    } else {
        None
    };

    if args.next().is_some() {
        return Err(Error {
            span,
            msg: "`env!()` takes 1 or 2 arguments".into(),
            note: None,
        });
    }

    let value = match (env::var(&key), error) {
        (Ok(value), _) => Ok(value),
        (Err(_), Some((error, _))) => Err(Error {
            span,
            msg: error.into(),
            note: None,
        }),
        (Err(env::VarError::NotPresent), None) => Err(Error {
            span,
            msg: format!("environment variable `{key}` not defined at compile time").into(),
            note: None,
        }),
        (Err(env::VarError::NotUnicode(_)), None) => Err(Error {
            span,
            msg: format!(
                "environment variable `{key}` was present but not unicode at compile time"
            )
            .into(),
            note: None,
        }),
    }?;
    processed_out.push(TokenTree::Literal(Literal::string(&value)));
    Ok(())
}

fn execute_option_env(
    span: Span,
    mut args: &mut dyn Iterator<Item = TokenTree>,
    processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    let (key, _) = expect_string_literal(args.next_or(span), Param::Named("key"))?;
    if let Some(tt) = args.next() {
        expect_punct(Ok(tt), ',')?;
    }
    if args.next().is_some() {
        return Err(Error {
            span,
            msg: "`option_env!()` takes 1 arguments".into(),
            note: None,
        });
    }

    match env::var(&key) {
        Err(env::VarError::NotUnicode(_)) => {
            return Err(Error {
                span,
                msg: format!(
                    "environment variable `{key}` was present but not unicode at compile time"
                )
                .into(),
                note: None,
            })
        }
        Ok(value) => {
            let val = TokenTree::Literal(Literal::string(&value));
            let some = Ident::new("Some", span);
            let group = Group::new(Delimiter::Parenthesis, val.into());
            processed_out.push(some.into());
            processed_out.push(group.into());
        }
        Err(env::VarError::NotPresent) => processed_out.push(Ident::new("None", span).into()),
    }

    Ok(())
}

fn execute_file(
    span: Span,
    mut _args: &mut dyn Iterator<Item = TokenTree>,
    _processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    Err(Error {
        span,
        msg: "eager `file!` is not implemented yet.".into(),
        note: None,
    })
}

fn execute_stringify(
    args: &mut dyn Iterator<Item = TokenTree>,
    processed_out: &mut EfficientGroupV,
) {
    let s = args.collect::<TokenStream>().to_string();
    processed_out.push(TokenTree::Literal(Literal::string(&s)));
}

fn include_helper<R: 'static>(
    span: Span,
    mut args: &mut dyn Iterator<Item = TokenTree>,
    f: impl FnOnce(&Path) -> io::Result<R>,
) -> Result<R, Error> {
    let (file, file_span) = expect_string_literal(args.next_or(span), Param::Named("file"))?;
    if let Some(tt) = args.next() {
        expect_punct(Ok(tt), ',')?;
    }

    let mut path = Path::new(&file);
    if path.is_relative() {
        return Err(Error{
            span: file_span,
            msg: r#"relative path is not supported here; use `include!(concat!(env!("CARGO_MANIFEST_DIR"), ...))"#.into(),
            note: None,
        });
    }

    // Make Windows verbatim paths work even with mixed path separators, which
    // can happen when a path is produced using `concat!`.
    let path_buf: PathBuf;
    if let Some(path::Component::Prefix(prefix)) = path.components().next() {
        if prefix.kind().is_verbatim() {
            path_buf = path.components().collect();
            path = &path_buf;
        }
    }

    f(path).map_err(|err| Error {
        span,
        msg: format!("Couldn't read {}: {err:?}", path.display()).into(),
        note: None,
    })
}

fn execute_include(
    span: Span,
    args: &mut dyn Iterator<Item = TokenTree>,
    unprocessed: &mut Vec<token_stream::IntoIter>,
) -> Result<(), Error> {
    let parsed = include_helper(span, args, |path| {
        let content = fs::read_to_string(path)?;
        let parsed = TokenStream::from_str(&content).map_err(|err| Error {
            span,
            msg: format!("Couldn't parse file {}: {err:?}", path.display()).into(),
            note: None,
        });
        Ok(parsed)
    })??;

    unprocessed.push(parsed.into_iter());
    Ok(())
}

fn execute_include_str(
    span: Span,
    args: &mut dyn Iterator<Item = TokenTree>,
    processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    let content = include_helper(span, args, |p| fs::read_to_string(p))?;
    let string = Literal::string(&content);

    processed_out.push(string.into());
    Ok(())
}

fn execute_include_bytes(
    span: Span,
    args: &mut dyn Iterator<Item = TokenTree>,
    processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    let content = include_helper(span, args, |p| fs::read(p))?;
    let string = Literal::byte_string(&content);

    processed_out.push(string.into());
    Ok(())
}

fn execute_line(
    span: Span,
    _args: &mut dyn Iterator<Item = TokenTree>,
    _processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    Err(Error {
        span,
        msg: "eager `line!` is not implemented yet.".into(),
        note: None,
    })
}

fn execute_module_path(
    span: Span,
    _args: &mut dyn Iterator<Item = TokenTree>,
    _processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    Err(Error {
        span,
        msg: "eager `module_path!` is not implemented yet.".into(),
        note: None,
    })
}

fn execute_eager_coalesce(
    span: Span,
    mut args: &mut dyn Iterator<Item = TokenTree>,
    processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    let mut looking = true;
    loop {
        let group = expect_group(args.next_or(span), Param::Named("arg"))?.stream();
        if looking && !group.is_empty() {
            // Process the rest for syntax check
            looking = false;
            processed_out.as_mut_vec().extend(group);
        }
        if let Some(comma) = args.next() {
            expect_punct(Ok(comma), ',')?;
        } else {
            break;
        }
    }

    Ok(())
}

fn execute_eager_macro_rules(
    span: Span,
    args: &mut dyn Iterator<Item = TokenTree>,
    processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    let eager_call_sigil = eager_call_sigil();
    let output = expand_rules_legacy(span, &eager_call_sigil, args)?;
    processed_out.extend(output);
    Ok(())
}

fn execute_eager_if(
    span: Span,
    mut args: &mut dyn Iterator<Item = TokenTree>,
    unprocessed: &mut Vec<token_stream::IntoIter>,
) -> Result<(), Error> {
    let check = expect_ident(args.next_or(span), Param::Named("check"))?;
    if let Some(tt) = args.next() {
        expect_punct(Ok(tt), ',')?;
    }

    if args.next().is_some() {
        return Err(Error {
            span,
            msg: "`eager_if!()` takes 1 argument".into(),
            note: None,
        });
    }

    let check = match check.to_string().as_str() {
        "true" => true,
        "false" => false,
        _ => {
            return Err(Error {
                span: check.span(),
                msg: "expected either token `true` or token `false`".into(),
                note: None,
            })
        }
    };

    let true_case = expect_group(
        unprocessed.pop_next().ok_or(span),
        Param::Named("true_case"),
    )?;

    let false_case = expect_group(
        unprocessed.pop_next().ok_or(span),
        Param::Named("false_case"),
    )?;

    let output = if check { true_case } else { false_case }.stream();

    unprocessed.push(output.into_iter());

    Ok(())
}

fn execute_token_eq(
    span: Span,
    mut args: &mut dyn Iterator<Item = TokenTree>,
    processed_out: &mut EfficientGroupV,
) -> Result<(), Error> {
    struct TtWrapper(TokenTree);
    impl PartialEq for TtWrapper {
        fn eq(&self, other: &Self) -> bool {
            match (
                eat_zero_group(self.0.clone()),
                eat_zero_group(other.0.clone()),
            ) {
                (TokenTree::Group(a), TokenTree::Group(b)) => group_eq(&a, &b),
                (TokenTree::Ident(a), TokenTree::Ident(b)) => a.to_string() == b.to_string(),
                (TokenTree::Punct(a), TokenTree::Punct(b)) => a.as_char() == b.as_char(),
                (TokenTree::Literal(a), TokenTree::Literal(b)) => a.to_string() == b.to_string(),
                _ => false,
            }
        }
    }
    fn stream_eq(a: TokenStream, b: TokenStream) -> bool {
        a.is_empty() == b.is_empty()
            && a.into_iter()
                .map(TtWrapper)
                .eq(b.into_iter().map(TtWrapper))
    }
    fn group_eq(a: &Group, b: &Group) -> bool {
        a.delimiter() == b.delimiter() && stream_eq(a.stream(), b.stream())
    }

    let mut prev = None;
    for i in 0.. {
        let name = format!("arg_{i}");
        let next = expect_group(args.next_or(span), Param::Named(&name))?;
        if let Some(prev) = prev.take() {
            if !group_eq(&prev, &next) {
                processed_out.push(Ident::new("false", span).into());
                return Ok(());
            }
        }
        prev = Some(next);

        if let Some(comma) = args.next() {
            expect_punct(Ok(comma), ',')?;
        } else {
            break;
        }
    }
    processed_out.push(Ident::new("true", span).into());

    Ok(())
}

fn execute_unstringify(
    span: Span,
    mut args: &mut dyn Iterator<Item = TokenTree>,
    unprocessed: &mut Vec<token_stream::IntoIter>,
) -> Result<(), Error> {
    let (src, _) = expect_string_literal(args.next_or(span), Param::Named("src"))?;
    if let Some(tt) = args.next() {
        expect_punct(Ok(tt), ',')?;
    }

    let unstrung = TokenStream::from_str(&src).unwrap();

    unprocessed.push(unstrung.into_iter());
    Ok(())
}
