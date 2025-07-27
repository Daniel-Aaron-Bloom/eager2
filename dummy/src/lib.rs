use eager2::eager_proc_macro;
use proc_macro::{Punct, Spacing, TokenStream, TokenTree};

#[eager_proc_macro]
pub fn add(input: TokenStream) -> TokenStream {
    let mut res = TokenStream::new();
    res.extend([TokenTree::from(Punct::new('+', Spacing::Alone))]);
    res.extend(input.into_iter().collect::<Vec<TokenTree>>());
    res
}
