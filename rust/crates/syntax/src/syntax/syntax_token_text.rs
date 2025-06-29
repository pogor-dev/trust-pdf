use std::{
    borrow::Borrow,
    ops::{Deref, Range},
};

/// Reference to the text of a SyntaxToken without having to worry about the lifetime of `&str`.
#[derive(Eq, Clone)]
pub struct TokenText {
    // Using a green token to ensure this type is Send + Sync.
    token: GreenToken,
    /// Relative range of the "selected" token text.
    range: Range<u32>,
}

impl std::hash::Hash for TokenText {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.text().hash(state);
    }
}

impl TokenText {
    pub(crate) fn new(token: GreenToken) -> TokenText {
        let range = 0..token.text_len();
        Self { token, range }
    }

    pub(crate) fn with_range(token: GreenToken, range: Range<u32>) -> TokenText {
        debug_assert!(range.end() <= token.text_len());
        Self { token, range }
    }

    /// Returns the length of the text
    pub fn len(&self) -> u32 {
        self.range.len()
    }

    /// Returns `true` if the text is empty
    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }

    /// Returns a subslice of the text.
    /// `range.end()` must be lower or equal to `self.len()`
    pub fn slice(mut self, range: Range<u32>) -> TokenText {
        assert!(
            range.end() <= self.len(),
            "Range {range:?} exceeds the text length {:?}",
            self.len()
        );
        self.range = range.start() + self.range.start()..range.end();
        self
    }

    pub fn text(&self) -> &[u8] {
        &self.token.text()[self.range]
    }
}

impl Deref for TokenText {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.text()
    }
}

impl std::fmt::Display for TokenText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text())
    }
}

impl std::fmt::Debug for TokenText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.text())
    }
}

impl PartialEq for TokenText {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl PartialEq<&'_ [u8]> for TokenText {
    fn eq(&self, rhs: &&'_ [u8]) -> bool {
        **self == **rhs
    }
}

impl PartialEq<TokenText> for &'_ [u8] {
    fn eq(&self, other: &TokenText) -> bool {
        **self == **other
    }
}

impl Borrow<[u8]> for TokenText {
    fn borrow(&self) -> &[u8] {
        self.text()
    }
}
