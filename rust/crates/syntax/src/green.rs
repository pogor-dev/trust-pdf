mod builder;
mod cache;
mod element;
mod node;
mod token;
mod trivia;

pub use self::{
    builder::{Checkpoint, GreenNodeBuilder},
    cache::NodeCache,
    node::{GreenNode, GreenNodeData},
    token::{GreenToken, GreenTokenData},
    trivia::GreenTrivia,
};

pub(crate) use self::{element::GreenElementRef, node::Slot};

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);
