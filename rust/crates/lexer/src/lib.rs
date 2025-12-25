mod cursor;
mod lexer;

pub use crate::lexer::Lexer;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
