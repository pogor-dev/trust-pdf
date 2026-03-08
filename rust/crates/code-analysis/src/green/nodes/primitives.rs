use crate::{GreenCst, GreenDiagnostic, GreenExpressionSyntax, GreenNode, GreenNodeElement, GreenNodeSyntax, GreenTokenElement, SyntaxKind};

/// Literal value: number, name, string, hex string, boolean, or null
/// ISO 32000-2:2020, 7.3 — Objects
#[derive(Clone)]
pub(crate) struct GreenLiteralExpressionSyntax(pub(crate) GreenExpressionSyntax);

impl GreenLiteralExpressionSyntax {
    pub(crate) fn new(kind: SyntaxKind, token: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![token.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenLiteralExpressionSyntax(GreenExpressionSyntax(green))
    }

    pub(crate) fn token(&self) -> Option<GreenTokenElement> {
        match self.green().slot(0) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
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
            Some(GreenNodeElement::Token(t)) => matches!(
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
