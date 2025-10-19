mod arena;
mod builder;
mod cache;
mod element;
mod node;
mod token;
mod trivia;

pub use self::{
    builder::GreenNodeBuilder,
    node::GreenNode,
    token::GreenToken,
    trivia::{GreenTrivia, GreenTriviaList},
};

pub(crate) use self::{cache::GreenCache, element::GreenElement};

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);
