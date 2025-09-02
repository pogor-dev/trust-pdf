mod element;
mod node;
mod token;
mod trivia;

use std::fmt;

pub use self::{node::GreenNode, token::GreenToken, trivia::GreenTrivia};

/// Converts bytes to a string representation, handling PDF's mixed text encodings.
///
/// Returns UTF-8 text when valid, otherwise escapes binary data for debugging.
/// This is essential for PDF processing where content can be ASCII, UTF-8, or binary.
fn byte_to_string(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match std::str::from_utf8(bytes) {
        Ok(text) => write!(f, "{}", text),
        Err(_) => {
            write!(f, "b\"")?;
            for &byte in bytes {
                match byte {
                    b' ' | b'!'..=b'~' => write!(f, "{}", byte as char)?,
                    b'\n' => write!(f, "\\n")?,
                    b'\r' => write!(f, "\\r")?,
                    b'\t' => write!(f, "\\t")?,
                    _ => write!(f, "\\x{:02x}", byte)?,
                }
            }
            write!(f, "\"")
        }
    }
}
