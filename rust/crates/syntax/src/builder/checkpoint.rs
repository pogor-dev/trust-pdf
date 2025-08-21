use std::{cell::Cell, rc::Rc};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Checkpoint<P>(Rc<Cell<Inner<P>>>)
where
    P: Copy;

impl<P> Checkpoint<P>
where
    P: Copy,
{
    pub(crate) fn new(node: P, parent: Option<P>) -> Self {
        Self(Rc::new(Cell::new(Inner { node, parent })))
    }

    pub(crate) fn set(&self, node: P, parent: Option<P>) {
        self.0.set(Inner { node, parent });
    }

    pub(crate) fn node(&self) -> P {
        self.0.get().node
    }

    pub(crate) fn get(&self) -> (P, Option<P>) {
        let Inner { node, parent } = self.0.get();
        (node, parent)
    }
}

/// The parent of the checkpoint.
#[derive(Debug, Clone, Copy)]
struct Inner<P> {
    // The node being wrapped by the checkpoint.
    node: P,
    // The parent node of the context being checkpointed.
    parent: Option<P>,
}
