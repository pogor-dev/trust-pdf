mod element;
mod node;
mod token;
mod trivia;

#[cfg(test)]
mod tests;

/// SyntaxKind is a type tag for each token or node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SyntaxKind(pub u16);
