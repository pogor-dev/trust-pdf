use hashbrown::hash_map::RawEntryMut;
use rustc_hash::FxHasher;
use triomphe::UniqueArc;

use std::hash::{BuildHasherDefault, Hash, Hasher};

use crate::{
    GreenNode, NodeOrToken, SyntaxKind,
    green::{GreenElement, arena::GreenTree, node::GreenChild, token::GreenTokenInTree, trivia::GreenTriviaInTree},
};

type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;

#[derive(Debug)]
struct NoHash<T>(T);

pub struct GreenCache {
    nodes: HashMap<NoHash<GreenNode>, ()>,
    tokens: HashMap<NoHash<GreenTokenInTree>, ()>,
    trivias: HashMap<NoHash<GreenTriviaInTree>, ()>,
    pub(super) arena: UniqueArc<GreenTree>,
}

impl Default for GreenCache {
    #[inline]
    fn default() -> Self {
        Self {
            nodes: HashMap::default(),
            tokens: HashMap::default(),
            trivias: HashMap::default(),
            arena: GreenTree::new(),
        }
    }
}

impl GreenCache {
    pub(crate) fn trivia(&mut self, kind: SyntaxKind, text: &[u8]) -> (u64, GreenTriviaInTree) {
        let hash = {
            let mut h = FxHasher::default();
            kind.hash(&mut h);
            text.hash(&mut h);
            h.finish()
        };

        let entry = self
            .trivias
            .raw_entry_mut()
            .from_hash(hash, |trivia| trivia.0.kind() == kind && trivia.0.bytes() == text);

        let trivia = match entry {
            RawEntryMut::Occupied(entry) => entry.key().0,
            RawEntryMut::Vacant(entry) => {
                let trivia = self.arena.alloc_trivia(kind, text);
                entry.insert_with_hasher(hash, NoHash(trivia), (), |t| trivia_hash(&t.0));
                trivia
            }
        };

        (hash, trivia)
    }

    pub(crate) fn token(
        &mut self,
        kind: SyntaxKind,
        text: &[u8],
        leading_trivia: &[GreenTriviaInTree],
        trailing_trivia: &[GreenTriviaInTree],
    ) -> (u64, GreenTokenInTree) {
        let hash = {
            let mut h = FxHasher::default();
            kind.hash(&mut h);
            text.hash(&mut h);
            h.finish()
        };

        let entry = self
            .tokens
            .raw_entry_mut()
            .from_hash(hash, |token| token.0.kind() == kind && token.0.bytes().as_slice() == text);

        let token = match entry {
            RawEntryMut::Occupied(entry) => entry.key().0,
            RawEntryMut::Vacant(entry) => {
                let leading_trivia_list = self.arena.alloc_trivia_list(leading_trivia);
                let trailing_trivia_list = self.arena.alloc_trivia_list(trailing_trivia);
                let token = self.arena.alloc_token(kind, text, leading_trivia_list, trailing_trivia_list);
                entry.insert_with_hasher(hash, NoHash(token), (), |t| token_hash(&t.0));
                token
            }
        };

        (hash, token)
    }

    pub(crate) fn node(&mut self, kind: SyntaxKind, children: &mut Vec<(u64, GreenElement)>, first_child: usize) -> (u64, GreenNode) {
        let mut build_node = |children: &mut Vec<(u64, GreenElement)>| {
            let full_width = children[first_child..].iter().map(|(_, child)| child.full_width()).sum();

            let mut rel_offset = 0;
            let children = children.drain(first_child..).map(|(_, child)| match child {
                NodeOrToken::Node(node) => {
                    let offset = rel_offset;
                    rel_offset += node.full_width();
                    GreenChild::Node { rel_offset: offset, node }
                }
                NodeOrToken::Token(token) => {
                    let offset = rel_offset;
                    rel_offset += token.full_width();
                    GreenChild::Token { rel_offset: offset, token }
                }
            });

            self.arena.alloc_node(kind, full_width, children.len() as u16, children)
        };

        let children_ref = &children[first_child..];

        // If there are too many children, skip caching.
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
            node.0.kind() == kind && node.0.children().len() == children_ref.len() && {
                let lhs = node.0.children();
                let rhs = children_ref.iter().map(|(_, it)| it);

                let lhs = lhs
                    .iter()
                    .map(|it| match it {
                        GreenChild::Node { rel_offset: _, node } => NodeOrToken::Node(node),
                        GreenChild::Token { rel_offset: _, token } => NodeOrToken::Token(token),
                    })
                    .map(element_id);
                let rhs = rhs.map(|it| element_id(it.as_ref()));

                lhs.eq(rhs)
            }
        });

        let node = match entry {
            RawEntryMut::Occupied(entry) => {
                drop(children.drain(first_child..));
                entry.key().0
            }
            RawEntryMut::Vacant(entry) => {
                let node = build_node(children);
                entry.insert_with_hasher(hash, NoHash(node), (), |n| node_hash(&n.0));
                node
            }
        };

        (hash, node)
    }
}

fn trivia_hash(trivia: &GreenTriviaInTree) -> u64 {
    let mut h = FxHasher::default();
    trivia.kind().hash(&mut h);
    trivia.bytes().hash(&mut h);
    h.finish()
}

fn token_hash(token: &GreenTokenInTree) -> u64 {
    let mut h = FxHasher::default();
    token.kind().hash(&mut h);
    token.bytes().hash(&mut h);

    for piece in token.leading_trivia().pieces() {
        trivia_hash(piece).hash(&mut h);
    }

    for piece in token.trailing_trivia().pieces() {
        trivia_hash(piece).hash(&mut h);
    }

    h.finish()
}

fn node_hash(node: &GreenNode) -> u64 {
    let mut h = FxHasher::default();
    node.kind().hash(&mut h);
    for child in node.children() {
        match child {
            GreenChild::Node { rel_offset: _, node } => node_hash(node),
            GreenChild::Token { rel_offset: _, token } => token_hash(token),
        }
        .hash(&mut h)
    }
    h.finish()
}

fn element_id(elem: NodeOrToken<&GreenNode, &GreenTokenInTree>) -> *const () {
    match elem {
        NodeOrToken::Node(it) => it.data.as_ptr().cast(),
        NodeOrToken::Token(it) => it.data.as_ptr().cast(),
    }
}
