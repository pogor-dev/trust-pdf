mod list;
mod node;
mod node_trait;
mod token;
mod token2;
mod trivia;
mod trivia2;
mod trivia_list;
mod utils;

use std::fmt;

pub use self::{
    list::{GreenList, SyntaxList, SyntaxListWithTwoChildren},
    node::GreenNode,
    node_trait::GreenNodeTrait,
    token::GreenToken,
    token2::GreenToken2,
    trivia::GreenTrivia,
    trivia_list::GreenTriviaList,
    trivia2::GreenTrivia2,
    utils::{EitherNodeOrToken, ItemOrList},
};

type Trivia<'a> = ItemOrList<GreenTrivia2<'a>, GreenList<'a>>;
type Node<'a> = ItemOrList<GreenNode<'a>, GreenList<'a>>;
type NodeOrToken<'a> = EitherNodeOrToken<Node<'a>, GreenToken2<'a>>;

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

fn get_first_terminal<'a, T: GreenNodeTrait<'a>>(node: &T) -> Option<GreenToken2<'a>> {
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

fn get_last_terminal<'a, T: GreenNodeTrait<'a>>(node: &T) -> Option<GreenToken2<'a>> {
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

/// Converts bytes to a string representation, handling PDF's mixed text encodings.
///
/// Returns UTF-8 text when valid, otherwise escapes binary data for debugging.
/// This is essential for PDF processing where content can be ASCII, UTF-8, or binary.
fn byte_to_string(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match std::str::from_utf8(bytes) {
        Ok(text) => write!(f, "{}", text),
        Err(_) => {
            write!(f, "b\"")?;
            for &byte in bytes {
                match byte {
                    b' ' | b'!'..=b'~' => write!(f, "{}", byte as char)?,
                    b'\n' => write!(f, "\\n")?,
                    b'\r' => write!(f, "\\r")?,
                    b'\t' => write!(f, "\\t")?,
                    _ => write!(f, "\\x{:02x}", byte)?,
                }
            }
            write!(f, "\"")
        }
    }
}
