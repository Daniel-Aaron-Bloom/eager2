use std::str::FromStr;

use crate::pm::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, ToTokens, TokenStream};

const EAGER2_IDENT: &str = "__eager2_ident_hyR7dMdkMPcptU6h21dioFE3EhoLprgj";
pub const EAGER_CALL_SIGIL: &str = "0𓊆eager2𓊇";
pub const LAZY_SIGIL: &str = "𓆉";
pub const EAGER_SIGIL: &str = "𓂺";

#[must_use]
#[inline(always)]
pub fn get_eager_2_ident() -> Ident {
    Ident::new(EAGER2_IDENT, Span::call_site())
}

#[must_use]
#[inline(always)]
pub(crate) fn get_eager_2_pm_ident(suffix: Option<&str>) -> Ident {
    Ident::new(
        format!("{EAGER2_IDENT}{s}", s = suffix.unwrap_or_default()).as_str(),
        Span::call_site(),
    )
}

#[must_use]
#[inline(always)]
pub fn eager_call_sigil() -> Literal {
    Literal::from_str(EAGER_CALL_SIGIL).unwrap()
}

#[must_use]
#[inline(always)]
pub(crate) fn eager_call_sigil_proc_macro() -> TokenStream {
    use crate::interpolate;

    interpolate! {
        ::proc_macro::Literal::from_str(#EAGER_CALL_SIGIL).unwrap()
    }
}

#[must_use]
pub fn crate_path() -> TokenStream {
    let mut tokens = TokenStream::new();
    let mut dollar = Punct::new('$', Spacing::Joint);
    dollar.set_span(Span::mixed_site());
    dollar.to_tokens(&mut tokens);
    Ident::new("crate", Span::mixed_site()).to_tokens(&mut tokens);

    let mut group = Group::new(Delimiter::None, tokens);
    group.set_span(Span::mixed_site());

    group.into_token_stream()
}
