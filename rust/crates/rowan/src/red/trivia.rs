use crate::{GreenTrivia, SyntaxToken};

pub struct SyntaxTrivia {
    pub token: SyntaxToken,
    underlying_node: GreenTrivia,
    position: u64,
    index: u16,
}
