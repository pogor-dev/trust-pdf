use crate::{GreenToken, SyntaxKind};

#[derive(Debug)]
pub struct GreenNode {
    kind: SyntaxKind,
    children: Vec<GreenChild>,
    full_width: usize,
}

#[derive(Debug)]
pub enum GreenChild {
    Node(GreenNode),
    Token(GreenToken),
}
