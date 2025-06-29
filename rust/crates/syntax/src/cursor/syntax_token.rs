use std::rc::Rc;

use crate::cursor::syntax_node_data::NodeData;

#[derive(Clone, Debug)]
pub(crate) struct SyntaxToken {
    ptr: Rc<NodeData>,
}
