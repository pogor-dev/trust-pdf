use crate::NodeOrToken;
use crate::api::element::SyntaxElement;
use crate::api::language::Language;
use crate::api::node::SyntaxNode;
use crate::green::GreenNode;
use crate::{GreenToken, green::GreenTokenData, red, utils::Direction};
use std::{fmt, marker::PhantomData, ops::Range};

/// Represents a token in the syntax tree for a given language.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyntaxToken<L: Language> {
    pub(super) raw: red::SyntaxToken,
    pub(super) _p: PhantomData<L>,
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

    pub fn text_range(&self) -> Range<u64> {
        self.raw.text_range()
    }

    pub fn index(&self) -> usize {
        self.raw.index()
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
    #[deprecated = "use `SyntaxToken::parent_ancestors` instead"]
    pub fn ancestors(&self) -> impl Iterator<Item = SyntaxNode<L>> + use<L> {
        self.parent_ancestors()
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

    pub fn siblings_with_tokens(&self, direction: Direction) -> impl Iterator<Item = SyntaxElement<L>> + use<L> {
        self.raw.siblings_with_tokens(direction).map(SyntaxElement::from)
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
}

impl<L: Language> From<red::SyntaxToken> for SyntaxToken<L> {
    fn from(raw: red::SyntaxToken) -> SyntaxToken<L> {
        SyntaxToken { raw, _p: PhantomData }
    }
}

impl<L: Language> From<SyntaxToken<L>> for red::SyntaxToken {
    fn from(token: SyntaxToken<L>) -> red::SyntaxToken {
        token.raw
    }
}

impl<L: Language> fmt::Debug for SyntaxToken<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{:?}", self.kind(), self.text_range())?;
        if self.full_text().len() < 25 {
            return write!(f, " {:?}", self.full_text());
        }
        let text = self.full_text();
        if let Ok(text_str) = std::str::from_utf8(&text) {
            for idx in 21..25 {
                if text_str.is_char_boundary(idx) {
                    let text = format!("{} ...", &text_str[..idx]);
                    return write!(f, " {:?}", text);
                }
            }
        } else {
            // Fallback for non-UTF8 data
            let text = format!("{:?} ...", &text[..21.min(text.len())]);
            return write!(f, " {}", text);
        }
        unreachable!()
    }
}

impl<L: Language> fmt::Display for SyntaxToken<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.raw, f)
    }
}
