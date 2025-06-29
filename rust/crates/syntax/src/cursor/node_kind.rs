use std::rc::Rc;

use crate::cursor::{
    green_element::GreenElement, node_data::NodeData, weak_green_element::WeakGreenElement,
};

/// A single NodeData (red node) is either a "root node" (no parent node and
/// holds a strong reference to the root of the green tree) or a "child node"
/// (holds a strong reference to its parent red node and a weak reference to its
/// counterpart green node)
#[derive(Debug)]
pub(crate) enum NodeKind {
    Root {
        green: GreenElement,
    },
    Child {
        green: WeakGreenElement,
        parent: Rc<NodeData>,
    },
}
