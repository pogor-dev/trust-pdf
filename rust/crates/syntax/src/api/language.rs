use crate::SyntaxKind;
use std::fmt;

/// Trait representing a language for syntax tree construction.
/// Used to convert between raw and typed syntax kinds.
pub trait Language: Sized + Copy + fmt::Debug + Eq + Ord + std::hash::Hash {
    type Kind: Sized + Copy + fmt::Debug + Eq + Ord + std::hash::Hash;

    fn kind_from_raw(raw: SyntaxKind) -> Self::Kind;
    fn kind_to_raw(kind: Self::Kind) -> SyntaxKind;
}
