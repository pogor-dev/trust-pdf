#[derive(PartialEq, Eq, Hash)]
struct GreenTokenHead {
    kind: SyntaxKind,
    _c: Count<GreenToken>,
}
