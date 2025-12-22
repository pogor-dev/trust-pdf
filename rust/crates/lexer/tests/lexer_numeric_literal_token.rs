mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax::{SyntaxKind, tree};

#[test]
fn test_numeric_literal_123() {
    let mut lexer = Lexer::new(b"123");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"123")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_literal_43445() {
    let mut lexer = Lexer::new(b"43445");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"43445")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_literal_plus_17() {
    let mut lexer = Lexer::new(b"+17");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"+17")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_literal_minus_98() {
    let mut lexer = Lexer::new(b"-98");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"-98")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_literal_0() {
    let mut lexer = Lexer::new(b"0");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"0")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_literal_00987() {
    let mut lexer = Lexer::new(b"00987");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"00987")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_literal_34_5() {
    let mut lexer = Lexer::new(b"34.5");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"34.5")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_literal_minus_3_62() {
    let mut lexer = Lexer::new(b"-3.62");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"-3.62")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_literal_plus_123_6() {
    let mut lexer = Lexer::new(b"+123.6");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"+123.6")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_literal_4_() {
    let mut lexer = Lexer::new(b"4.");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"4.")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
#[allow(non_snake_case)]
fn test_numeric_literal_minus__002() {
    let mut lexer = Lexer::new(b"-.002");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"-.002")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_literal_009_87() {
    let mut lexer = Lexer::new(b"009.87");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b"009.87")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
#[allow(non_snake_case)]
fn test_numeric_literal__3_4() {
    let mut lexer = Lexer::new(b".34");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into(), b".34")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_plus_plus_invalid() {
    let mut lexer = Lexer::new(b"++");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::BadToken.into(), b"++")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_plus_minus_345_minus_36_invalid() {
    let mut lexer = Lexer::new(b"+345-36");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::BadToken.into(), b"+345-36")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_multiple_decimal_points_invalid() {
    let mut lexer = Lexer::new(b"12.34.56");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::BadToken.into(), b"12.34.56")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_numeric_multiple_decimal_points_starts_with_decimal_point_invalid() {
    let mut lexer = Lexer::new(b".1.2.3");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::BadToken.into(), b".1.2.3")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
