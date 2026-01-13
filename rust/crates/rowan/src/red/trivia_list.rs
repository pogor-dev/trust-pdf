use std::fmt;

use crate::{GreenTriviaList, SyntaxToken, SyntaxTrivia};

/// A list of trivia (leading or trailing) attached to a token.
///
/// Represents a sequence of trivia elements with position information.
#[repr(C)]
#[derive(Clone, Default)]
pub struct SyntaxTriviaList<'a> {
    underlying_node: Option<GreenTriviaList>, // 16 bytes
    token: Option<&'a SyntaxToken<'a>>,       // 8 bytes
    position: u64,                            // 8 bytes
    index: u16,                               // 2 bytes
}

impl<'a> SyntaxTriviaList<'a> {
    /// Creates a new `SyntaxTriviaList` with the given properties.
    #[inline]
    pub fn new(token: Option<&'a SyntaxToken>, underlying_node: Option<GreenTriviaList>, position: u64, index: u16) -> Self {
        Self {
            token,
            underlying_node,
            position,
            index,
        }
    }

    /// Returns a reference to the associated token.
    #[inline]
    pub fn token(&self) -> Option<&SyntaxToken<'a>> {
        self.token
    }

    /// Returns the position of this trivia list in the source.
    #[inline]
    fn position(&self) -> u64 {
        self.position
    }

    /// Returns the index of this trivia list within its token.
    #[inline]
    fn index(&self) -> u16 {
        self.index
    }

    /// Returns the full width of all trivia in this list.
    #[inline]
    pub fn full_width(&self) -> u32 {
        self.underlying_node.as_ref().map_or(0, |list| list.full_width())
    }

    /// Returns the number of trivia pieces in this list.
    #[inline]
    pub fn len(&self) -> usize {
        self.underlying_node.as_ref().map_or(0, |list| list.pieces().len())
    }

    /// Returns true if the trivia list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the trivia in this list.
    #[inline]
    pub fn iter(&self) -> TriviaListIterator<'_> {
        TriviaListIterator {
            list: self,
            index: 0,
            position: self.position,
        }
    }
}

impl<'a> PartialEq for SyntaxTriviaList<'a> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token && self.underlying_node == other.underlying_node && self.position == other.position && self.index == other.index
    }
}

impl<'a> Eq for SyntaxTriviaList<'a> {}

impl<'a> fmt::Debug for SyntaxTriviaList<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxTriviaList")
            .field("len", &self.len())
            .field("position", &self.position)
            .field("index", &self.index)
            .finish()
    }
}

impl<'a> fmt::Display for SyntaxTriviaList<'a> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.underlying_node.as_ref() {
            Some(list) => write!(f, "{}", list),
            None => Ok(()),
        }
    }
}

/// Iterator over trivia in a `SyntaxTriviaList`.
pub struct TriviaListIterator<'a> {
    list: &'a SyntaxTriviaList<'a>,
    index: usize,
    position: u64,
}

impl<'a> Iterator for TriviaListIterator<'a> {
    type Item = SyntaxTrivia<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let green_list = self.list.underlying_node.as_ref()?;
        let (list_in_tree, arena) = green_list.clone().into_raw_parts();
        let pieces = list_in_tree.pieces();

        if self.index >= pieces.len() {
            return None;
        }

        let green_trivia = pieces[self.index].to_green_trivia(arena.clone());
        let current_position = self.position;

        // Advance position for next iteration
        self.position += green_trivia.full_width() as u64;
        self.index += 1;

        Some(SyntaxTrivia::new(
            self.list.token,
            Some(green_trivia),
            current_position,
            (self.index - 1) as u16,
        ))
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_memory_layout() {
        // 16 (GreenTriviaList) + 8 (token) + 8 (position) + 2 (index) = 34 bytes
        // + 6 bytes padding for 8-byte alignment = 40 bytes total
        assert_eq!(std::mem::size_of::<SyntaxTriviaList>(), 40);
        assert_eq!(std::mem::align_of::<SyntaxTriviaList>(), 8);
    }
}
