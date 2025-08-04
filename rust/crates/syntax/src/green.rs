mod element;
mod node;
mod node_child;
mod node_children;
mod token;
mod trivia;
mod trivia_child;

#[cfg(test)]
mod tests;

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);
