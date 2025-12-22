mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax::{SyntaxKind, tree};

#[test]
fn test_bad_token() {
    let mut lexer = Lexer::new(b"@#$");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::BadToken.into(), b"@#$")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
