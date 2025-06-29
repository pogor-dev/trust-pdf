use std::{
    hash::{Hash, Hasher},
    ops::Range,
    ptr::NonNull,
    rc::Rc,
};

use crate::{cursor::node_data::NodeData, green::token_data::GreenTokenData};

#[derive(Clone, Debug)]
pub(crate) struct SyntaxToken {
    ptr: Rc<NodeData>,
}

impl SyntaxToken {
    #[inline]
    pub(crate) fn green(&self) -> &GreenTokenData {
        match self.data().green().as_token() {
            Some(token) => token,
            None => {
                panic!(
                    "corrupted tree: a node thinks it is a token: {:?}",
                    self.data().green().as_node().unwrap().to_string()
                );
            }
        }
    }

    pub(crate) fn key(&self) -> (NonNull<()>, u64) {
        self.data().key()
    }

    #[inline]
    pub(super) fn data(&self) -> &NodeData {
        self.ptr.as_ref()
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        self.green().text()
    }

    #[inline]
    pub fn text_range(&self) -> Range<u64> {
        self.data().text_range()
    }
}

impl Eq for SyntaxToken {}

// Identity semantics for hash & eq
impl PartialEq for SyntaxToken {
    #[inline]
    fn eq(&self, other: &SyntaxToken) -> bool {
        self.data().key() == other.data().key()
    }
}

impl Hash for SyntaxToken {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data().key().hash(state);
    }
}
