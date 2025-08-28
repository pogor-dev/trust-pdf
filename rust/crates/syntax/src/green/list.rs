use crate::{GreenNode, SyntaxKind};

pub trait SyntaxList: GreenNode {
    fn kind(&self) -> SyntaxKind {
        SyntaxKind::List
    }
}
