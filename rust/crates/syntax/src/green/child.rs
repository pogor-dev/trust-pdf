#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub(crate) enum GreenChild {
    Node { rel_offset: u32, node: GreenNode },
    Token { rel_offset: u32, token: GreenToken },
}

impl GreenChild {
    #[inline]
    pub(crate) fn as_ref(&self) -> GreenElementRef {}

    #[inline]
    pub(crate) fn rel_offset(&self) -> u32 {}

    #[inline]
    fn rel_range(&self) -> Range {}
}
