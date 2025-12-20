mod arena;
mod builder;
mod cache;
mod element;
mod macros;
mod node;
mod token;
mod trivia;

pub use self::{
    builder::GreenNodeBuilder,
    cache::GreenCache,
    element::GreenElement,
    node::GreenNode,
    token::{GreenToken, GreenTokenInTree},
    trivia::{GreenTrivia, GreenTriviaInTree, GreenTriviaList, GreenTriviaListInTree},
};

pub(crate) use self::{
    arena::GreenTree,
    element::GreenElementInTree,
    node::{GreenChild, GreenNodeInTree},
};

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);
