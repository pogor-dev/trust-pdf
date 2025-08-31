mod list;
mod node;
mod node_trait;
mod token;
mod trivia;
mod utils;

use std::io;

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

fn get_first_non_null_child_index<'a, T: GreenNodeTrait<'a>>(node: &T) -> u8 {
    for i in 0..node.slot_count() {
        if node.slot(i).is_some() {
            return i;
        }
    }
    0 // If no children found
}

fn get_last_non_null_child_index<'a, T: GreenNodeTrait<'a>>(node: &T) -> u8 {
    for i in (0..node.slot_count()).rev() {
        if node.slot(i).is_some() {
            return i;
        }
    }
    0 // If no children found
}

fn get_first_terminal<'a, T: GreenNodeTrait<'a>>(node: &T) -> Option<GreenToken<'a>> {
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

fn get_last_terminal<'a, T: GreenNodeTrait<'a>>(node: &T) -> Option<GreenToken<'a>> {
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

// fn write_to<'a, W: io::Write>(node: &NodeOrToken<'a>, writer: &mut W, leading: bool, trailing: bool) -> io::Result<()> {
//     // Use explicit stack to avoid stack overflow on deeply nested structures
//     let mut stack: Vec<(NodeOrToken<'a>, bool, bool)> = Vec::new();
//     stack.push((node.clone(), leading, trailing));

//     while let Some((current_node, current_leading, current_trailing)) = stack.pop() {
//         if current_node.is_token() {
//             current_node.write_token_to(writer, current_leading, current_trailing)?;
//             continue;
//         }

//         // TODO: this will never happen?
//         if current_node.is_trivia() {
//             current_node.write_trivia_to(writer)?;
//             continue;
//         }

//         let first_index = get_first_non_null_child_index(&current_node);
//         let last_index = get_last_non_null_child_index(&current_node);

//         // Push children in reverse order (since stack is LIFO)
//         for i in (first_index..=last_index).rev() {
//             if let Some(child) = current_node.slot(i) {
//                 let first = i == first_index;
//                 let last = i == last_index;

//                 let child_leading = current_leading || !first;
//                 let child_trailing = current_trailing || !last;

//                 stack.push((child, child_leading, child_trailing));
//             }
//         }
//     }

//     Ok(())
// }
