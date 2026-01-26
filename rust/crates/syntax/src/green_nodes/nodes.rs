use crate::{GreenDiagnostics, GreenElement, GreenNode, GreenNodeSyntax, GreenToken, GreenTrait, SyntaxKind, green::Slots};

// TODO: lex the PDF version separately? Might be false positive inside the document
#[derive(Clone)]
pub struct GreenPdfDocumentSyntax {
    kind: SyntaxKind,
    bodies: GreenNode,
}

pub struct GreenPdfDocumentInnerSyntax {
    kind: SyntaxKind,
    objects: GreenNode,
    xref_table: GreenNode,
    trailer: GreenNode,
}

#[derive(Clone)]
pub struct GreenExpressionSyntax(GreenNode);

impl GreenNodeSyntax for GreenExpressionSyntax {
    #[inline]
    fn green(&self) -> &GreenNode {
        &self.0
    }
}

#[derive(Clone)]
pub struct GreenLiteralExpressionSyntax(GreenExpressionSyntax);

impl GreenLiteralExpressionSyntax {
    pub fn new(kind: SyntaxKind, token: GreenToken, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Token(token)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenLiteralExpressionSyntax(GreenExpressionSyntax(green))
    }

    pub fn token(&self) -> Option<GreenToken> {
        match self.green().slot(0) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

impl GreenNodeSyntax for GreenLiteralExpressionSyntax {
    #[inline]
    fn green(&self) -> &GreenNode {
        &self.0.0
    }
}
