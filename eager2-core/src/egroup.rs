use std::mem;

use crate::pm::{token_stream, Delimiter, Group, ToTokens, TokenStream, TokenTree};

pub enum EfficientGroup<P> {
    Raw(Group),
    Processed(P),
}

impl<P: Default> Default for EfficientGroup<P> {
    fn default() -> Self {
        Self::Processed(P::default())
    }
}

impl<P> From<Group> for EfficientGroup<P> {
    fn from(g: Group) -> Self {
        Self::Raw(g)
    }
}
pub enum EgIntoIter<P: IntoIterator<Item = TokenTree>> {
    Raw(token_stream::IntoIter),
    Processed(P::IntoIter),
}

impl<P: IntoIterator<Item = TokenTree>> Clone for EgIntoIter<P>
where
    P::IntoIter: Clone,
{
    fn clone(&self) -> Self {
        match self {
            Self::Processed(p) => Self::Processed(p.clone()),
            Self::Raw(g) => Self::Raw(g.clone()),
        }
    }
}

impl<P: IntoIterator<Item = TokenTree>> Iterator for EgIntoIter<P> {
    type Item = TokenTree;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Processed(p) => p.next(),
            Self::Raw(g) => g.next(),
        }
    }
}

impl<P: IntoIterator<Item = TokenTree>> IntoIterator for EfficientGroup<P> {
    type IntoIter = EgIntoIter<P>;
    type Item = TokenTree;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Processed(p) => EgIntoIter::Processed(p.into_iter()),
            Self::Raw(g) => EgIntoIter::Raw(g.stream().into_iter()),
        }
    }
}

pub type EfficientGroupT = EfficientGroup<TokenStream>;
pub type EfficientGroupV = EfficientGroup<Vec<TokenTree>>;

impl ToTokens for EfficientGroupT {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Raw(g) => g.to_tokens(tokens),
            Self::Processed(s) => Group::new(Delimiter::Bracket, s.clone()).to_tokens(tokens),
        }
    }
}
impl ToTokens for EfficientGroupV {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Raw(g) => g.to_tokens(tokens),
            Self::Processed(s) => {
                let group = Group::new(Delimiter::Bracket, s.iter().cloned().collect());
                group.to_tokens(tokens);
            }
        }
    }
}

impl EfficientGroupV {
    pub fn push(&mut self, tt: TokenTree) {
        self.as_mut_vec().push(tt);
    }

    pub fn as_mut_vec(&mut self) -> &mut Vec<TokenTree> {
        if let Self::Raw(r) = self {
            let v = r.stream().into_iter().collect();
            *self = Self::Processed(v);
        }

        let Self::Processed(p) = self else {
            unreachable!()
        };
        p
    }
    pub fn append_to_stream(&self, tokens: &mut TokenStream) {
        match self {
            Self::Processed(p) => tokens.extend(p.iter().cloned()),
            Self::Raw(g) => tokens.extend(g.stream()),
        }
    }
    pub fn append(&mut self, other: Self) {
        let v = self.as_mut_vec();
        match other {
            Self::Processed(mut p) => v.append(&mut p),
            Self::Raw(g) => v.extend(g.stream()),
        }
    }
    pub fn extend(&mut self, other: impl IntoIterator<Item = TokenTree>) {
        let v = self.as_mut_vec();
        v.extend(other);
    }
    #[must_use]
    pub fn take(&mut self) -> Self {
        mem::replace(self, Self::Processed(vec![]))
    }
}
impl EfficientGroupT {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Processed(p) => p.is_empty(),
            Self::Raw(g) => g.stream().is_empty(),
        }
    }
    pub fn append_to_stream(&self, tokens: &mut TokenStream) {
        match self {
            Self::Processed(p) => tokens.extend(p.clone()),
            Self::Raw(g) => tokens.extend(g.stream()),
        }
    }
    #[must_use]
    pub fn into_stream(self) -> TokenStream {
        match self {
            Self::Processed(p) => p,
            Self::Raw(g) => g.stream(),
        }
    }
    pub fn as_mut_stream(&mut self) -> &mut TokenStream {
        if let Self::Raw(r) = self {
            *self = Self::Processed(r.stream());
        }

        let Self::Processed(p) = self else {
            unreachable!()
        };
        p
    }
    pub fn append<P>(&mut self, other: EfficientGroup<P>)
    where
        P: IntoIterator<Item = TokenTree>,
    {
        let s = self.as_mut_stream();
        match other {
            EfficientGroup::Processed(p) => s.extend(p),
            EfficientGroup::Raw(g) => s.extend(g.stream()),
        }
    }
}
