use std::fmt;

use crate::{GreenNode, SyntaxKind, green::list::SyntaxList};

pub struct SyntaxListWithTwoChildren {
    child0: Box<dyn GreenNode>,
    child1: Box<dyn GreenNode>,
    full_width: usize,
}

impl SyntaxListWithTwoChildren {
    pub fn new(child0: Box<dyn GreenNode>, child1: Box<dyn GreenNode>) -> Self {
        let full_width = child0.full_width() + child1.full_width();
        Self { child0, child1, full_width }
    }
}

impl SyntaxList for SyntaxListWithTwoChildren {}

impl GreenNode for SyntaxListWithTwoChildren {
    fn kind(&self) -> SyntaxKind {
        <Self as SyntaxList>::kind(self)
    }

    fn full_width(&self) -> usize {
        self.full_width
    }

    fn slot_count(&self) -> usize {
        2
    }

    fn slot(&self, index: usize) -> Option<&U> {
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

impl Clone for SyntaxListWithTwoChildren {
    fn clone(&self) -> Self {
        Self {
            child0: self.child0.clone(),
            child1: self.child1.clone(),
            full_width: self.full_width,
        }
    }
}

impl PartialEq for SyntaxListWithTwoChildren {
    fn eq(&self, other: &Self) -> bool {
        self.full_width == other.full_width && self.child0 == other.child0 && self.child1 == other.child1
    }
}

impl Eq for SyntaxListWithTwoChildren {}

unsafe impl Send for SyntaxListWithTwoChildren {}
unsafe impl Sync for SyntaxListWithTwoChildren {}

impl fmt::Debug for SyntaxListWithTwoChildren {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyntaxListWithTwoChildren")
            .field("child0", &self.child0)
            .field("child1", &self.child1)
            .finish()
    }
}
