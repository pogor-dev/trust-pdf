use std::ptr;

use crate::{
    SyntaxKind,
    green::GreenTokenData,
    red::{Green, NodeData, SyntaxNode, free},
};

#[derive(Debug)]
pub struct SyntaxToken {
    ptr: ptr::NonNull<NodeData>,
}

impl SyntaxToken {
    pub(super) fn new(green: &GreenTokenData, parent: SyntaxNode, index: u32, offset: u64) -> SyntaxToken {
        let green = Green::Token { ptr: green.into() };
        SyntaxToken {
            ptr: NodeData::new(Some(parent), index, offset, green),
        }
    }

    #[inline]
    fn data(&self) -> &NodeData {
        unsafe { self.ptr.as_ref() }
    }

    #[inline]
    pub(super) fn can_take_ptr(&self) -> bool {
        self.data().rc.get() == 1
    }

    #[inline]
    pub(super) fn take_ptr(self) -> ptr::NonNull<NodeData> {
        assert!(self.can_take_ptr());
        let ret = self.ptr;
        // don't change the refcount when self gets dropped
        std::mem::forget(self);
        ret
    }

    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data().kind()
    }

    #[inline]
    pub fn index(&self) -> u32 {
        self.data().index()
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        match self.data().green().as_token() {
            Some(it) => it.full_text(),
            None => {
                debug_assert!(
                    false,
                    "corrupted tree: a node thinks it is a token: {:?}",
                    self.data().green().as_node().unwrap().to_string()
                );
                b"".to_vec()
            }
        }
    }

    #[inline]
    pub fn green(&self) -> &GreenTokenData {
        self.data().green().into_token().unwrap()
    }

    #[inline]
    pub fn parent(&self) -> Option<SyntaxNode> {
        self.data().parent_node()
    }
}

impl Clone for SyntaxToken {
    #[inline]
    fn clone(&self) -> Self {
        self.data().inc_rc();
        SyntaxToken { ptr: self.ptr }
    }
}

impl Drop for SyntaxToken {
    #[inline]
    fn drop(&mut self) {
        if self.data().dec_rc() {
            unsafe { free(self.ptr) }
        }
    }
}
