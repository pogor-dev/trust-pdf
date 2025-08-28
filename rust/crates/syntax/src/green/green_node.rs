use std::{fmt, io};

use crate::SyntaxKind;

pub trait GreenNode: fmt::Debug + Eq + PartialEq + Clone + Send + Sync {
    fn kind(&self) -> SyntaxKind;

    fn to_string(&self) -> &[u8];

    fn to_full_string(&self) -> &[u8];

    #[inline]
    fn width(&self) -> usize {
        self.full_width() - self.leading_trivia_width() - self.trailing_trivia_width()
    }

    fn full_width(&self) -> usize;

    #[inline]
    fn get_slot<T: GreenNode>(&self, _index: usize) -> Option<&T> {
        None
    }

    #[inline]
    fn slot_count(&self) -> usize {
        0
    }

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

    fn leading_trivia<T: GreenNode>(&self) -> Option<&T>;

    fn trailing_trivia<T: GreenNode>(&self) -> Option<&T>;

    fn leading_trivia_width(&self) -> usize;

    fn trailing_trivia_width(&self) -> usize;

    #[inline]
    fn has_leading_trivia(&self) -> bool {
        self.leading_trivia_width() != 0
    }

    #[inline]
    fn has_trailing_trivia(&self) -> bool {
        self.trailing_trivia_width() != 0
    }

    #[inline]
    fn write_token_to<T: GreenNode, W: io::Write>(&self, _writer: &mut W, _leading: bool, _trailing: bool) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_trivia_to<W: io::Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn write_to<T: GreenNode, W: io::Write>(&self, writer: &mut W, leading: bool, trailing: bool) -> io::Result<()>
    where
        Self: Sized,
    {
        // Use explicit stack to avoid stack overflow on deeply nested structures
        let mut stack: Vec<(&Self, bool, bool)> = Vec::new();
        stack.push((self, leading, trailing));

        while let Some((current_node, current_leading, current_trailing)) = stack.pop() {
            if current_node.is_token() {
                current_node.write_token_to::<T, W>(writer, current_leading, current_trailing)?;
                continue;
            }

            if current_node.is_trivia() {
                current_node.write_trivia_to::<W>(writer)?;
                continue;
            }

            let first_index = Self::get_first_non_null_child_index(current_node);
            let last_index = Self::get_last_non_null_child_index(current_node);

            // Push children in reverse order (since stack is LIFO)
            for i in (first_index..=last_index).rev() {
                if let Some(child) = current_node.get_slot::<Self>(i) {
                    let first = i == first_index;
                    let last = i == last_index;

                    let child_leading = current_leading || !first;
                    let child_trailing = current_trailing || !last;

                    stack.push((child, child_leading, child_trailing));
                }
            }
        }

        Ok(())
    }

    fn get_first_non_null_child_index(node: &Self) -> usize
    where
        Self: Sized,
    {
        for i in 0..node.slot_count() {
            if node.get_slot::<Self>(i).is_some() {
                return i;
            }
        }
        0 // If no children found
    }

    fn get_last_non_null_child_index(node: &Self) -> usize
    where
        Self: Sized,
    {
        for i in (0..node.slot_count()).rev() {
            if node.get_slot::<Self>(i).is_some() {
                return i;
            }
        }
        0 // If no children found
    }
}
