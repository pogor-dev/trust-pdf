use crate::{
    SyntaxKind,
    green::{flags::GreenFlags, green_node::GreenNode, green_trait::GreenNodeTrait},
};

pub(crate) struct GreenTrivia {
    green_node: GreenNode,
    text: Vec<u8>,
}

impl GreenTrivia {
    pub(super) fn new(kind: SyntaxKind, text: &[u8]) -> Self {
        let full_width = text.len() as u32;
        let green_node = GreenNode::new(kind, full_width);

        GreenTrivia {
            green_node,
            text: text.to_vec(),
        }
    }
}

impl GreenNodeTrait for GreenTrivia {
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
    fn is_trivia(&self) -> bool {
        true
    }

    #[inline]
    fn get_slot_count(&self) -> u32 {
        // Trivia nodes should never have slots, so we can return
        0
    }

    #[inline]
    fn set_slot_count(&mut self, _value: u8) {
        // Trivia nodes should never have slots, so we can ignore this.
    }

    #[inline]
    fn get_slot(&self, _index: usize) -> Option<GreenNode> {
        // Trivia nodes should never have slots, so we can return None for any index.
        None
    }

    #[inline]
    fn text(&self) -> Vec<u8> {
        self.text.clone()
    }

    #[inline]
    fn full_text(&self) -> Vec<u8> {
        self.text.clone()
    }

    #[inline]
    fn width(&self) -> u32 {
        debug_assert!(
            self.text.len() as u32 == self.green_node.full_width(),
            "Trivia text length does not match full width"
        );

        self.text.len() as u32
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
    fn write_trivia_to(&self) -> &[u8] {
        &self.text
    }
}
