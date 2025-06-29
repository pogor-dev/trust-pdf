use std::fmt;

pub trait Language: Sized + Clone + Copy + fmt::Debug + Eq + Ord + std::hash::Hash {
    type Kind: SyntaxKind;
    type Root: AstNode<Language = Self> + Clone + Eq + fmt::Debug;
}
