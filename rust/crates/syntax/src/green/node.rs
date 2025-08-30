use std::{
    borrow::Cow,
    ops::{Add, Sub},
};

use crate::{
    SyntaxKind,
    green::{NodeOrToken, Trivia},
};

/// Immutable syntax tree node representing PDF syntactic elements with full fidelity
///
/// Green nodes capture the complete structure of PDF files including semantically
/// significant whitespace required by ISO 32000-2. This enables round-trip editing
/// and incremental parsing while preserving PDF format correctness.
pub trait GreenNode<'a, Size = u64>
where
    // we can use arithmetic operations on Size
    Size: Copy + Add<Output = Size> + Sub<Output = Size> + Eq + Default,
{
    fn kind(&self) -> SyntaxKind;

    fn to_string(&self) -> Cow<'a, [u8]>;

    fn to_full_string(&self) -> Cow<'a, [u8]>;

    #[inline]
    fn width(&self) -> Size {
        self.full_width() - self.leading_trivia_width() - self.trailing_trivia_width()
    }

    fn full_width(&self) -> Size;

    /// Get the child node at the given slot index, if it exists.
    /// We expect up to 256 (1 byte) slots.
    fn slot(&self, index: u8) -> Option<NodeOrToken>;

    /// Get the number of child slots this node has.
    /// We expect up to 256 (1 byte) slots.
    fn slot_count(&self) -> u8;

    #[inline]
    fn is_token(&self) -> bool {
        false
    }

    #[inline]
    fn is_trivia(&self) -> bool {
        false
    }

    #[inline]
    fn is_list(&self) -> bool {
        self.kind() == SyntaxKind::List
    }

    fn leading_trivia(&'_ self) -> Option<Trivia<'_>>;

    fn trailing_trivia(&'_ self) -> Option<Trivia<'_>>;

    fn leading_trivia_width(&self) -> Size;

    fn trailing_trivia_width(&self) -> Size;

    #[inline]
    fn has_leading_trivia(&self) -> bool {
        self.leading_trivia_width() != Size::default()
    }

    #[inline]
    fn has_trailing_trivia(&self) -> bool {
        self.trailing_trivia_width() != Size::default()
    }

    // fn write_token_to<W: io::Write>(&self, _writer: &mut W, _leading: bool, _trailing: bool) -> io::Result<()>;
    // fn write_trivia_to<W: io::Write>(&self, _writer: &mut W) -> io::Result<()>;

    // fn write_to<W: io::Write>(&self, writer: &mut W, leading: bool, trailing: bool) -> io::Result<()>;

    // fn get_first_non_null_child_index(node: &Self) -> Size;

    // fn get_last_non_null_child_index(node: &Self) -> Size;

    // // Default implementations for terminal finding
    // fn get_first_terminal(&self) -> Option<GreenElement>;
    // fn get_last_terminal(&self) -> Option<GreenElement>;
}
