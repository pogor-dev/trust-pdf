use std::{fmt, io};

use crate::SyntaxKind;

pub trait GreenNode: fmt::Debug + Eq + PartialEq + Clone + Send + Sync {
    type GreenNodeType: GreenNode;

    fn kind(&self) -> SyntaxKind;

    fn to_string(&self) -> Vec<u8>
    where
        Self: GreenNode<GreenNodeType = Self>,
    {
        let mut result = Vec::new();
        let _ = self.write_to(&mut result, false, false);
        result
    }

    fn to_full_string(&self) -> Vec<u8>
    where
        Self: GreenNode<GreenNodeType = Self>,
    {
        let mut result = Vec::new();
        let _ = self.write_to(&mut result, true, true);
        result
    }

    #[inline]
    fn width(&self) -> usize
    where
        Self: GreenNode<GreenNodeType = Self>,
    {
        self.full_width() - self.leading_trivia_width() - self.trailing_trivia_width()
    }

    fn full_width(&self) -> usize;

    #[inline]
    fn slot(&self, _index: usize) -> Option<&Self::GreenNodeType> {
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

    fn leading_trivia(&self) -> Option<&Self::GreenNodeType> {
        None
    }

    fn trailing_trivia(&self) -> Option<&Self::GreenNodeType> {
        None
    }

    fn leading_trivia_width(&self) -> usize
    where
        Self: GreenNode<GreenNodeType = Self>,
    {
        if self.full_width() != 0 {
            self.get_first_terminal()
                .map(|first_terminal| first_terminal.leading_trivia_width())
                .unwrap_or(0)
        } else {
            0
        }
    }

    fn trailing_trivia_width(&self) -> usize
    where
        Self: GreenNode<GreenNodeType = Self>,
    {
        if self.full_width() != 0 {
            self.get_last_terminal().map(|last_terminal| last_terminal.trailing_trivia_width()).unwrap_or(0)
        } else {
            0
        }
    }

    #[inline]
    fn has_leading_trivia(&self) -> bool
    where
        Self: GreenNode<GreenNodeType = Self>,
    {
        self.leading_trivia_width() != 0
    }

    #[inline]
    fn has_trailing_trivia(&self) -> bool
    where
        Self: GreenNode<GreenNodeType = Self>,
    {
        self.trailing_trivia_width() != 0
    }

    #[inline]
    fn write_token_to<W: io::Write>(&self, _writer: &mut W, _leading: bool, _trailing: bool) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn write_trivia_to<W: io::Write>(&self, _writer: &mut W) -> io::Result<()> {
        Ok(())
    }

    fn write_to<W: io::Write>(&self, writer: &mut W, leading: bool, trailing: bool) -> io::Result<()>
    where
        Self: Sized,
        Self: GreenNode<GreenNodeType = Self>,
    {
        // Use explicit stack to avoid stack overflow on deeply nested structures
        let mut stack: Vec<(&Self, bool, bool)> = Vec::new();
        stack.push((self, leading, trailing));

        while let Some((current_node, current_leading, current_trailing)) = stack.pop() {
            if current_node.is_token() {
                current_node.write_token_to(writer, current_leading, current_trailing)?;
                continue;
            }

            if current_node.is_trivia() {
                current_node.write_trivia_to(writer)?;
                continue;
            }

            let first_index = Self::get_first_non_null_child_index(current_node);
            let last_index = Self::get_last_non_null_child_index(current_node);

            // Push children in reverse order (since stack is LIFO)
            for i in (first_index..=last_index).rev() {
                if let Some(child) = current_node.slot(i) {
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
            if node.slot(i).is_some() {
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
            if node.slot(i).is_some() {
                return i;
            }
        }
        0 // If no children found
    }

    // Default implementations for terminal finding
    fn get_first_terminal(&self) -> Option<&Self::GreenNodeType>
    where
        Self: GreenNode<GreenNodeType = Self>,
    {
        let mut node: Option<&Self::GreenNodeType> = Some(self);

        loop {
            let current = node?;

            // Find first non-null child
            let mut first_child = None;
            let slot_count = current.slot_count();

            for i in 0..slot_count {
                if let Some(child) = current.slot(i) {
                    first_child = Some(child);
                    break;
                }
            }

            node = first_child;

            // Optimization: if no children or reached terminal, stop
            if node.map(|n| n.slot_count()).unwrap_or(0) == 0 {
                break;
            }
        }

        node
    }

    fn get_last_terminal(&self) -> Option<&Self::GreenNodeType>
    where
        Self: GreenNode<GreenNodeType = Self>,
    {
        let mut node: Option<&Self::GreenNodeType> = Some(self);

        loop {
            let current = node?;

            // Find last non-null child
            let mut last_child = None;
            let slot_count = current.slot_count();

            for i in (0..slot_count).rev() {
                if let Some(child) = current.slot(i) {
                    last_child = Some(child);
                    break;
                }
            }

            node = last_child;

            // Optimization: if no children or reached terminal, stop
            if node.map(|n| n.slot_count()).unwrap_or(0) == 0 {
                break;
            }
        }

        node
    }
}
