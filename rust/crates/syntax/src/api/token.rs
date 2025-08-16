use std::{fmt, marker::PhantomData, ops::Range};

use crate::{
    Direction, GreenNode, GreenToken, GreenTokenData, Language, NodeOrToken, SyntaxElement,
    SyntaxNode, cursor,
};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxToken<L: Language> {
    raw: cursor::SyntaxToken,
    _p: PhantomData<L>,
}

impl<L: Language> SyntaxToken<L> {
    /// Returns a green tree, equal to the green tree this token
    /// belongs to, except with this token substituted. The complexity
    /// of the operation is proportional to the depth of the tree.
    pub fn replace_with(&self, new_token: GreenToken) -> GreenNode {
        self.raw.replace_with(new_token)
    }

    pub fn kind(&self) -> L::Kind {
        L::kind_from_raw(self.raw.kind())
    }

    pub fn span(&self) -> Range<u32> {
        self.raw.span()
    }

    pub fn full_span(&self) -> Range<u32> {
        self.raw.full_span()
    }

    pub fn index(&self) -> usize {
        self.raw.index()
    }

    pub fn text(&self) -> &[u8] {
        self.raw.text()
    }

    pub fn full_text(&self) -> Vec<u8> {
        self.raw.full_text()
    }

    pub fn green(&self) -> &GreenTokenData {
        self.raw.green()
    }

    pub fn parent(&self) -> Option<SyntaxNode<L>> {
        self.raw.parent().map(SyntaxNode::from)
    }

    /// Iterator over all the ancestors of this token excluding itself.
    pub fn parent_ancestors(&self) -> impl Iterator<Item = SyntaxNode<L>> + use<L> {
        self.raw.ancestors().map(SyntaxNode::from)
    }

    pub fn next_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
        self.raw.next_sibling_or_token().map(NodeOrToken::from)
    }

    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElement<L>> {
        self.raw.prev_sibling_or_token().map(NodeOrToken::from)
    }

    pub fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = SyntaxElement<L>> + use<L> {
        self.raw
            .siblings_with_tokens(direction)
            .map(SyntaxElement::from)
    }

    /// Next token in the tree (i.e, not necessary a sibling).
    pub fn next_token(&self) -> Option<SyntaxToken<L>> {
        self.raw.next_token().map(SyntaxToken::from)
    }
    /// Previous token in the tree (i.e, not necessary a sibling).
    pub fn prev_token(&self) -> Option<SyntaxToken<L>> {
        self.raw.prev_token().map(SyntaxToken::from)
    }

    pub fn detach(&self) {
        self.raw.detach()
    }

    // pub fn leading_trivia(&self) -> SyntaxTrivia<L> {}
    // pub fn trailing_trivia(&self) -> SyntaxTrivia<L> {}
}

impl<L: Language> From<cursor::SyntaxToken> for SyntaxToken<L> {
    fn from(raw: cursor::SyntaxToken) -> SyntaxToken<L> {
        SyntaxToken {
            raw,
            _p: PhantomData,
        }
    }
}

impl<L: Language> From<SyntaxToken<L>> for cursor::SyntaxToken {
    fn from(token: SyntaxToken<L>) -> cursor::SyntaxToken {
        token.raw
    }
}

impl<L: Language> fmt::Debug for SyntaxToken<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{:?}", self.kind(), self.full_span())?;
        if self.text().len() < 25 {
            return write!(f, " {:?}", self.text());
        }
        let text = self.text();
        for idx in 21..25 {
            if text.is_char_boundary(idx) {
                let text = format!("{} ...", &text[..idx]);
                return write!(f, " {:?}", text);
            }
        }
        unreachable!()
    }
}

impl<L: Language> fmt::Display for SyntaxToken<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.raw, f)
    }
}
