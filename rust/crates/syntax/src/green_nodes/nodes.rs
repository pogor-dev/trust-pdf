use crate::{GreenCst, GreenDiagnostics, GreenElement, GreenNode, GreenNodeSyntax, GreenToken, GreenTrait, SyntaxKind, green::Slots};

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

impl GreenCst for GreenLiteralExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        // Accept only literal expression node kinds with exactly one token child.
        let is_literal_kind = matches!(
            node.kind(),
            SyntaxKind::TrueLiteralExpression
                | SyntaxKind::FalseLiteralExpression
                | SyntaxKind::NullLiteralExpression
                | SyntaxKind::NumericLiteralExpression
                | SyntaxKind::NameLiteralExpression
                | SyntaxKind::StringLiteralExpression
                | SyntaxKind::HexStringLiteralExpression
        );

        if !is_literal_kind || node.slot_count() != 1 {
            return false;
        }

        match node.slot(0) {
            Some(GreenElement::Token(t)) => matches!(
                t.kind(),
                SyntaxKind::TrueKeyword
                    | SyntaxKind::FalseKeyword
                    | SyntaxKind::NullKeyword
                    | SyntaxKind::NumericLiteralToken
                    | SyntaxKind::NameLiteralToken
                    | SyntaxKind::StringLiteralToken
                    | SyntaxKind::HexStringLiteralToken
            ),
            _ => false,
        }
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenLiteralExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

#[cfg(test)]
mod cast_tests {
    use super::*;

    #[test]
    fn test_can_cast_literal_expression_true() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword, b"true", None, None, None);
        let node = GreenNode::new(SyntaxKind::TrueLiteralExpression, vec![GreenElement::Token(token)], None);
        assert!(GreenLiteralExpressionSyntax::can_cast(&node));
        assert!(GreenLiteralExpressionSyntax::cast(node).is_some());
    }

    #[test]
    fn test_can_cast_literal_expression_wrong_kind() {
        let token = GreenToken::new(SyntaxKind::TrueKeyword, b"true", None, None, None);
        let node = GreenNode::new(SyntaxKind::List, vec![GreenElement::Token(token)], None);
        assert!(!GreenLiteralExpressionSyntax::can_cast(&node));
    }
}
