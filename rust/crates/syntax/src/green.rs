mod arena;
mod element;
mod node;
mod token;
mod trivia;

pub use self::{
    node::GreenNode,
    token::GreenToken,
    trivia::{GreenTrivia, GreenTriviaList},
};

pub(crate) use self::element::GreenElement;

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);
