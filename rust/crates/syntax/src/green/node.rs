use std::{
    borrow::Cow,
    fmt,
    hash::Hash,
    ops::{Add, Sub},
};

use crate::{
    SyntaxKind,
    green::{GreenToken, NodeOrToken, Trivia},
};

/// Immutable syntax tree node representing PDF syntactic elements with full fidelity
///
/// Green nodes capture the complete structure of PDF files including semantically
/// significant whitespace required by ISO 32000-2. This enables round-trip editing
/// and incremental parsing while preserving PDF format correctness.
pub trait GreenNode<'a, Size = u64>: Eq + PartialEq + Clone + Hash + fmt::Debug + Send + Sync
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
    fn slot(&self, index: u8) -> Option<NodeOrToken<'a>>;

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

    fn leading_trivia(&self) -> Option<Trivia<'a>>;

    fn trailing_trivia(&self) -> Option<Trivia<'a>>;

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

    // fn get_first_non_null_child_index(node: &Self) -> u8 {
    //     for i in 0..node.slot_count() {
    //         if node.slot(i).is_some() {
    //             return i;
    //         }
    //     }
    //     0 // If no children found
    // }

    // fn get_last_non_null_child_index(node: &Self) -> u8 {
    //     for i in (0..node.slot_count()).rev() {
    //         if node.slot(i).is_some() {
    //             return i;
    //         }
    //     }
    //     0 // If no children found
    // }

    // // Default implementations for terminal finding
    // fn get_first_terminal(&self) -> Option<&Self::GreenNodeType> {
    //     let mut node: Option<&Self::GreenNodeType> = Some(self);

    //     loop {
    //         let current = node?;

    //         // Find first non-null child
    //         let mut first_child = None;
    //         let slot_count = current.slot_count();

    //         for i in 0..slot_count {
    //             if let Some(child) = current.slot(i) {
    //                 first_child = Some(child);
    //                 break;
    //             }
    //         }

    //         node = first_child;

    //         // Optimization: if no children or reached terminal, stop
    //         if node.map(|n| n.slot_count()).unwrap_or(0) == 0 {
    //             break;
    //         }
    //     }

    //     node
    // }

    // fn get_last_terminal(&self) -> Option<&GreenToken<'a>> {
    //     let mut node: Option<&Self> = Some(self);

    //     loop {
    //         let current = node?;

    //         // Find last non-null child
    //         let mut last_child = None;
    //         let slot_count = current.slot_count();

    //         for i in (0..slot_count).rev() {
    //             if let Some(child) = current.slot(i) {
    //                 last_child = Some(child);
    //                 break;
    //             }
    //         }

    //         node = last_child;

    //         // Optimization: if no children or reached terminal, stop
    //         if node.map(|n| n.slot_count()).unwrap_or(0) == 0 {
    //             break;
    //         }
    //     }

    //     node
    // }
}
