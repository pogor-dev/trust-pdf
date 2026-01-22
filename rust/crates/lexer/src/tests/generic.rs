use super::utils::{assert_nodes_equal, generate_node_from_lexer};
use crate::Lexer;
use syntax::{SyntaxKind, tree};

#[test]
fn test_scan_token_when_unknown_characters_expect_bad_token() {
    let mut lexer = Lexer::new(b" @#$%");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::BadToken) => {
                trivia(SyntaxKind::WhitespaceTrivia, b" "),
                text(b"@#$"),
                trivia(SyntaxKind::CommentTrivia, b"%")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_token_when_unmatched_closing_paren_expect_bad_token() {
    let mut lexer = Lexer::new(b" ) ");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::BadToken) => {
                trivia(SyntaxKind::WhitespaceTrivia, b" "),
                text(b")"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
