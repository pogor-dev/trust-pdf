use crate::{GreenCst, GreenDiagnostics, GreenElement, GreenExpressionSyntax, GreenNode, GreenNodeSyntax, GreenToken, GreenTrait, SyntaxKind};

/// Literal value: number, name, string, hex string, boolean, or null
/// ISO 32000-2:2020, 7.3 — Objects
#[derive(Clone)]
pub struct GreenLiteralExpressionSyntax(pub GreenExpressionSyntax);

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
