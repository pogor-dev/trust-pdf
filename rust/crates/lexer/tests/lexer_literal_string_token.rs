mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax::{SyntaxKind, tree};


#[test]
fn test_string_literal_simple() {
    let mut lexer = Lexer::new(b"(This is a string)");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StringLiteralToken.into(), b"(This is a string)")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}