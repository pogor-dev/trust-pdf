use crate::{GreenToken, SyntaxNode};

pub struct SyntaxToken {
    pub node: SyntaxNode,
    underlying_node: GreenToken,
    position: u64,
    index: u16,
}
