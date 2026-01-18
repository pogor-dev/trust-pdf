use hashbrown::hash_map::RawEntryMut;
use rustc_hash::FxHasher;
use std::hash::{BuildHasherDefault, Hash, Hasher};

use crate::{GreenNode, GreenNodeData, GreenToken, GreenTokenData, GreenTrivia, GreenTriviaData, SyntaxKind, green::node::Slot};

use super::element::GreenElement;

type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;

#[derive(Debug)]
struct NoHash<T>(T);

/// Interner for GreenTokens and GreenNodes
// XXX: the impl is a bit tricky. As usual when writing interners, we want to
// store all values in one HashSet.
//
// However, hashing trees is fun: hash of the tree is recursively defined. We
// maintain an invariant -- if the tree is interned, then all of its children
// are interned as well.
//
// That means that computing the hash naively is wasteful -- we just *know*
// hashes of children, and we can re-use those.
//
// So here we use *raw* API of hashbrown and provide the hashes manually,
// instead of going via a `Hash` impl. Our manual `Hash` and the
// `#[derive(Hash)]` are actually different! At some point we had a fun bug,
// where we accidentally mixed the two hashes, which made the cache much less
// efficient.
//
// To fix that, we additionally wrap the data in `NoHash` wrapper, to make sure
// we don't accidentally use the wrong hash!
#[derive(Default, Debug)]
pub struct NodeCache {
    nodes: HashMap<NoHash<GreenNode>, ()>,
    tokens: HashMap<NoHash<GreenToken>, ()>,
    trivia: HashMap<NoHash<GreenTrivia>, ()>,
}

fn token_hash(token: &GreenTokenData) -> u64 {
    let mut h = FxHasher::default();
    token.kind().hash(&mut h);
    token.text().hash(&mut h);
    h.finish()
}

fn trivia_hash(trivia: &GreenTriviaData) -> u64 {
    let mut h = FxHasher::default();
    trivia.kind().hash(&mut h);
    trivia.text().hash(&mut h);
    h.finish()
}

fn node_hash(node: &GreenNodeData) -> u64 {
    let mut h = FxHasher::default();
    node.kind().hash(&mut h);
    for slot in node.slots() {
        match slot {
            Slot::Node { node: it, .. } => node_hash(it),
            Slot::Token { token: it, .. } => token_hash(it),
            Slot::Trivia { trivia: it, .. } => trivia_hash(it),
        }
        .hash(&mut h)
    }
    h.finish()
}

fn element_id(slot: &Slot) -> *const () {
    match slot {
        Slot::Node { node, .. } => node as *const GreenNode as *const (),
        Slot::Token { token, .. } => token as *const GreenToken as *const (),
        Slot::Trivia { trivia, .. } => trivia as *const GreenTrivia as *const (),
    }
}

impl NodeCache {
    pub(crate) fn node(&mut self, kind: SyntaxKind, children: &mut Vec<(u64, GreenElement)>, first_child: usize) -> (u64, GreenNode) {
        let build_node = move |children: &mut Vec<(u64, GreenElement)>| GreenNode::new(kind, children.drain(first_child..).map(|(_, it)| it));

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
        //
        // For example, all `#[inline]` in this file share the same green node!
        // For `libsyntax/parse/parser.rs`, measurements show that deduping saves
        // 17% of the memory for green nodes!
        let entry = self.nodes.raw_entry_mut().from_hash(hash, |node| {
            node.0.kind() == kind && node.0.slot_count() == children_ref.len() && {
                let lhs = node.0.slots().map(|slot| match slot {
                    Slot::Node { node, .. } => node as *const GreenNode as *const (),
                    Slot::Token { token, .. } => token as *const GreenToken as *const (),
                    Slot::Trivia { trivia, .. } => trivia as *const GreenTrivia as *const (),
                });
                let rhs = children_ref.iter().map(|(_, it)| match it {
                    GreenElement::Node(n) => n as *const GreenNode as *const (),
                    GreenElement::Token(t) => t as *const GreenToken as *const (),
                    GreenElement::Trivia(tr) => tr as *const GreenTrivia as *const (),
                });

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

    pub(crate) fn token(&mut self, kind: SyntaxKind, text: &[u8], leading_trivia: Option<GreenNode>, trailing_trivia: Option<GreenNode>) -> (u64, GreenToken) {
        let hash = {
            let mut h = FxHasher::default();
            kind.hash(&mut h);
            text.hash(&mut h);
            h.finish()
        };

        let entry = self
            .tokens
            .raw_entry_mut()
            .from_hash(hash, |token| token.0.kind() == kind && token.0.text() == text);

        let token = match entry {
            RawEntryMut::Occupied(entry) => entry.key().0.clone(),
            RawEntryMut::Vacant(entry) => {
                let token = GreenToken::new(kind, text, leading_trivia, trailing_trivia);
                entry.insert_with_hasher(hash, NoHash(token.clone()), (), |t| token_hash(&t.0));
                token
            }
        };

        (hash, token)
    }

    pub(crate) fn trivia(&mut self, kind: SyntaxKind, text: &[u8]) -> (u64, GreenTrivia) {
        let hash = {
            let mut h = FxHasher::default();
            kind.hash(&mut h);
            text.hash(&mut h);
            h.finish()
        };

        let entry = self
            .trivia
            .raw_entry_mut()
            .from_hash(hash, |trivia| trivia.0.kind() == kind && trivia.0.text() == text);

        let trivia = match entry {
            RawEntryMut::Occupied(entry) => entry.key().0.clone(),
            RawEntryMut::Vacant(entry) => {
                let trivia = GreenTrivia::new(kind, text);
                entry.insert_with_hasher(hash, NoHash(trivia.clone()), (), |t| trivia_hash(&t.0));
                trivia
            }
        };

        (hash, trivia)
    }
}
