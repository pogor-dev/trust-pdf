use std::fmt;

use crate::green::syntax_kind::RawSyntaxKind;

/// Type tag for each node or token of a language
pub trait SyntaxKind: fmt::Debug + PartialEq + Copy {
    const EOF: Self;

    /// Converts this kind to a raw syntax kind.
    fn to_raw(&self) -> RawSyntaxKind;

    /// Creates a syntax kind from a raw kind.
    fn from_raw(raw: RawSyntaxKind) -> Self;

    /// Returns `true` if this kind is for a root node.
    fn is_root(&self) -> bool;

    /// Returns `true` if this kind is a list node.
    fn is_list(&self) -> bool;

    /// Returns a string for keywords and punctuation tokens or `None` otherwise.
    fn to_string(&self) -> Option<&'static str>;
}
