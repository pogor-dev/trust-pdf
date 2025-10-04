use crate::{GreenToken, SyntaxKind};

#[derive(Debug)]
pub struct GreenNode<'node> {
    kind: SyntaxKind,
    children: Vec<GreenChild<'node>>,
    full_width: usize,
}

#[derive(Debug)]
pub enum GreenChild<'child> {
    Node(&'child GreenNode<'child>),
    Token(&'child GreenToken<'child>),
}
