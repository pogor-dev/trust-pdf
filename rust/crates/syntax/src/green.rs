mod list;
mod node;
mod node_trait;
mod token;
mod trivia;
mod utils;

pub use self::{
    list::{GreenList, SyntaxList, SyntaxListWithTwoChildren},
    node::GreenNode,
    node_trait::GreenNodeTrait,
    token::GreenToken,
    trivia::GreenTrivia,
    utils::{EitherNodeOrToken, ItemOrList},
};

type Trivia<'a> = ItemOrList<GreenTrivia<'a>, GreenList<'a>>;
type Node<'a> = ItemOrList<GreenNode<'a>, GreenList<'a>>;
type NodeOrToken<'a> = EitherNodeOrToken<Node<'a>, GreenToken<'a>>;

pub fn get_first_non_null_child_index<'a, T: GreenNodeTrait<'a>>(node: &T) -> u8 {
    for i in 0..node.slot_count() {
        if node.slot(i).is_some() {
            return i;
        }
    }
    0 // If no children found
}

pub fn get_last_non_null_child_index<'a, T: GreenNodeTrait<'a>>(node: &T) -> u8 {
    for i in (0..node.slot_count()).rev() {
        if node.slot(i).is_some() {
            return i;
        }
    }
    0 // If no children found
}

pub fn get_first_terminal<'a, T: GreenNodeTrait<'a>>(node: &T) -> Option<&GreenToken> {
    // let mut node = Some(node);

    // loop {
    //     let current = node?;
    //     let mut first_child = None;

    //     for i in 0..current.slot_count() {
    //         if let Some(child) = current.slot(i) {
    //             first_child = Some(child);
    //             break;
    //         }
    //     }

    //     node = first_child;

    //     // Optimization: if no children or reached terminal, stop
    //     if node.map(|n| n.slot_count()).unwrap_or(0) == 0 {
    //         break;
    //     }
    // }

    None
}
