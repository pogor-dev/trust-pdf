use std::{borrow::Cow, fmt};

use crate::{GreenNode, SyntaxKind, green::NodeOrToken};

/// A trait representing a syntax list node.
/// We expect up to 256 slots (1 byte)
pub trait SyntaxList<'a>: GreenNode<'a> {
    fn kind(&self) -> SyntaxKind {
        SyntaxKind::List
    }
}

pub struct SyntaxListWithTwoChildren<'a> {
    child0: &'a NodeOrToken<'a>,
    child1: &'a NodeOrToken<'a>,
    full_width: u64,
}

impl<'a> SyntaxList<'a> for SyntaxListWithTwoChildren<'a> {}

impl<'a> GreenNode<'a> for SyntaxListWithTwoChildren<'a> {
    fn kind(&self) -> SyntaxKind {
        <Self as SyntaxList>::kind(self)
    }

    fn full_width(&self) -> u64 {
        self.full_width
    }

    fn slot_count(&self) -> u8 {
        2
    }

    fn to_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    fn to_full_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    fn slot(&self, _index: u8) -> Option<NodeOrToken> {
        todo!()
    }

    fn leading_trivia(&'_ self) -> Option<super::Trivia<'_>> {
        todo!()
    }

    fn trailing_trivia(&'_ self) -> Option<super::Trivia<'_>> {
        todo!()
    }

    fn leading_trivia_width(&self) -> u64 {
        todo!()
    }

    fn trailing_trivia_width(&self) -> u64 {
        todo!()
    }
}

impl Clone for SyntaxListWithTwoChildren<'_> {
    fn clone(&self) -> Self {
        Self {
            child0: self.child0,
            child1: self.child1,
            full_width: self.full_width,
        }
    }
}

impl PartialEq for SyntaxListWithTwoChildren<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.full_width == other.full_width && self.child0 == other.child0 && self.child1 == other.child1
    }
}

impl Eq for SyntaxListWithTwoChildren<'_> {}

unsafe impl Send for SyntaxListWithTwoChildren<'_> {}
unsafe impl Sync for SyntaxListWithTwoChildren<'_> {}

impl fmt::Debug for SyntaxListWithTwoChildren<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxListWithTwoChildren")
            .field("child0", &self.child0)
            .field("child1", &self.child1)
            .finish()
    }
}
