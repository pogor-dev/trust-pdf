#[repr(transparent)]
pub struct GreenTokenData {
    data: ReprThin,
}

impl PartialEq for GreenTokenData {
    fn eq(&self, other: &Self) -> bool {}
}

impl ToOwned for GreenTokenData {
    type Owned = GreenToken;

    #[inline]
    fn to_owned(&self) -> GreenToken {}
}

impl fmt::Debug for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

impl fmt::Display for GreenTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}

impl GreenTokenData {
    /// Kind of this Token.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {}

    /// Text of this Token.
    #[inline]
    pub fn text(&self) -> &[u8] {}

    /// Returns the length of the text covered by this token.
    #[inline]
    pub fn text_len(&self) -> u32 {}
}
