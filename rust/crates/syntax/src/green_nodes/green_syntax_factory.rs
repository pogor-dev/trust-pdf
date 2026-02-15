use std::sync::{OnceLock, RwLock};

use crate::{GreenCst, GreenLiteralExpressionSyntax, GreenToken, NodeCache, SyntaxKind};

static CACHE: OnceLock<RwLock<NodeCache>> = OnceLock::new();

fn cache() -> &'static RwLock<NodeCache> {
    // TODO: consider using a more efficient concurrent cache implementation if contention becomes an issue
    // TODO: static lifetime will not evict nodes, so we may want to implement some eviction strategy if memory usage becomes a concern
    CACHE.get_or_init(|| RwLock::new(NodeCache::default()))
}

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

pub fn missing_token(kind: SyntaxKind) -> GreenToken {
    // TODO: consider caching missing tokens similar to Roslyn's approach
    let (_, token) = cache()
        .write()
        .expect("Failed to acquire write lock on node cache")
        .token(kind, b"", None, None, None);

    token
}
