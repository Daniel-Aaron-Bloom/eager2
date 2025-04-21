use crate::{
    consts::eager_call_sigil,
    pm::{Delimiter, Group, Ident, Punct, Spacing, Span, ToTokens, TokenStream},
    state::{Mode, State},
    Error,
};

pub fn mode(stream: TokenStream, eager: bool) -> Result<TokenStream, Error> {
    eval_helper(stream, eager)
}

pub fn eager_wrap(stream: TokenStream, name: &str) -> Result<TokenStream, Error> {
    let mut tokens = TokenStream::new();
    Ident::new(name, Span::call_site()).to_tokens(&mut tokens);
    Punct::new('!', Spacing::Alone).to_tokens(&mut tokens);
    Group::new(Delimiter::Brace, stream).to_tokens(&mut tokens);
    eval_helper(tokens, true)
}

fn eval_helper(stream: TokenStream, eager: bool) -> Result<TokenStream, Error> {
    // Check the caller mode
    let mode = Mode::eager(eager);
    let eager_call_sigil = eager_call_sigil();

    // As with all eager-macros, first try the sigil
    // If we find one, then the caller was eager, so we can just
    // pick up where they left off
    let state = State::decode_from_stream(stream.clone(), true, Ok)?;

    // Fallback to the case where the caller wasn't eager
    // in which case we start from scratch
    let state =
        state.unwrap_or_else(|| State::new(Span::call_site(), Delimiter::Bracket, mode, stream));

    // Try to move the state as far forward as we can
    let output = state.process(&eager_call_sigil)?.into_token_stream();

    Ok(output)
}
