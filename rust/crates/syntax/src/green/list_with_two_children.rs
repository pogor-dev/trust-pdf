use std::fmt;

use crate::{GreenNode, SyntaxKind, green::list::SyntaxList};

pub struct SyntaxListWithTwoChildren<T: GreenNode> {
    child0: T,
    child1: T,
    full_width: usize,
}

impl<T: GreenNode> SyntaxListWithTwoChildren<T> {
    pub fn new(child0: T, child1: T) -> Self {
        let full_width = child0.full_width() + child1.full_width();
        Self { child0, child1, full_width }
    }
}

impl<T: GreenNode> SyntaxList for SyntaxListWithTwoChildren<T> {}

impl<T: GreenNode> GreenNode for SyntaxListWithTwoChildren<T> {
    fn kind(&self) -> SyntaxKind {
        <Self as SyntaxList>::kind(self)
    }

    fn full_width(&self) -> usize {
        self.full_width
    }

    fn slot_count(&self) -> usize {
        2
    }

    fn slot<U: GreenNode>(&self, index: usize) -> Option<&U> {
        match index {
            0 => {
                // Safety: This is safe if T and U are the same type at runtime
                // The caller is responsible for ensuring type compatibility
                let ptr = &self.child0 as *const T as *const U;
                Some(unsafe { &*ptr })
            }
            1 => {
                let ptr = &self.child1 as *const T as *const U;
                Some(unsafe { &*ptr })
            }
            _ => None,
        }
    }
}

impl<T: GreenNode> Clone for SyntaxListWithTwoChildren<T> {
    fn clone(&self) -> Self {
        Self {
            child0: self.child0.clone(),
            child1: self.child1.clone(),
            full_width: self.full_width,
        }
    }
}

impl<T: GreenNode> PartialEq for SyntaxListWithTwoChildren<T> {
    fn eq(&self, other: &Self) -> bool {
        self.full_width == other.full_width && self.child0 == other.child0 && self.child1 == other.child1
    }
}

impl<T: GreenNode> Eq for SyntaxListWithTwoChildren<T> {}

unsafe impl<T: GreenNode> Send for SyntaxListWithTwoChildren<T> {}
unsafe impl<T: GreenNode> Sync for SyntaxListWithTwoChildren<T> {}

impl<T: GreenNode> fmt::Debug for SyntaxListWithTwoChildren<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxListWithTwoChildren")
            .field("child0", &self.child0)
            .field("child1", &self.child1)
            .finish()
    }
}
