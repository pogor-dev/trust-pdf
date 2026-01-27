use crate::{GreenCst, GreenDiagnostics, GreenElement, GreenNode, GreenNodeSyntax, GreenTrait, SyntaxKind};

#[derive(Clone)]
pub struct GreenExpressionSyntax(pub GreenNode);

impl GreenNodeSyntax for GreenExpressionSyntax {
    #[inline]
    fn green(&self) -> &GreenNode {
        &self.0
    }
}

#[derive(Clone)]
pub struct GreenListSyntax(GreenExpressionSyntax);

impl GreenListSyntax {
    pub fn new(kind: SyntaxKind, elements: Vec<GreenElement>, diagnostics: Option<GreenDiagnostics>) -> Self {
        let green = GreenNode::new(kind, elements, diagnostics);
        GreenListSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn elements(&self) -> Option<GreenNode> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => Some(n),
            _ => None,
        }
    }
}

impl GreenCst for GreenListSyntax {
    fn can_cast(node: &crate::GreenNode) -> bool {
        node.kind() == SyntaxKind::List
    }

    fn cast(node: crate::GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenListSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}
