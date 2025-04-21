use crate::pm::{Delimiter, Group, Literal, ToTokens, TokenStream};
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
