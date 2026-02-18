use crate::{
    SyntaxKind,
    green::{flags::GreenFlags, flags_and_slot_count::NodeFlagsAndSlotCount, green_trait::GreenNodeTrait},
};

/// Pack the kind, node-flags, slot-count, and full-width into 64bits.
///
/// Note: if we need more bits in the future, we can always directly use a packed int64 here,
/// and manage where all these bits go manually.
#[repr(C)]
pub(crate) struct GreenNode {
    kind: SyntaxKind,                                 // 2 bytes
    node_flags_and_slot_count: NodeFlagsAndSlotCount, // 2 bytes
    full_width: u32,                                  // 4 bytes
}

impl GreenNode {
    pub(super) fn new(kind: SyntaxKind, full_width: u32) -> Self {
        let mut node_flags_and_slot_count = NodeFlagsAndSlotCount::default();
        node_flags_and_slot_count.set_small_slot_count(0);
        node_flags_and_slot_count.set_node_flags(GreenFlags::NONE);

        GreenNode {
            kind,
            node_flags_and_slot_count,
            full_width,
        }
    }
}

impl GreenNodeTrait for GreenNode {
    #[inline]
    fn kind(&self) -> SyntaxKind {
        self.kind
    }

    #[inline]
    fn flags(&self) -> GreenFlags {
        self.node_flags_and_slot_count.node_flags()
    }

    #[inline]
    fn set_flags(&mut self, flags: GreenFlags) {
        self.node_flags_and_slot_count.set_node_flags(flags);
    }

    #[inline]
    fn full_width(&self) -> u32 {
        self.full_width
    }

    #[inline]
    fn get_slot_count(&self) -> u32 {
        let count = self.node_flags_and_slot_count.small_slot_count();
        match count {
            NodeFlagsAndSlotCount::SLOT_COUNT_TOO_LARGE => self.get_large_slot_count(),
            _ => count as u32,
        }
    }

    #[inline]
    fn set_slot_count(&mut self, value: u8) {
        debug_assert!(value <= u8::MAX, "slot count {} exceeds maximum representable value", value);
        self.node_flags_and_slot_count.set_small_slot_count(value);
    }

    #[inline]
    fn get_slot(&self, _index: usize) -> Option<GreenNode> {
        None
    }
}
