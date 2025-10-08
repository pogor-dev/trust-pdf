mod arena;
mod node;
mod token;
mod trivia;

pub use {
    node::GreenNode,
    token::GreenToken,
    trivia::{GreenTrivia, GreenTriviaList},
};

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);
