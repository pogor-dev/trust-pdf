use crate::{
    arc::header_slice::HeaderSlice,
    green::{node_child::GreenChild, node_head::GreenNodeHead},
};

mod element;
mod element_ref;
mod node;
mod node_child;
mod node_children;
mod node_data;
mod node_head;
mod token;
mod trivia;
mod trivia_child;

#[cfg(test)]
mod tests;

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);

type GreenNodeRepr = HeaderSlice<GreenNodeHead, [GreenChild]>;
type GreenNodeReprThin = HeaderSlice<GreenNodeHead, [GreenChild; 0]>;
