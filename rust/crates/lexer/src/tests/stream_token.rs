use super::utils::{assert_nodes_equal, generate_node_from_lexer};
use crate::Lexer;
use syntax::{SyntaxKind, tree};

/// Tests for PDF stream tokens (RawStreamDataToken)
///
/// Per ISO 32000-2:2020, §7.3.8 Stream Objects:
/// A stream shall consist of a dictionary followed by zero or more bytes bracketed between
/// the keywords `stream` (followed by newline) and `endstream`:
///
/// ```text
/// dictionary
/// stream
/// …Zero or more bytes…
/// endstream
/// ```
///
/// The RawStreamDataToken captures the entire stream content including:
/// - The `stream` keyword
/// - The mandatory end-of-line (LF or CRLF) after `stream`
/// - Zero or more bytes of stream data
/// - The `endstream` keyword
///
/// The stream length is determined by the Length entry in the stream dictionary.
/// Per §7.3.8.2: "Length shall be the number of bytes from the beginning of the line
/// following the keyword stream to the last byte just before the keyword endstream."

// ============================================================================
// Basic Stream Token Recognition
// ============================================================================

#[test]
fn test_scan_stream_when_empty_stream_with_lf_expect_stream_keyword_data_and_endstream() {
    // Minimal stream: `stream\n` + data (none) + `endstream`
    let mut lexer = Lexer::new(b"stream\nendstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b""),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_empty_stream_with_crlf_expect_stream_keyword_data_and_endstream() {
    // Minimal stream with CRLF: `stream\r\n` + data (none) + `endstream`
    let mut lexer = Lexer::new(b"stream\r\nendstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\r\n")
            },
            (SyntaxKind::RawStreamDataToken, b""),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_stream_with_simple_data_expect_stream_keyword_data_and_endstream() {
    // Stream with simple ASCII content
    let mut lexer = Lexer::new(b"stream\nHello, World!endstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"Hello, World!"),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_stream_with_binary_data_expect_stream_keyword_data_and_endstream() {
    // Stream with binary content (raw bytes)
    let data = b"stream\n\x00\x01\x02\xff\xfe\xfdendstream";
    let mut lexer = Lexer::new(data);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"\x00\x01\x02\xff\xfe\xfd"),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Stream with Line Breaks and Whitespace
// ============================================================================

#[test]
fn test_scan_stream_when_stream_with_multiple_lines_expect_stream_keyword_data_and_endstream() {
    // Stream with newlines in content
    let mut lexer = Lexer::new(b"stream\nLine 1\nLine 2\nLine 3endstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"Line 1\nLine 2\nLine 3"),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_stream_with_eol_before_endstream_expect_stream_keyword_data_with_trailing_eol_and_endstream() {
    // Per ISO 32000-2:2020: "There should be an end-of-line marker after the data
    // and before endstream; this marker shall not be included in the stream length."
    // The EOL before endstream is part of the RawStreamDataToken (or trivia before endstream)
    let mut lexer = Lexer::new(b"stream\ndata content\nendstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"data content"),
            (SyntaxKind::EndStreamKeyword) => {
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                text(b"endstream")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_stream_with_spaces_in_data_expect_stream_keyword_data_and_endstream() {
    // Stream with spaces and tabs in content
    let mut lexer = Lexer::new(b"stream\ndata  \t  with\twhitespaceendstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"data  \t  with\twhitespace"),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Stream with Special PDF Content
// ============================================================================

#[test]
fn test_scan_stream_when_stream_with_pdf_operators_expect_stream_keyword_data_and_endstream() {
    // Stream containing PDF content stream operators
    // Example: simple graphics operators
    let mut lexer = Lexer::new(b"stream\nBT\n/F1 12 Tf\n100 700 Td\n(Hello) TjETendstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"BT\n/F1 12 Tf\n100 700 Td\n(Hello) TjET"),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_stream_with_hex_data_expect_stream_keyword_data_and_endstream() {
    // Stream with hexadecimal-like content
    let mut lexer = Lexer::new(b"stream\n<48656C6C6F>endstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"<48656C6C6F>"),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Stream with Various End-of-Line Markers
// ============================================================================

#[test]
fn test_scan_stream_when_stream_with_crlf_eol_after_keyword_expect_stream_keyword_data_and_endstream() {
    // CRLF after stream keyword
    let mut lexer = Lexer::new(b"stream\r\nBinary data hereendstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\r\n")
            },
            (SyntaxKind::RawStreamDataToken, b"Binary data here"),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_stream_with_mixed_line_endings_in_content_expect_stream_keyword_data_and_endstream() {
    // Mixed CRLF and LF in stream content
    let mut lexer = Lexer::new(b"stream\nLine1\r\nLine2\nLine3\r\nendstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"Line1\r\nLine2\nLine3"),
            (SyntaxKind::EndStreamKeyword) => {
                trivia(SyntaxKind::EndOfLineTrivia, b"\r\n"),
                text(b"endstream")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Stream with Null Bytes and Control Characters
// ============================================================================

#[test]
fn test_scan_stream_when_stream_with_null_bytes_expect_stream_keyword_data_and_endstream() {
    // Streams commonly contain null bytes (e.g., in image data)
    let data = b"stream\ndata\x00with\x00nulls\x00hereendstream";
    let mut lexer = Lexer::new(data);
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"data\x00with\x00nulls\x00here"),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_stream_with_all_byte_values_expect_stream_keyword_data_and_endstream() {
    const STREAM: &[u8] = b"stream\n";
    const ENDSTREAM: &[u8] = b"endstream";

    let mut data = STREAM.to_vec();
    // Add bytes 0-255
    for i in 0..=255 {
        data.push(i as u8);
    }
    data.extend_from_slice(ENDSTREAM);

    let data = Box::leak(data.into_boxed_slice());
    let mut lexer = Lexer::new(data);
    let actual_node = generate_node_from_lexer(&mut lexer);

    // Extract the data part (everything between 'stream\n' and 'endstream')
    let expected_data = &data[STREAM.len()..data.len() - ENDSTREAM.len()];

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, expected_data),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Stream Followed by Other Tokens
// ============================================================================

#[test]
fn test_scan_stream_when_stream_followed_by_whitespace_expect_raw_stream_token_and_trivia() {
    // Stream token followed by whitespace trivia
    let mut lexer = Lexer::new(b"stream\ndata\nendstream ");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"data"),
            (SyntaxKind::EndStreamKeyword) => {
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                text(b"endstream"),
                trivia(SyntaxKind::WhitespaceTrivia, b" ")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_stream_followed_by_newline_expect_raw_stream_token_and_trivia() {
    // Stream token followed by newline trivia (common in PDF structure)
    let mut lexer = Lexer::new(b"stream\ndata\nendstream\nendobj");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"data"),
            (SyntaxKind::EndStreamKeyword) => {
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                text(b"endstream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::IndirectEndObjectKeyword, b"endobj")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_stream_followed_by_comment_expect_raw_stream_token_and_comment() {
    // Stream followed by a comment
    let mut lexer = Lexer::new(b"stream\ndata\nendstream%comment");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"data"),
            (SyntaxKind::EndStreamKeyword) => {
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                text(b"endstream"),
                trivia(SyntaxKind::CommentTrivia, b"%comment")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Stream in Indirect Object Context
// ============================================================================

#[test]
fn test_scan_stream_when_in_indirect_object_context_expect_stream_and_endobj() {
    // Complete indirect object with stream (simplified without dictionary)
    // Real format: `N 0 obj << /Length L >> stream\n...\nendstream\nendobj`
    let mut lexer = Lexer::new(b"stream\nimage data hereendstream\nendobj");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"image data here"),
            (SyntaxKind::EndStreamKeyword) => {
                text(b"endstream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::IndirectEndObjectKeyword, b"endobj")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Large Stream Data
// ============================================================================

#[test]
fn test_scan_stream_when_large_stream_expect_raw_stream_token() {
    // Test with a larger stream (simulating image data, font data, etc.)
    const STREAM: &[u8] = b"stream\n";
    const ENDSTREAM: &[u8] = b"endstream";

    let mut stream_data = STREAM.to_vec();
    for _ in 0..1000 {
        stream_data.extend_from_slice(b"Image data line\n");
    }
    stream_data.extend_from_slice(ENDSTREAM);

    let stream_data = Box::leak(stream_data.into_boxed_slice());
    let mut lexer = Lexer::new(stream_data);
    let actual_node = generate_node_from_lexer(&mut lexer);

    // The data is everything between the EOL after stream and endstream
    let expected_data = &stream_data[STREAM.len()..stream_data.len() - ENDSTREAM.len() - 1]; // -1 for the final newline

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, expected_data),
            (SyntaxKind::EndStreamKeyword) => {
                trivia(SyntaxKind::EndOfLineTrivia, b"\n"),
                text(b"endstream")
            }
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

// ============================================================================
// Edge Cases and PDF Specification Compliance
// ============================================================================

#[test]
fn test_scan_stream_when_endstream_not_on_separate_line_expect_raw_stream_token() {
    // Per spec: endstream should ideally be on its own line, but we capture the raw token
    let mut lexer = Lexer::new(b"stream\ndataendstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"data"),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_eof_before_endstream_expect_raw_stream_token_and_eof() {
    // Missing endstream: lexer should consume remaining bytes as raw data and then emit EOF
    let mut lexer = Lexer::new(b"stream\ntruncated stream data with no end stream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"truncated stream data with no end stream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_stream_data_contains_partial_endstream_expect_raw_stream_token() {
    // Stream content that contains "end" or "stream" as data should not be confused with keywords
    // The lexer must find the actual endstream keyword
    let mut lexer = Lexer::new(b"stream\nend stream end endstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"end stream end "),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_stream_length_matches_spec_expect_raw_stream_token() {
    // Test that captures exactly what goes between stream EOL and endstream
    // Per ISO 32000-2:2020 §7.3.8.2: "Length shall be the number of bytes from the
    // beginning of the line following the keyword stream to the last byte just before
    // the keyword endstream."
    let mut lexer = Lexer::new(b"stream\n123 bytes of actual stream content here.endstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"123 bytes of actual stream content here."),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}

#[test]
fn test_scan_stream_when_stream_with_no_eol_before_endstream_expect_raw_stream_token() {
    // Some PDFs might not have EOL before endstream (non-compliant but should be handled)
    let mut lexer = Lexer::new(b"stream\ndata without EOL beforeendstream");
    let actual_node = generate_node_from_lexer(&mut lexer);

    let expected_node = tree! {
        SyntaxKind::None => {
            (SyntaxKind::StreamKeyword) => {
                text(b"stream"),
                trivia(SyntaxKind::EndOfLineTrivia, b"\n")
            },
            (SyntaxKind::RawStreamDataToken, b"data without EOL before"),
            (SyntaxKind::EndStreamKeyword, b"endstream")
        }
    };

    assert_nodes_equal(&actual_node, &expected_node);
}
