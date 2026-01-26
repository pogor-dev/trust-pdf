use crate::{GreenLiteralExpressionSyntax, GreenToken, SyntaxKind};

pub struct GreenSyntaxFactory;

impl GreenSyntaxFactory {
    pub fn literal_expression(kind: SyntaxKind, token: GreenToken) -> GreenLiteralExpressionSyntax {
        match kind {
            SyntaxKind::TrueLiteralExpression => {}
            SyntaxKind::FalseLiteralExpression => {}
            SyntaxKind::NullLiteralExpression => {}
            SyntaxKind::NumericLiteralExpression => {}
            SyntaxKind::NameLiteralExpression => {}
            SyntaxKind::StringLiteralExpression => {}
            SyntaxKind::HexStringLiteralExpression => {}
            _ => panic!("Invalid kind for literal expression: {:?}", kind),
        }

        debug_assert!(
            matches!(
                token.kind(),
                SyntaxKind::TrueKeyword
                    | SyntaxKind::FalseKeyword
                    | SyntaxKind::NullKeyword
                    | SyntaxKind::NumericLiteralToken
                    | SyntaxKind::NameLiteralToken
                    | SyntaxKind::StringLiteralToken
                    | SyntaxKind::HexStringLiteralToken
            ),
            "Invalid token kind for literal expression: {:?}",
            token.kind()
        );

        GreenLiteralExpressionSyntax::new(kind, token, None)
    }
}
