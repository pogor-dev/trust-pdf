use crate::{GreenNodeBuilder, SyntaxKind, cursor::node::SyntaxNode};

fn build_tree(chunks: &[&[u8]]) -> SyntaxNode {
    let mut builder = GreenNodeBuilder::new();
    builder.start_node(SyntaxKind(62));
    for &chunk in chunks.iter() {
        builder.token(SyntaxKind(92), chunk)
    }
    builder.finish_node();
    SyntaxNode::new_root(builder.finish())
}

#[test]
fn test_text_equality() {
    fn do_check(t1: &[&[u8]], t2: &[&[u8]]) {
        let t1 = build_tree(t1).full_text();
        let t2 = build_tree(t2).full_text();
        let expected = t1.to_bytes() == t2.to_bytes();
        let actual = t1 == t2;
        assert_eq!(
            expected, actual,
            "`{}` (SyntaxText) `{}` (SyntaxText)",
            t1, t2
        );
        let actual = t1 == *t2.to_bytes();
        assert_eq!(expected, actual, "`{}` (SyntaxText) `{}` (&[u8])", t1, t2);
    }
    fn check(t1: &[&[u8]], t2: &[&[u8]]) {
        do_check(t1, t2);
        do_check(t2, t1)
    }

    check(&[b""], &[b""]);
    check(&[b"a"], &[b""]);
    check(&[b"a"], &[b"a"]);
    check(&[b"abc"], &[b"def"]);
    check(&[b"hello", b"world"], &[b"hello", b"world"]);
    check(&[b"hellowo", b"rld"], &[b"hell", b"oworld"]);
    check(&[b"hel", b"lowo", b"rld"], &[b"helloworld"]);
    check(&[b"{", b"abc", b"}"], &[b"{", b"123", b"}"]);
    check(&[b"{", b"abc", b"}", b"{"], &[b"{", b"123", b"}"]);
    check(&[b"{", b"abc", b"}"], &[b"{", b"123", b"}", b"{"]);
    check(&[b"{", b"abc", b"}ab"], &[b"{", b"abc", b"}", b"ab"]);
}

#[test]
fn test_len_and_is_empty() {
    let empty_text = build_tree(&[b""]).full_text();
    assert_eq!(empty_text.len(), 0);
    assert!(empty_text.is_empty());

    let hello_text = build_tree(&[b"hello"]).full_text();
    assert_eq!(hello_text.len(), 5);
    assert!(!hello_text.is_empty());

    let multipart_text = build_tree(&[b"hello", b" ", b"world"]).full_text();
    assert_eq!(multipart_text.len(), 11);
    assert!(!multipart_text.is_empty());
}

#[test]
fn test_contains_byte() {
    let text = build_tree(&[b"hello", b" ", b"world"]).full_text();

    assert!(text.contains_byte(b'h'));
    assert!(text.contains_byte(b'e'));
    assert!(text.contains_byte(b'o'));
    assert!(text.contains_byte(b' '));
    assert!(text.contains_byte(b'w'));
    assert!(text.contains_byte(b'd'));

    assert!(!text.contains_byte(b'x'));
    assert!(!text.contains_byte(b'z'));
}

#[test]
fn test_find_byte() {
    let text = build_tree(&[b"hello", b" ", b"world"]).full_text();

    assert_eq!(text.find_byte(b'h'), Some(0));
    assert_eq!(text.find_byte(b'e'), Some(1));
    assert_eq!(text.find_byte(b'l'), Some(2)); // First 'l'
    assert_eq!(text.find_byte(b'o'), Some(4)); // First 'o'
    assert_eq!(text.find_byte(b' '), Some(5));
    assert_eq!(text.find_byte(b'w'), Some(6));
    assert_eq!(text.find_byte(b'd'), Some(10));

    assert_eq!(text.find_byte(b'x'), None);
    assert_eq!(text.find_byte(b'z'), None);
}

#[test]
fn test_byte_at() {
    let text = build_tree(&[b"hello", b" ", b"world"]).full_text();

    assert_eq!(text.byte_at(0), Some(b'h'));
    assert_eq!(text.byte_at(1), Some(b'e'));
    assert_eq!(text.byte_at(2), Some(b'l'));
    assert_eq!(text.byte_at(4), Some(b'o'));
    assert_eq!(text.byte_at(5), Some(b' '));
    assert_eq!(text.byte_at(6), Some(b'w'));
    assert_eq!(text.byte_at(10), Some(b'd'));

    assert_eq!(text.byte_at(11), None); // Out of bounds
    assert_eq!(text.byte_at(100), None); // Way out of bounds
}

#[test]
fn test_slice() {
    let text = build_tree(&[b"hello", b" ", b"world"]).full_text();

    // Test Range<usize>
    let slice = text.slice(1..4);
    assert_eq!(slice.to_bytes(), b"ell");

    // Test RangeFrom<usize>
    let slice = text.slice(6..);
    assert_eq!(slice.to_bytes(), b"world");

    // Test RangeTo<usize>
    let slice = text.slice(..5);
    assert_eq!(slice.to_bytes(), b"hello");

    // Test RangeFull
    let slice = text.slice(..);
    assert_eq!(slice.to_bytes(), b"hello world");

    // Test empty slice
    let slice = text.slice(3..3);
    assert!(slice.is_empty());
    assert_eq!(slice.len(), 0);
}

#[test]
fn test_for_each_chunk() {
    let text = build_tree(&[b"hello", b" ", b"world"]).full_text();

    let mut chunks = Vec::new();
    text.for_each_chunk(|chunk| {
        chunks.push(chunk.to_vec());
    });

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0], b"hello");
    assert_eq!(chunks[1], b" ");
    assert_eq!(chunks[2], b"world");
}

#[test]
fn test_to_bytes() {
    let text = build_tree(&[b"hello", b" ", b"world"]).full_text();

    let bytes = text.to_bytes();
    assert_eq!(bytes, b"hello world");

    // Test with empty text
    let empty_text = build_tree(&[]).full_text();
    let empty_bytes = empty_text.to_bytes();
    assert!(empty_bytes.is_empty());

    // Test with invalid UTF-8
    let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
    let invalid_text = build_tree(&[invalid_utf8]).full_text();
    let invalid_bytes = invalid_text.to_bytes();
    assert_eq!(invalid_bytes, vec![0xFF, 0xFE, 0xFD]);
}

#[test]
fn test_debug_format() {
    let text = build_tree(&[b"hello"]).full_text();
    let debug_str = format!("{:?}", text);
    assert_eq!(debug_str, "\"hello\"");

    // Verify the underlying bytes match
    assert_eq!(text.to_bytes(), b"hello");
}

#[test]
fn test_from_syntax_text_to_string() {
    let text = build_tree(&[b"hello", b" ", b"world"]).full_text();

    // Test to_string() method directly (keeping one test for string conversion)
    let string_from_method = text.to_string();
    assert_eq!(string_from_method, "hello world");

    // Test String conversion via Into trait
    let string: String = text.into();
    assert_eq!(string, "hello world");
}

#[test]
fn test_partial_eq_implementations() {
    let text = build_tree(&[b"hello"]).full_text();
    let bytes: &[u8] = b"hello";
    let wrong_bytes: &[u8] = b"world";

    // Test PartialEq<[u8]>
    assert_eq!(text, *bytes);
    assert_ne!(text, *wrong_bytes);

    // Test PartialEq<&[u8]>
    assert_eq!(text, bytes);
    assert_ne!(text, wrong_bytes);

    // Test symmetric equality for [u8]
    assert_eq!(*bytes, text);
    assert_ne!(*wrong_bytes, text);

    // Test symmetric equality for &[u8]
    assert_eq!(bytes, text);
    assert_ne!(wrong_bytes, text);
}

#[test]
fn test_display_with_invalid_utf8() {
    // Create a tree with invalid UTF-8 bytes
    let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
    let text = build_tree(&[invalid_utf8]).full_text();

    let display_str = format!("{}", text);
    assert!(display_str.contains("\\x"));
    assert!(display_str.contains("ff"));
    assert!(display_str.contains("fe"));
    assert!(display_str.contains("fd"));

    // Verify the raw bytes are preserved correctly
    assert_eq!(text.to_bytes(), vec![0xFF, 0xFE, 0xFD]);
}

#[test]
fn test_private_syntax_text_range_impls() {
    use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

    let text = build_tree(&[b"hello world"]).full_text();

    // Test that the private range implementations work with slice
    let _slice1 = text.slice(Range { start: 0, end: 5 });
    let _slice2 = text.slice(RangeFrom { start: 6 });
    let _slice3 = text.slice(RangeTo { end: 5 });
    let _slice4 = text.slice(RangeFull);

    // These calls exercise the private trait implementations
    // even though we can't directly test them
}

#[test]
fn test_edge_cases() {
    // Empty tree
    let empty_text = build_tree(&[]).full_text();
    assert!(empty_text.is_empty());
    assert_eq!(empty_text.len(), 0);
    assert_eq!(empty_text.find_byte(b'a'), None);
    assert_eq!(empty_text.byte_at(0), None);
    assert!(!empty_text.contains_byte(b'a'));

    // Single byte
    let single_text = build_tree(&[b"a"]).full_text();
    assert_eq!(single_text.len(), 1);
    assert!(!single_text.is_empty());
    assert_eq!(single_text.byte_at(0), Some(b'a'));
    assert_eq!(single_text.find_byte(b'a'), Some(0));
    assert!(single_text.contains_byte(b'a'));

    // Text with only empty tokens
    let empty_tokens_text = build_tree(&[b"", b"", b""]).full_text();
    assert!(empty_tokens_text.is_empty());
    assert_eq!(empty_tokens_text.len(), 0);
}
