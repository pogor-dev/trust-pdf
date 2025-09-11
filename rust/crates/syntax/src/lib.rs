mod api;
mod arc;
mod cow_mut;
mod green;
mod node_text;
mod red;
mod utils;

pub use crate::{
    api::{Language, SyntaxElement, SyntaxNode, SyntaxToken},
    green::{GreenToken, GreenTrivia, SyntaxKind},
    node_text::SyntaxText,
    utils::{Direction, NodeOrToken, TokenAtOffset, WalkEvent},
};

/// Converts bytes to a string representation, handling PDF's mixed text encodings.
///
/// Returns UTF-8 text when valid, otherwise escapes binary data for debugging.
/// This is essential for PDF processing where content can be ASCII, UTF-8, or binary.
fn byte_to_string(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(text) => text.to_string(),
        Err(_) => {
            let mut s = String::new();
            for &byte in bytes {
                match byte {
                    b' ' | b'!'..=b'~' => s.push(byte as char),
                    b'\n' => s.push_str("\\n"),
                    b'\r' => s.push_str("\\r"),
                    b'\t' => s.push_str("\\t"),
                    _ => s.push_str(&format!("\\x{:02X}", byte)),
                }
            }
            s
        }
    }
}
