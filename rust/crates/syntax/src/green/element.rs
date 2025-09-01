use crate::{
    GreenToken, NodeOrToken,
    green::{GreenNode, node::GreenNodeData, token::GreenTokenData},
};

pub(super) type GreenElement = NodeOrToken<GreenNode, GreenToken>;
pub(super) type GreenElementRef<'a> = NodeOrToken<&'a GreenNodeData, &'a GreenTokenData>;
