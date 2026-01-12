use crate::GreenNode;

pub struct SyntaxNode {
    parent: Option<Box<SyntaxNode>>,
    underlying_node: GreenNode,
    position: u64,
    index: u16,
}
