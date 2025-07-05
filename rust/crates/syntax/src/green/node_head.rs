#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct GreenNodeHead {
    kind: SyntaxKind,
    text_len: u32,
    _c: Count<GreenNode>,
}

type Repr = HeaderSlice<GreenNodeHead, [GreenChild]>;
type ReprThin = HeaderSlice<GreenNodeHead, [GreenChild; 0]>;
