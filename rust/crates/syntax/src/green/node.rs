/// Internal node in the immutable tree.
/// It has other nodes and tokens as children.
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GreenNode {
    ptr: ThinArc<GreenNodeHead, GreenChild>,
}

impl GreenNode {
    /// Creates new Node.
    #[inline]
    pub fn new<I>(kind: SyntaxKind, children: I) -> GreenNode
    where
        I: IntoIterator<Item = GreenElement>,
        I::IntoIter: ExactSizeIterator,
    {
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenNode) -> ptr::NonNull<GreenNodeData> {}

    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenNodeData>) -> GreenNode {}
}

impl Borrow<GreenNodeData> for GreenNode {
    #[inline]
    fn borrow(&self) -> &GreenNodeData {}
}

impl From<Cow<'_, GreenNodeData>> for GreenNode {
    #[inline]
    fn from(cow: Cow<'_, GreenNodeData>) -> Self {}
}

impl fmt::Debug for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

impl fmt::Display for GreenNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

impl ops::Deref for GreenNode {
    type Target = GreenNodeData;

    #[inline]
    fn deref(&self) -> &GreenNodeData {}
}
