mod support;

use lexer::Lexer;
use support::{assert_nodes_equal, generate_node_from_lexer};
use syntax::{SyntaxKind, tree};

/// Tests for PDF structure keywords: obj, endobj, R, stream, endstream, xref, f, n, trailer, startxref
///
/// See: ISO 32000-2:2020, §7.3.10 Indirect Objects, §7.3.8 Stream Objects, §7.5.4 Cross-Reference Table

// ============================================================================
// Indirect Object Keywords (§7.3.10)
// ============================================================================

#[test]
fn test_scan_keyword_when_obj_expect_indirect_object_keyword() {
    let mut lexer = Lexer::new(b"obj");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::IndirectObjectKeyword.into(), b"obj")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_endobj_expect_indirect_end_object_keyword() {
    let mut lexer = Lexer::new(b"endobj");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::IndirectEndObjectKeyword.into(), b"endobj")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_uppercase_r_expect_indirect_reference_keyword() {
    let mut lexer = Lexer::new(b"R");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::IndirectReferenceKeyword.into(), b"R")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_indirect_object_pattern_expect_correct_tokens() {
    // Example: "1 0 obj"
    let mut lexer = Lexer::new(b"1 0 obj");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into()) => {
                text(b"1"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            },
            (SyntaxKind::NumericLiteralToken.into()) => {
                text(b"0"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            },
            (SyntaxKind::IndirectObjectKeyword.into(), b"obj")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_indirect_reference_pattern_expect_correct_tokens() {
    // Example: "5 0 R"
    let mut lexer = Lexer::new(b"5 0 R");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::NumericLiteralToken.into()) => {
                text(b"5"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            },
            (SyntaxKind::NumericLiteralToken.into()) => {
                text(b"0"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            },
            (SyntaxKind::IndirectReferenceKeyword.into(), b"R")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Stream Keywords (§7.3.8)
// ============================================================================

#[test]
fn test_scan_keyword_when_stream_expect_stream_keyword() {
    let mut lexer = Lexer::new(b"stream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StreamKeyword.into(), b"stream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_endstream_expect_end_stream_keyword() {
    let mut lexer = Lexer::new(b"endstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::EndStreamKeyword.into(), b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Cross-Reference Table Keywords (§7.5.4)
// ============================================================================

#[test]
fn test_scan_keyword_when_xref_expect_xref_keyword() {
    let mut lexer = Lexer::new(b"xref");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::XRefKeyword.into(), b"xref")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_lowercase_f_expect_xref_free_entry_keyword() {
    let mut lexer = Lexer::new(b"f");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::XRefFreeEntryKeyword.into(), b"f")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_lowercase_n_expect_xref_in_use_entry_keyword() {
    let mut lexer = Lexer::new(b"n");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::XRefInUseEntryKeyword.into(), b"n")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_trailer_expect_file_trailer_keyword() {
    let mut lexer = Lexer::new(b"trailer");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::FileTrailerKeyword.into(), b"trailer")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_startxref_expect_start_xref_keyword() {
    let mut lexer = Lexer::new(b"startxref");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StartXRefKeyword.into(), b"startxref")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Case Sensitivity Tests
// ============================================================================

#[test]
fn test_scan_keyword_when_uppercase_obj_expect_bad_token() {
    // Keywords are case-sensitive per PDF spec
    let mut lexer = Lexer::new(b"OBJ");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::BadToken.into(), b"OBJ")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_mixed_case_stream_expect_bad_token() {
    let mut lexer = Lexer::new(b"Stream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::BadToken.into(), b"Stream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_lowercase_r_expect_bad_token() {
    // 'R' must be uppercase for indirect reference
    let mut lexer = Lexer::new(b"r");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::BadToken.into(), b"r")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Delimiter and Whitespace Tests
// ============================================================================

#[test]
fn test_scan_keyword_when_obj_followed_by_delimiter_expect_obj_keyword() {
    let mut lexer = Lexer::new(b"obj<<");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::IndirectObjectKeyword.into(), b"obj"),
            (SyntaxKind::OpenDictToken.into(), b"<<")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_stream_followed_by_newline_expect_stream_keyword_with_eol() {
    let mut lexer = Lexer::new(b"stream\n");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::StreamKeyword.into()) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia.into(), b"\n")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_keyword_when_multiple_structure_keywords_expect_separate_tokens() {
    let mut lexer = Lexer::new(b"obj endobj stream endstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::LexerNode.into() => {
            (SyntaxKind::IndirectObjectKeyword.into()) => {
                text(b"obj"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            },
            (SyntaxKind::IndirectEndObjectKeyword.into()) => {
                text(b"endobj"),
                trivia(SyntaxKind::WhitespaceTrivia.into(), b" ")
            },
            (SyntaxKind::StreamKeyword.into(), b"stream"),
            (SyntaxKind::RawStreamDataToken.into(), b" "),
            (SyntaxKind::EndStreamKeyword.into(), b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
