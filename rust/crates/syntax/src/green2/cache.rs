use rustc_hash::FxHasher;
use std::hash::{BuildHasherDefault, Hash, Hasher};

use crate::SyntaxKind;

type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;

#[derive(Debug)]
struct NoHash<T>(T);

pub struct NodeCache {
    // nodes: HashMap<NoHash<GreenNodeInTree>, ()>,
    // tokens: HashMap<NoHash<GreenTokenInTree>, ()>,
    // pub(super) arena: UniqueArc<GreenTree>,
}

impl NodeCache {
    // pub(super) fn token(&mut self, kind: SyntaxKind, text: &[u8]) -> (u64, GreenToken) {
    // let hash = {
    //     let mut h = FxHasher::default();
    //     kind.hash(&mut h);
    //     text.hash(&mut h);
    //     h.finish()
    // };

    // let entry = self.tokens.raw_entry_mut().from_hash(hash, |token| {
    //     token.0.kind() == kind && token.0.text() == text
    // });

    // let token = match entry {
    //     RawEntryMut::Occupied(entry) => entry.key().0,
    //     RawEntryMut::Vacant(entry) => {
    //         let token = self.arena.alloc_token(kind, text);
    //         entry.insert_with_hasher(hash, NoHash(token), (), |t| token_hash(&t.0));
    //         token
    //     }
    // };

    // (hash, token)
    // }
}

impl Default for NodeCache {
    #[inline]
    fn default() -> Self {
        Self {
            // nodes: HashMap::default(),
            // tokens: HashMap::default(),
            // arena: GreenTree::new(),
        }
    }
}

// fn token_hash(token: &GreenTokenInTree) -> u64 {
//     let mut h = FxHasher::default();
//     token.kind().hash(&mut h);
//     token.text().hash(&mut h);
//     h.finish()
// }
