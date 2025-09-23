use hashbrown::hash_map::RawEntryMut;
use rustc_hash::FxHasher;
use std::hash::{BuildHasherDefault, Hash, Hasher};

use crate::{
    GreenToken, GreenTrivia, NodeOrToken, SyntaxKind,
    green::{GreenElementRef, GreenNode, GreenNodeData, GreenTokenData, Slot, element::GreenElement, trivia::GreenTriviaPiece},
};

type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;

#[derive(Debug)]
struct NoHash<T>(T);

#[derive(Default, Debug)]
pub struct NodeCache {
    nodes: HashMap<NoHash<GreenNode>, ()>,
    tokens: HashMap<NoHash<GreenToken>, ()>,
    trivias: HashMap<NoHash<GreenTriviaPiece>, ()>,
}

impl NodeCache {
    pub(crate) fn node(&mut self, kind: SyntaxKind, children: &mut Vec<(u64, GreenElement)>, first_child: usize) -> (u64, GreenNode) {
        let build_node = move |children: &mut Vec<(u64, GreenElement)>| GreenNode::new_list(kind, children.drain(first_child..).map(|(_, it)| it));

        let children_ref = &children[first_child..];
        if children_ref.len() > 3 {
            let node = build_node(children);
            return (0, node);
        }

        let hash = {
            let mut h = FxHasher::default();
            kind.hash(&mut h);
            for &(hash, _) in children_ref {
                if hash == 0 {
                    let node = build_node(children);
                    return (0, node);
                }
                hash.hash(&mut h);
            }
            h.finish()
        };

        // Green nodes are fully immutable, so it's ok to deduplicate them.
        // This is the same optimization that Roslyn does
        // https://github.com/KirillOsenkov/Bliki/wiki/Roslyn-Immutable-Trees
        let entry = self.nodes.raw_entry_mut().from_hash(hash, |node| {
            node.0.kind() == kind && node.0.slots().len() == children_ref.len() && {
                let lhs = node.0.slots().map(|it| it.as_ref());
                let rhs = children_ref.iter().map(|(_, it)| it.as_deref());

                let lhs = lhs.map(element_id);
                let rhs = rhs.map(element_id);

                lhs.eq(rhs)
            }
        });

        let node = match entry {
            RawEntryMut::Occupied(entry) => {
                drop(children.drain(first_child..));
                entry.key().0.clone()
            }
            RawEntryMut::Vacant(entry) => {
                let node = build_node(children);
                entry.insert_with_hasher(hash, NoHash(node.clone()), (), |n| node_hash(&n.0));
                node
            }
        };

        (hash, node)
    }

    pub(crate) fn token(&mut self, kind: SyntaxKind, text: &[u8]) -> (u64, GreenToken) {
        let hash = {
            let mut h = FxHasher::default();
            kind.hash(&mut h);
            text.hash(&mut h); // TODO: TRIVIA?
            h.finish()
        };

        let entry = self
            .tokens
            .raw_entry_mut()
            .from_hash(hash, |token| token.0.kind() == kind && token.0.text() == text);

        let token = match entry {
            RawEntryMut::Occupied(entry) => entry.key().0.clone(),
            RawEntryMut::Vacant(entry) => {
                let token = GreenToken::new(kind, text);
                entry.insert_with_hasher(hash, NoHash(token.clone()), (), |t| token_hash(&t.0));
                token
            }
        };

        (hash, token)
    }

    pub(crate) fn trivia(&mut self, kind: SyntaxKind, text: &[u8]) -> (u64, GreenTriviaPiece) {
        let hash = {
            let mut h = FxHasher::default();
            kind.hash(&mut h);
            text.hash(&mut h);
            h.finish()
        };

        let entry = self
            .trivias
            .raw_entry_mut()
            .from_hash(hash, |trivia| trivia.0.kind() == kind && trivia.0.full_text() == text);

        let trivia = match entry {
            RawEntryMut::Occupied(entry) => entry.key().0.clone(),
            RawEntryMut::Vacant(entry) => {
                let trivia = GreenTriviaPiece::new(kind, text);
                entry.insert_with_hasher(hash, NoHash(trivia.clone()), (), |t| trivia_hash(&t.0));
                trivia
            }
        };

        (hash, trivia)
    }
}

fn token_hash(token: &GreenTokenData) -> u64 {
    let mut h = FxHasher::default();
    token.kind().hash(&mut h);
    token.full_text().hash(&mut h);
    h.finish()
}

fn trivia_hash(trivia: &GreenTriviaPiece) -> u64 {
    let mut h = FxHasher::default();
    trivia.kind().hash(&mut h);
    trivia.full_text().hash(&mut h);
    h.finish()
}

fn node_hash(node: &GreenNodeData) -> u64 {
    let mut h = FxHasher::default();
    node.kind().hash(&mut h);
    for child in node.slots() {
        match child {
            Slot::Node { node, .. } => node_hash(node),
            Slot::Token { token, .. } => token_hash(token),
        }
        .hash(&mut h)
    }
    h.finish()
}

fn element_id(elem: GreenElementRef<'_>) -> *const () {
    match elem {
        NodeOrToken::Node(it) => it as *const GreenNodeData as *const (),
        NodeOrToken::Token(it) => it as *const GreenTokenData as *const (),
    }
}
