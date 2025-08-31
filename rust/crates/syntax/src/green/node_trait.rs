use std::{
    borrow::Cow,
    fmt,
    hash::Hash,
    io,
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
pub trait GreenNodeTrait<'a, Size = u64>: Eq + PartialEq + Clone + Hash + fmt::Debug + Send + Sync
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

    #[inline]
    fn write_token_to<W: io::Write>(&self, _writer: &mut W, _leading: bool, _trailing: bool) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_trivia_to<W: io::Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }

    // fn write_to<W: io::Write>(&self, writer: &mut W, leading: bool, trailing: bool) -> io::Result<()>
    // where
    //     Self: Sized,
    //     Self: GreenNodeTrait<'a>,
    // {
    //     // Use explicit stack to avoid stack overflow on deeply nested structures
    //     let mut stack: Vec<(&Self, bool, bool)> = Vec::new();
    //     stack.push((self, leading, trailing));

    //     while let Some((current_node, current_leading, current_trailing)) = stack.pop() {
    //         if current_node.is_token() {
    //             current_node.write_token_to(writer, current_leading, current_trailing)?;
    //             continue;
    //         }

    //         if current_node.is_trivia() {
    //             current_node.write_trivia_to(writer)?;
    //             continue;
    //         }

    //         let first_index = get_first_non_null_child_index(current_node);
    //         let last_index = get_last_non_null_child_index(current_node);

    //         // Push children in reverse order (since stack is LIFO)
    //         for i in (first_index..=last_index).rev() {
    //             if let Some(child) = current_node.slot(i) {
    //                 let first = i == first_index;
    //                 let last = i == last_index;

    //                 let child_leading = current_leading || !first;
    //                 let child_trailing = current_trailing || !last;

    //                 stack.push((child, child_leading, child_trailing));
    //             }
    //         }
    //     }

    //     Ok(())
    // }
}
