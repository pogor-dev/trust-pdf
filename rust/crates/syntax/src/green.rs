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

pub fn get_first_terminal<'a, T: GreenNodeTrait<'a>>(node: &T) -> Option<GreenToken<'a>> {
    for i in 0..node.slot_count() {
        if let Some(child) = node.slot(i) {
            match child {
                EitherNodeOrToken::Token(token) => {
                    return Some(token);
                }
                EitherNodeOrToken::Node(node_data) => {
                    let result = match node_data {
                        ItemOrList::Item(item) => get_first_terminal(&item),
                        ItemOrList::List(list) => get_first_terminal(&list),
                    };
                    if result.is_some() {
                        return result;
                    }
                }
            }
        }
    }
    None
}

pub fn get_last_terminal<'a, T: GreenNodeTrait<'a>>(node: &T) -> Option<GreenToken<'a>> {
    for i in (0..node.slot_count()).rev() {
        if let Some(child) = node.slot(i) {
            match child {
                EitherNodeOrToken::Token(token) => {
                    return Some(token);
                }
                EitherNodeOrToken::Node(node_data) => {
                    let result = match node_data {
                        ItemOrList::Item(item) => get_last_terminal(&item),
                        ItemOrList::List(list) => get_last_terminal(&list),
                    };
                    if result.is_some() {
                        return result;
                    }
                }
            }
        }
    }
    None
}
