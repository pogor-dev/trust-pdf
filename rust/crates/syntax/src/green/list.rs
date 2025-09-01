use std::{borrow::Cow, fmt};

use crate::{
    GreenNodeTrait, SyntaxKind,
    green::{NodeOrToken2, Trivia, utils::IsGreenList},
};

pub trait SyntaxList<'a>: GreenNodeTrait<'a> {
    fn kind(&self) -> SyntaxKind {
        SyntaxKind::List
    }
}

#[derive(Hash)]
pub struct SyntaxListWithTwoChildren<'a> {
    child0: &'a NodeOrToken2<'a>,
    child1: &'a NodeOrToken2<'a>,
    full_width: u64,
}

impl<'a> SyntaxList<'a> for SyntaxListWithTwoChildren<'a> {}

impl<'a> GreenNodeTrait<'a> for SyntaxListWithTwoChildren<'a> {
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

    fn slot(&self, _index: u8) -> Option<NodeOrToken2<'a>> {
        todo!()
    }

    fn leading_trivia(&self) -> Option<super::Trivia<'a>> {
        todo!()
    }

    fn trailing_trivia(&self) -> Option<super::Trivia<'a>> {
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GreenList<'a> {
    ListWithTwoChildren(SyntaxListWithTwoChildren<'a>),
}

impl<'a> IsGreenList<'a> for GreenList<'a> {}

impl<'a> GreenNodeTrait<'a> for GreenList<'a> {
    #[inline]
    fn kind(&self) -> crate::SyntaxKind {
        SyntaxKind::List
    }

    #[inline]
    fn is_list(&self) -> bool {
        true
    }

    #[inline]
    fn to_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    #[inline]
    fn to_full_string(&self) -> Cow<'a, [u8]> {
        todo!()
    }

    #[inline]
    fn full_width(&self) -> u64 {
        todo!()
    }

    #[inline]
    fn slot(&self, _index: u8) -> Option<NodeOrToken2<'a>> {
        todo!()
    }

    #[inline]
    fn slot_count(&self) -> u8 {
        todo!()
    }

    #[inline]
    fn leading_trivia(&self) -> Option<Trivia<'a>> {
        todo!()
    }

    #[inline]
    fn trailing_trivia(&self) -> Option<Trivia<'a>> {
        todo!()
    }

    #[inline]
    fn leading_trivia_width(&self) -> u64 {
        todo!()
    }

    #[inline]
    fn trailing_trivia_width(&self) -> u64 {
        todo!()
    }
}
