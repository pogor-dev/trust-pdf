type Repr = HeaderSlice<GreenTokenHead, [u8]>;
type ReprThin = HeaderSlice<GreenTokenHead, [u8; 0]>;

/// Leaf node in the immutable tree.
#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub struct GreenToken {
    ptr: ThinArc<GreenTokenHead, u8>,
}

impl GreenToken {
    /// Creates new Token.
    #[inline]
    pub fn new(kind: SyntaxKind, text: &[u8]) -> GreenToken {}
    #[inline]
    pub(crate) fn into_raw(this: GreenToken) -> ptr::NonNull<GreenTokenData> {}

    /// # Safety
    ///
    /// This function uses `unsafe` code to create an `Arc` from a raw pointer and then transmutes it into a `ThinArc`.
    ///
    /// - The raw pointer must be valid and correctly aligned for the type `ReprThin`.
    /// - The lifetime of the raw pointer must outlive the lifetime of the `Arc` created from it.
    /// - The transmute operation must be safe, meaning that the memory layout of `Arc<ReprThin>` must be compatible with `ThinArc<GreenTokenHead, u8>`.
    ///
    /// Failure to uphold these invariants can lead to undefined behavior.
    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenTokenData>) -> GreenToken {}
}

impl Borrow<GreenTokenData> for GreenToken {
    #[inline]
    fn borrow(&self) -> &GreenTokenData {
        self
    }
}

impl fmt::Debug for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

impl fmt::Display for GreenToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

impl ops::Deref for GreenToken {
    type Target = GreenTokenData;

    #[inline]
    fn deref(&self) -> &GreenTokenData {}
}
