mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax::{SyntaxKind, tree};

#[test]
fn test_scan_numeric_literal_when_integer_123_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_integer_43445_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_positive_integer_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_negative_integer_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_zero_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_leading_zeros_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_real_number_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_negative_real_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_positive_real_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_trailing_decimal_point_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_leading_decimal_point_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_leading_zeros_with_decimal_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_only_decimal_fraction_expect_numeric_literal_token() {
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
fn test_scan_numeric_literal_when_double_plus_expect_bad_token() {
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
fn test_scan_numeric_literal_when_sign_mid_number_expect_bad_token() {
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
fn test_scan_numeric_literal_when_multiple_decimal_points_expect_bad_token() {
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
fn test_scan_numeric_literal_when_multiple_decimals_starting_with_point_expect_bad_token() {
    let mut lexer = Lexer::new(b".1.2.3");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::BadToken.into(), b".1.2.3")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
