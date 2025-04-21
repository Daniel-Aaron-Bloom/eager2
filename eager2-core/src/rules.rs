use std::iter::Peekable;

use crate::{
    consts::{crate_path, eager_call_sigil, get_eager_2_ident},
    parse::{expect_group, expect_ident, expect_punct, Param},
    pm::{
        Delimiter, Group, Ident, Literal, Punct, Spacing, Span, ToTokens, TokenStream, TokenTree,
    },
    state::State,
    utils::{eager_data, NextOr},
    Error,
};

#[derive(Clone)]
struct Rule {
    grammar: Group,
    expansion: Group,
}

struct HiddenIdent<'a>(bool, &'a Ident);
impl ToTokens for HiddenIdent<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        Punct::new('$', Spacing::Joint).to_tokens(tokens);
        self.1.to_tokens(tokens);
        if self.0 {
            Punct::new(':', Spacing::Joint).to_tokens(tokens);
            Ident::new("tt", Span::call_site()).to_tokens(tokens);
        }
    }
}

struct HiddenIdentSet<'a>(bool, &'a Ident);
impl ToTokens for HiddenIdentSet<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        Punct::new('$', Spacing::Joint).to_tokens(tokens);
        Group::new(
            Delimiter::Parenthesis,
            HiddenIdent(self.0, self.1).into_token_stream(),
        )
        .to_tokens(tokens);
        Punct::new('*', Spacing::Alone).to_tokens(tokens);
    }
}

fn eager_rule(
    grammar: bool,
    eager_call_sigil: &Literal,
    hidden_ident: &Ident,
    tail: TokenStream,
) -> TokenStream {
    eager_data(
        eager_call_sigil,
        HiddenIdentSet(grammar, hidden_ident).into_token_stream(),
        tail,
    )
}

impl Rule {
    fn make_eager(
        mut self,
        crate_path: &TokenStream,
        eager_call_sigil: &Literal,
        hidden_ident: &Ident,
    ) -> Self {
        self.grammar = Group::new(
            self.grammar.delimiter(),
            eager_rule(true, eager_call_sigil, hidden_ident, self.grammar.stream()),
        );

        let mut expansion = TokenStream::new();
        expansion.extend(crate_path.clone());
        Punct::new(':', Spacing::Joint).to_tokens(&mut expansion);
        Punct::new(':', Spacing::Joint).to_tokens(&mut expansion);
        Ident::new("eager", Span::call_site()).to_tokens(&mut expansion);
        Punct::new('!', Spacing::Alone).to_tokens(&mut expansion);
        Group::new(
            Delimiter::Brace,
            eager_rule(
                false,
                eager_call_sigil,
                hidden_ident,
                self.expansion.stream(),
            ),
        )
        .to_tokens(&mut expansion);

        self.expansion = Group::new(self.expansion.delimiter(), expansion);
        self
    }
}

impl ToTokens for Rule {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.grammar.to_tokens(tokens);
        Punct::new('=', Spacing::Joint).to_tokens(tokens);
        Punct::new('>', Spacing::Alone).to_tokens(tokens);
        self.expansion.to_tokens(tokens);
        Punct::new(';', Spacing::Alone).to_tokens(tokens);
    }
}
pub struct Rules {
    metas: Vec<TokenTree>,
    macro_name: Ident,
    eager: Vec<Rule>,
    pure: Vec<Rule>,
}
impl ToTokens for Rules {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.metas.iter().cloned());
        Ident::new("macro_rules", Span::call_site()).to_tokens(tokens);
        Punct::new('!', Spacing::Alone).to_tokens(tokens);
        self.macro_name.to_tokens(tokens);

        let mut rules = TokenStream::new();
        for rule in &self.eager {
            rule.to_tokens(&mut rules);
        }
        // Put the pure version after so eager is always tried first
        for rule in &self.pure {
            rule.to_tokens(&mut rules);
        }
        Group::new(Delimiter::Brace, rules).to_tokens(tokens);
    }
}

pub fn expand_rules(
    span: Span,
    hidden_ident: &Ident,
    crate_path: &TokenStream,
    eager_call_sigil: &Literal,
    stream: &mut Peekable<&mut dyn Iterator<Item = TokenTree>>,
) -> Result<Rules, Error> {
    let mut metas = vec![];
    loop {
        match stream.peek() {
            None => {
                return Err(Error {
                    span,
                    msg: "unexpected end of macro invocation".into(),
                    note: Some(crate::Note {
                        span: None,
                        msg: "while trying to match token `#` or ident `macro_rules`".into(),
                    }),
                })
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '#' => {
                let pound = stream.next().unwrap();
                metas.push(pound);

                let g = expect_group(stream.next_or(span), Delimiter::Bracket)?;
                metas.push(g.into());
            }
            Some(TokenTree::Ident(i)) if i.to_string() == "macro_rules" => break,
            Some(t) => {
                return Err(Error {
                    span: t.span(),
                    msg: "expected token `#` or or ident `macro_rules`".into(),
                    note: None,
                })
            }
        }
    }
    let _macro_rules = stream.next().unwrap();
    expect_punct(stream.next_or(span), '!')?;
    let macro_name = expect_ident(stream.next_or(span), Param::Named("macro_name"))?;

    let group = expect_group(stream.next_or(span), Delimiter::Brace)?;

    let mut rules = vec![];
    let span = group.span();
    let mut stream = group.stream().into_iter();
    loop {
        let Some(tt) = stream.next() else { break };
        let grammar = expect_group(Ok(tt), Param::Named("grammar"))?;

        // Arrow
        expect_punct(stream.next_or(span), ('=', Spacing::Joint))?;
        expect_punct(stream.next_or(span), '>')?;

        let expansion = expect_group(stream.next_or(span), Param::Named("expansion"))?;

        rules.push(Rule { grammar, expansion });

        let Some(tt) = stream.next() else { break };
        expect_punct(Ok(tt), ';')?;
    }

    let eager_rules = rules
        .iter()
        .cloned()
        .map(|r| r.make_eager(crate_path, eager_call_sigil, hidden_ident))
        .collect();
    let pure_rules = rules;

    Ok(Rules {
        metas,
        macro_name,
        eager: eager_rules,
        pure: pure_rules,
    })
}

pub fn expand_rules_legacy(
    span: Span,
    eager_call_sigil: &Literal,
    stream: &mut dyn Iterator<Item = TokenTree>,
) -> Result<TokenStream, Error> {
    let crate_path = crate_path();

    let mut stream = stream.peekable();

    let hidden_ident = match stream.peek() {
        None => return Ok(TokenStream::new()),
        Some(TokenTree::Punct(p)) if p.as_char() == '$' => {
            let _dollar = stream.next();
            expect_ident(stream.next_or(span), Param::Named("eager_ident"))?
        }
        Some(_token) => get_eager_2_ident(),
    };

    let mut output = TokenStream::new();
    while stream.peek().is_some() {
        let rules = expand_rules(
            span,
            &hidden_ident,
            &crate_path,
            eager_call_sigil,
            &mut stream,
        )?;

        rules.to_tokens(&mut output);
    }
    Ok(output)
}

pub fn eager_macro_rules(stream: TokenStream) -> Result<TokenStream, Error> {
    let eager_call_sigil = eager_call_sigil();

    // As with all eager-macros, first try the sigil.
    // If we find one, then the caller was eager, so we can just
    // pick up where they left off
    let state = State::decode_from_stream(stream.clone(), false, |mut v| {
        Ok(expand_rules_legacy(Span::call_site(), &eager_call_sigil, &mut v)?.into_iter())
    })?;

    let output = match state.map(|s| s.process(&eager_call_sigil)) {
        Some(Ok(processed)) => processed.into_token_stream(),
        Some(Err(err)) => return Err(err),
        None => expand_rules_legacy(
            Span::call_site(),
            &eager_call_sigil,
            &mut stream.into_iter(),
        )?,
    };

    Ok(output)
}

pub fn eager_macro(attr: TokenStream, stream: TokenStream) -> Result<TokenStream, Error> {
    let eager_call_sigil = eager_call_sigil();
    let crate_path = crate_path();
    let span = Span::call_site();

    let hidden_ident = {
        let mut stream = attr.into_iter();
        let hidden_ident = match stream.next() {
            Some(attr) => expect_ident(Ok(attr), Param::Named("eager_ident"))?,
            None => get_eager_2_ident(),
        };
        if stream.next().is_some() {
            return Err(Error {
                span,
                msg: "`eager_macro` only takes 1 input".into(),
                note: None,
            });
        }
        hidden_ident
    };
    let stream: &mut dyn Iterator<Item = TokenTree> = &mut stream.into_iter();

    expand_rules(
        span,
        &hidden_ident,
        &crate_path,
        &eager_call_sigil,
        &mut stream.peekable(),
    )
    .map(ToTokens::into_token_stream)
}
