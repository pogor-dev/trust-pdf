use std::sync::{OnceLock, RwLock};

use crate::{GreenCst, GreenLiteralExpressionSyntax, GreenToken, NodeCache, SyntaxKind};

static CACHE: OnceLock<RwLock<NodeCache>> = OnceLock::new();

fn cache() -> &'static RwLock<NodeCache> {
    CACHE.get_or_init(|| RwLock::new(NodeCache::default()))
}

pub struct GreenSyntaxFactory();

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

        let mut children = vec![(0u64, token.clone().into())];
        let (_, node) = cache()
            .write()
            .expect("Failed to acquire write lock on node cache")
            .node(kind, &mut children, 0, None);

        match GreenLiteralExpressionSyntax::cast(node) {
            Some(lit_expr) => lit_expr,
            None => panic!("Failed to cast cached node to GreenLiteralExpressionSyntax"),
        }
    }
}
