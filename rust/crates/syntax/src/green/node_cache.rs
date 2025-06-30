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
}

fn token_hash(token: &GreenTokenData) -> u64 {}

fn node_hash(node: &GreenNodeData) -> u64 {}

fn element_id(elem: GreenElementRef<'_>) -> *const () {}

impl NodeCache {
    pub(crate) fn node(
        &mut self,
        kind: SyntaxKind,
        children: &mut Vec<(u64, GreenElement)>,
        first_child: usize,
    ) -> (u64, GreenNode) {
    }

    pub(crate) fn token(&mut self, kind: SyntaxKind, text: &[u8]) -> (u64, GreenToken) {}
}
