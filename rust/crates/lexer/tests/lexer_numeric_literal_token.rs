mod support;

use lexer::Lexer;
use support::{assert_eof_token, assert_numeric_literal_token};
use syntax::{GreenToken, SyntaxKind};

#[test]
fn test_numeric_literal_123() {
    let mut lexer = Lexer::new(b"123");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"123");
    assert_eof_token(&lexer.next_token());
}

#[test]
fn test_numeric_literal_43445() {
    let mut lexer = Lexer::new(b"43445");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"43445");
    assert_eof_token(&lexer.next_token());
}

#[test]
fn test_numeric_literal_plus_17() {
    let mut lexer = Lexer::new(b"+17");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"+17");
    assert_eof_token(&lexer.next_token());
}

#[test]
fn test_numeric_literal_minus_98() {
    let mut lexer = Lexer::new(b"-98");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"-98");
    assert_eof_token(&lexer.next_token());
}

#[test]
fn test_numeric_literal_0() {
    let mut lexer = Lexer::new(b"0");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"0");
    assert_eof_token(&lexer.next_token());
}

#[test]
fn test_numeric_literal_00987() {
    let mut lexer = Lexer::new(b"00987");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"00987");
    assert_eof_token(&lexer.next_token());
}

#[test]
fn test_numeric_literal_34_5() {
    let mut lexer = Lexer::new(b"34.5");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"34.5");
    assert_eof_token(&lexer.next_token());
}

#[test]
fn test_numeric_literal_minus_3_62() {
    let mut lexer = Lexer::new(b"-3.62");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"-3.62");
    assert_eof_token(&lexer.next_token());
}

#[test]
fn test_numeric_literal_plus_123_6() {
    let mut lexer = Lexer::new(b"+123.6");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"+123.6");
    assert_eof_token(&lexer.next_token());
}

#[test]
fn test_numeric_literal_4_() {
    let mut lexer = Lexer::new(b"4.");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"4.");
    assert_eof_token(&lexer.next_token());
}

#[test]
#[allow(non_snake_case)]
fn test_numeric_literal_minus__002() {
    let mut lexer = Lexer::new(b"-.002");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"-.002");
    assert_eof_token(&lexer.next_token());
}

#[test]
fn test_numeric_literal_009_87() {
    let mut lexer = Lexer::new(b"009.87");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b"009.87");
    assert_eof_token(&lexer.next_token());
}

#[test]
#[allow(non_snake_case)]
fn test_numeric_literal__3_4() {
    let mut lexer = Lexer::new(b".34");
    let token = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::NumericLiteralToken, b".34");
    assert_eof_token(&lexer.next_token());
}

#[test]
fn test_numeric_plus_plus_invalid() {
    let mut lexer = Lexer::new(b"++");
    let token: GreenToken = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::BadToken, b"++");
    assert_eof_token(&lexer.next_token());
}

#[test]
fn test_numeric_plus_minus_345_minus_36_invalid() {
    let mut lexer = Lexer::new(b"+345-36");
    let token: GreenToken = lexer.next_token();
    assert_numeric_literal_token(&token, SyntaxKind::BadToken, b"+345-36");
    assert_eof_token(&lexer.next_token());
}

// TODO: Add test coverage for multiple decimal points (e.g., "1.2.3", "4.5.6.7") to verify proper BadToken handling.
