#![cfg(target_arch = "wasm32")]

use lexer::wasm::Lexer;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!();

#[wasm_bindgen_test]
fn test_lexer_when_empty_input_expect_eof() {
    let mut lx = Lexer::new(&[]);
    let t = lx.next_token();
    assert_eq!(t.kind(), "EndOfFileToken");
    assert_eq!(t.text(), "");
    assert_eq!(t.width(), 0);
}

#[wasm_bindgen_test]
fn test_lexer_when_nonempty_expect_not_eof_first_and_reach_eof() {
    let src = b"%PDF-2.0\n1 0 obj\nendobj\n";
    let mut lx = Lexer::new(src);

    // First token should not be EOF on non-empty input
    let first = lx.next_token();
    assert_ne!(first.kind(), "EndOfFileToken");

    // Accumulate widths until EOF and ensure we consumed the whole input
    let mut total_width = first.width();
    let mut reached_eof = false;
    for _ in 0..1024 {
        let t = lx.next_token();
        total_width += t.width();
        if t.kind() == "EndOfFileToken" {
            reached_eof = true;
            break;
        }
    }

    assert!(reached_eof, "lexer did not reach EOF within iteration budget");
    assert_eq!(total_width, src.len());
}

#[wasm_bindgen_test]
fn test_lexer_when_calls_repeat_expect_idempotent_eof() {
    // Once EOF is reached, repeated calls should continue to return EOF
    let mut lx = Lexer::new(&[]);
    for _ in 0..10 {
        let t = lx.next_token();
        assert_eq!(t.kind(), "EndOfFileToken");
        assert_eq!(t.width(), 0);
    }
}
