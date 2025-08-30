use crate::SyntaxKind;

/// A builder for a green tree.
#[derive(Default)]
pub struct GreenNodeBuilder {}

impl GreenNodeBuilder {
    /// Creates new builder.
    #[inline]
    pub fn new() -> GreenNodeBuilder {
        GreenNodeBuilder::default()
    }

    /// Adds new token to the current branch.
    #[inline]
    pub fn token(&mut self, kind: SyntaxKind, text: &[u8]) {
        // let (hash, token) = self.cache.token(kind, text);
        // self.children.push((hash, token.into()));
    }
}
