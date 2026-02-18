use crate::{
    SyntaxKind,
    green::{flags::GreenFlags, green_node::GreenNode, green_trait::GreenNodeTrait},
};

pub(crate) struct GreenToken {
    green_node: GreenNode,
}

impl GreenToken {
    pub(super) fn new(kind: SyntaxKind) -> Self {
        let full_width = kind.get_text().len() as u32;
        let mut green_node = GreenNode::new(kind, full_width);
        green_node.set_flags(GreenFlags::IS_NOT_MISSING); // We have other struct representing missing tokens.

        GreenToken { green_node }
    }
}

impl GreenNodeTrait for GreenToken {
    #[inline]
    fn kind(&self) -> SyntaxKind {
        self.green_node.kind()
    }

    #[inline]
    fn flags(&self) -> GreenFlags {
        self.green_node.flags()
    }

    #[inline]
    fn set_flags(&mut self, flags: GreenFlags) {
        self.green_node.set_flags(flags);
    }

    #[inline]
    fn is_token(&self) -> bool {
        true
    }

    #[inline]
    fn get_slot_count(&self) -> u32 {
        // Token nodes should never have slots, so we can return
        0
    }

    #[inline]
    fn set_slot_count(&mut self, _value: u8) {
        // Token nodes should never have slots, so we can ignore this.
    }

    #[inline]
    fn get_slot(&self, _index: usize) -> Option<GreenNode> {
        // Token nodes should never have slots, so we can return None for any index.
        None
    }

    #[inline]
    fn text(&self) -> Vec<u8> {
        self.kind().get_text().to_vec()
    }

    #[inline]
    fn full_text(&self) -> Vec<u8> {
        self.text()
    }

    #[inline]
    fn width(&self) -> u32 {
        self.text().len() as u32
    }

    #[inline]
    fn full_width(&self) -> u32 {
        self.green_node.full_width()
    }

    #[inline]
    fn get_leading_trivia_width(&self) -> u32 {
        0
    }

    #[inline]
    fn get_trailing_trivia_width(&self) -> u32 {
        0
    }

    #[inline]
    fn write_token_to(&self, _leading: bool, _trailing: bool) -> Vec<u8> {
        self.text().clone()
    }
}
