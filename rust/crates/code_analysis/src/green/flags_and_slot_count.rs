use crate::green::flags::GreenFlags;

/// Compact storage for node flags and a small slot count in one 16-bit value.
///
/// Mirrors Roslyn's `NodeFlagsAndSlotCount` design for memory-efficient metadata packing.
///
/// ## Bit Layout
/// ```text
/// Bit positions (MSB to LSB):
/// 15  14  13  12 │ 11  10  09  08  07  06  05  04  03  02  01  00
/// ───────────────┼────────────────────────────────────────────────
///  C   C   C   C │  F   F   F   F   F   F   F   F   F   F   F   F
/// ───────────────┼────────────────────────────────────────────────
/// Slot Count (4) │          Node Flags (12 bits)
/// ```
///
/// - **Slot Count (4 bits, positions 12-15)**: Stores child count 0-14 directly.
///   Value `0b1111` (15) means "too large" → fetch real count elsewhere.
/// - **Node Flags (12 bits, positions 0-11)**: Metadata like `IS_NOT_MISSING`,
///   `CONTAINS_DIAGNOSTICS`, etc.
///
/// ## Example
/// ```text
/// Setting slot_count=5, flags=0b0000_0000_0011:
///
/// Initial:  0000_0000_0000_0000
/// Set 5:    0101_0000_0000_0000  (5 << 12)
/// Set 0x03: 0101_0000_0000_0011  (flags in lower 12 bits)
///           ^^^^             ^^
///           slot=5           flags=0b0011
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct NodeFlagsAndSlotCount {
    data: u16,
}

impl NodeFlagsAndSlotCount {
    /// Mask isolating the 4 slot-count bits (positions 12-15).
    /// ```text
    /// 1111_0000_0000_0000
    /// ^^^^                ← slot count region
    /// ```
    const SLOT_COUNT_MASK: u16 = 0b1111_0000_0000_0000;

    /// Mask isolating the 12 node-flag bits (positions 0-11).
    /// ```text
    /// 0000_1111_1111_1111
    ///      ^^^^^^^^^^^^^^   ← flag region
    /// ```
    const NODE_FLAGS_MASK: u16 = 0b0000_1111_1111_1111;

    /// Shift amount to move slot count to/from upper 4 bits.
    const SLOT_COUNT_SHIFT: u16 = 12;

    /// Sentinel: when all 4 slot-count bits are set (15), the real count
    /// must be retrieved from alternate storage (e.g., separate field in list nodes).
    pub(crate) const SLOT_COUNT_TOO_LARGE: u8 = 0b0000_1111;

    /// Extracts the slot count from the packed value.
    ///
    /// ```text
    /// data: CCCC_FFFF_FFFF_FFFF
    ///       ^^^^            shift right 12 bits
    ///       └──► returns C as u8
    /// ```
    #[inline]
    pub(crate) fn small_slot_count(self) -> u8 {
        let shifted = self.data >> Self::SLOT_COUNT_SHIFT;
        debug_assert!(shifted <= Self::SLOT_COUNT_TOO_LARGE as u16);
        shifted as u8
    }

    /// Updates the slot count while preserving existing flags.
    ///
    /// ```text
    /// Step 1: Clear slot-count bits, keep flags
    ///   data:            CCCC_FFFF_FFFF_FFFF
    ///   NODE_FLAGS_MASK: 0000_1111_1111_1111
    ///   &               ────────────────────
    ///   result:          CCCC_FFFF_FFFF_FFFF  ← flags preserved
    ///
    /// Step 2: Prepare new slot count
    ///   value: 0000_0000_0000_0101 (5)
    ///   << 12: 0101_0000_0000_0000  ← shifted to slot-count position
    ///
    /// Step 3: Combine via bitwise OR
    ///   cleared flags:   CCCC_FFFF_FFFF_FFFF
    ///   shifted value:   0101_0000_0000_0000
    ///   |                ────────────────────
    ///   final:           0101_FFFF_FFFF_FFFF  ← slot count updated!
    /// ```
    #[inline]
    pub(crate) fn set_small_slot_count(&mut self, value: u8) {
        let mut value = value;
        if value > Self::SLOT_COUNT_TOO_LARGE {
            value = Self::SLOT_COUNT_TOO_LARGE;
        }

        self.data = (self.data & Self::NODE_FLAGS_MASK) | ((value as u16) << Self::SLOT_COUNT_SHIFT);
    }

    /// Extracts the node flags from the packed value.
    ///
    /// ```text
    /// data:            CCCC_FFFF_FFFF_FFFF
    /// NODE_FLAGS_MASK: 0000_1111_1111_1111
    /// &               ────────────────────
    /// result:          0000_FFFF_FFFF_FFFF  ← only flag bits
    ///                       └──► returns as GreenFlags
    /// ```
    #[inline]
    pub(crate) fn node_flags(self) -> GreenFlags {
        GreenFlags::from_bits((self.data & Self::NODE_FLAGS_MASK) as u8)
    }

    /// Updates the node flags while preserving the slot count.
    ///
    /// ```text
    /// Step 1: Clear flag bits, keep slot count
    ///   data:            CCCC_FFFF_FFFF_FFFF
    ///   SLOT_COUNT_MASK: 1111_0000_0000_0000
    ///   &               ────────────────────
    ///   result:          CCCC_0000_0000_0000  ← slot count preserved
    ///
    /// Step 2: Prepare new flags (already in lower 12 bits)
    ///   value.bits():    0000_0000_0010_1101  (flags as u8)
    ///   as u16:          0000_0000_0010_1101
    ///
    /// Step 3: Combine via bitwise OR
    ///   cleared slot:    CCCC_0000_0000_0000
    ///   new flags:       0000_0000_0010_1101
    ///   |                ────────────────────
    ///   final:           CCCC_0000_0010_1101  ← flags updated!
    /// ```
    #[inline]
    pub(crate) fn set_node_flags(&mut self, value: GreenFlags) {
        debug_assert!((value.bits() as u16) <= Self::NODE_FLAGS_MASK);
        self.data = (self.data & Self::SLOT_COUNT_MASK) | (value.bits() as u16);
    }
}
