/// Special flags that a syntax tree node can have.
///
/// Typed as `u8` providing 8 bits for flag storage. These flags are attached to
/// green nodes to track metadata without additional memory allocation.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GreenFlags(u8);

impl GreenFlags {
    pub const NONE: Self = Self(0);

    /// Marks a node as not-missing (i.e., present in source code).
    ///
    /// "Missing" nodes are synthetic nodes inserted by the parser during error recovery
    /// when expected syntax is absent (e.g., a missing value in a PDF dictionary like `<</Type >>`).
    ///
    /// **Design rationale**: Uses a non-zero bit for NOT_MISSING (rather than IS_MISSING)
    /// so that the flag automatically propagates upward via bitwise OR during tree construction.
    /// Once any child is not-missing, all parent nodes are not-missing without additional logic.
    ///
    /// # Example
    /// ```text
    /// Child A: 0b0001 (IS_NOT_MISSING set - real syntax)
    /// Child B: 0b0000 (missing node)
    /// Parent:  0b0001 | 0b0000 = 0b0001 (automatically not-missing)
    /// ```
    ///
    /// This pattern originates from Roslyn's syntax tree design.
    pub const IS_NOT_MISSING: Self = Self(1 << 0);
}

impl GreenFlags {
    #[inline(always)]
    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    #[inline(always)]
    pub const fn bits(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline(always)]
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    #[inline(always)]
    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    #[inline(always)]
    pub fn toggle(&mut self, other: Self) {
        self.0 ^= other.0;
    }
}

impl core::ops::BitOr for GreenFlags {
    type Output = Self;
    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl core::ops::BitOrAssign for GreenFlags {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl core::ops::BitAnd for GreenFlags {
    type Output = Self;
    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl core::ops::BitAndAssign for GreenFlags {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl core::ops::BitXor for GreenFlags {
    type Output = Self;
    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl core::ops::BitXorAssign for GreenFlags {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl core::ops::Not for GreenFlags {
    type Output = Self;
    #[inline(always)]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_none_constant() {
        let flags = GreenFlags::NONE;
        assert_eq!(flags.bits(), 0);
    }

    #[test]
    fn test_is_not_missing_constant() {
        let flags = GreenFlags::IS_NOT_MISSING;
        assert_eq!(flags.bits(), 1 << 0);
    }

    #[test]
    fn test_from_bits() {
        let flags = GreenFlags::from_bits(0b0101);
        assert_eq!(flags.bits(), 0b0101);
    }

    #[test]
    fn test_bits() {
        let flags = GreenFlags(42);
        assert_eq!(flags.bits(), 42);
    }

    #[test]
    fn test_contains_when_flag_present_expect_true() {
        let flags = GreenFlags::IS_NOT_MISSING;
        assert!(flags.contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_contains_when_flag_absent_expect_false() {
        let flags = GreenFlags::NONE;
        assert!(!flags.contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_contains_when_none_expect_true() {
        let flags = GreenFlags::NONE;
        assert!(flags.contains(GreenFlags::NONE));
    }

    #[test]
    fn test_insert() {
        let mut flags = GreenFlags::NONE;
        flags.insert(GreenFlags::IS_NOT_MISSING);
        assert!(flags.contains(GreenFlags::IS_NOT_MISSING));
        assert_eq!(flags.bits(), 0b0001);
    }

    #[test]
    fn test_insert_multiple() {
        let mut flags = GreenFlags::NONE;
        flags.insert(GreenFlags::IS_NOT_MISSING);
        assert!(flags.contains(GreenFlags::IS_NOT_MISSING));
        assert_eq!(flags.bits(), 0b0001);
    }

    #[test]
    fn test_remove() {
        let mut flags = GreenFlags::IS_NOT_MISSING;
        flags.remove(GreenFlags::IS_NOT_MISSING);
        assert!(!flags.contains(GreenFlags::IS_NOT_MISSING));
        assert_eq!(flags.bits(), 0);
    }

    #[test]
    fn test_remove_when_flag_absent_expect_no_change() {
        let mut flags = GreenFlags::NONE;
        flags.remove(GreenFlags::IS_NOT_MISSING);
        assert_eq!(flags.bits(), GreenFlags::NONE.bits());
    }

    #[test]
    fn test_toggle() {
        let mut flags = GreenFlags::NONE;
        flags.toggle(GreenFlags::IS_NOT_MISSING);
        assert!(flags.contains(GreenFlags::IS_NOT_MISSING));
        flags.toggle(GreenFlags::IS_NOT_MISSING);
        assert!(!flags.contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_bitor() {
        let flags1 = GreenFlags::NONE;
        let flags2 = GreenFlags::IS_NOT_MISSING;
        let result = flags1 | flags2;
        assert!(result.contains(GreenFlags::IS_NOT_MISSING));
        assert_eq!(result.bits(), 0b0001);
    }

    #[test]
    fn test_bitor_assign() {
        let mut flags = GreenFlags::NONE;
        flags |= GreenFlags::IS_NOT_MISSING;
        assert!(flags.contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_bitand() {
        let flags1 = GreenFlags::from_bits(0b0111);
        let flags2 = GreenFlags::from_bits(0b0011);
        let result = flags1 & flags2;
        assert_eq!(result.bits(), 0b0011);
    }

    #[test]
    fn test_bitand_assign() {
        let mut flags = GreenFlags::from_bits(0b0111);
        flags &= GreenFlags::from_bits(0b0011);
        assert_eq!(flags.bits(), 0b0011);
    }

    #[test]
    fn test_bitxor() {
        let flags1 = GreenFlags::from_bits(0b0101);
        let flags2 = GreenFlags::from_bits(0b0011);
        let result = flags1 ^ flags2;
        assert_eq!(result.bits(), 0b0110);
    }

    #[test]
    fn test_bitxor_assign() {
        let mut flags = GreenFlags::from_bits(0b0101);
        flags ^= GreenFlags::from_bits(0b0011);
        assert_eq!(flags.bits(), 0b0110);
    }

    #[test]
    fn test_not() {
        let flags = GreenFlags::from_bits(0b0101);
        let result = !flags;
        assert_eq!(result.bits(), !0b0101);
    }

    #[test]
    fn test_default() {
        let flags = GreenFlags::default();
        assert_eq!(flags.bits(), 0);
    }

    #[test]
    fn test_clone() {
        let flags1 = GreenFlags::IS_NOT_MISSING;
        let flags2 = flags1.clone();
        assert_eq!(flags1.bits(), flags2.bits());
    }

    #[test]
    fn test_copy() {
        let flags1 = GreenFlags::IS_NOT_MISSING;
        let flags2 = flags1; // Copy semantics
        assert_eq!(flags1.bits(), flags2.bits());
    }

    #[test]
    fn test_partial_eq() {
        let flags1 = GreenFlags::IS_NOT_MISSING;
        let flags2 = GreenFlags::IS_NOT_MISSING;
        let flags3 = GreenFlags::NONE;
        assert_eq!(flags1, flags2);
        assert_ne!(flags1, flags3);
    }

    #[test]
    fn test_upward_propagation_design() {
        // Simulate parent combining child flags via bitwise OR
        let child_a = GreenFlags::IS_NOT_MISSING; // Real syntax: 0b0001
        let child_b = GreenFlags::NONE; // Missing node: 0b0000
        let parent = child_a | child_b;

        // Parent automatically has IS_NOT_MISSING without additional logic
        assert!(parent.contains(GreenFlags::IS_NOT_MISSING));
        assert_eq!(parent.bits(), 0b0001);
    }

    #[test]
    fn test_upward_propagation_with_multiple_children() {
        // When multiple children combine, any non-missing child propagates upward
        let child1 = GreenFlags::NONE; // Missing
        let child2 = GreenFlags::IS_NOT_MISSING; // Not missing
        let child3 = GreenFlags::NONE; // Missing

        let combined = child1 | child2 | child3;
        assert!(combined.contains(GreenFlags::IS_NOT_MISSING));
    }

    #[test]
    fn test_all_missing_children_remain_missing() {
        // When all children are missing (NONE), parent remains missing
        let child1 = GreenFlags::NONE;
        let child2 = GreenFlags::NONE;
        let parent = child1 | child2;

        assert!(!parent.contains(GreenFlags::IS_NOT_MISSING));
        assert_eq!(parent.bits(), 0);
    }

    #[test]
    fn test_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let flags1 = GreenFlags::IS_NOT_MISSING;
        let flags2 = GreenFlags::IS_NOT_MISSING;

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        flags1.hash(&mut hasher1);
        flags2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }
}
