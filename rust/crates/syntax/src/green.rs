mod element;
mod node;
mod token;
mod trivia;

use std::fmt;

pub use self::{node::GreenNode, token::GreenToken, trivia::GreenTrivia};

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);

/// Converts bytes to a string representation, handling PDF's mixed text encodings.
///
/// Returns UTF-8 text when valid, otherwise escapes binary data for debugging.
/// This is essential for PDF processing where content can be ASCII, UTF-8, or binary.
fn byte_to_string(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match std::str::from_utf8(bytes) {
        Ok(text) => write!(f, "{}", text),
        Err(_) => {
            for &byte in bytes {
                match byte {
                    b' ' | b'!'..=b'~' => write!(f, "{}", byte as char)?,
                    b'\n' => write!(f, "\\n")?,
                    b'\r' => write!(f, "\\r")?,
                    b'\t' => write!(f, "\\t")?,
                    _ => write!(f, "\\x{:02X}", byte)?,
                }
            }
            write!(f, "")
        }
    }
}
