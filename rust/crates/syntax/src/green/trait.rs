use crate::{GreenDiagnostics, GreenElement, GreenNode, SyntaxKind, green::Slots};

/// Trait providing typed casting from a generic `GreenNode` into a specific
/// wrapper type. This mirrors Roslyn's `CanCast`/`Cast` pattern.
pub trait GreenCst: Sized {
    /// Returns true if the provided node matches the expected shape/kind.
    fn can_cast(node: &GreenNode) -> bool;

    /// Attempts to cast the provided node into the typed wrapper.
    fn cast(node: GreenNode) -> Option<Self>;
}

pub trait GreenTrait {
    fn kind(&self) -> SyntaxKind;
    fn diagnostics(&self) -> Option<&GreenDiagnostics>;
    fn text(&self) -> Vec<u8>;
    fn full_text(&self) -> Vec<u8>;
    fn width(&self) -> u32;
    fn full_width(&self) -> u32;
    fn leading_trivia(&self) -> Option<GreenNode>;
    fn trailing_trivia(&self) -> Option<GreenNode>;
    fn slot_count(&self) -> usize;
    fn slots(&self) -> Slots<'_>;
    fn slot(&self, index: usize) -> Option<GreenElement>;
    fn slot_offset(&self, index: usize) -> Option<u32>;
}

pub trait GreenNodeSyntax {
    /// Returns a shared reference to the underlying green node.
    fn green(&self) -> &GreenNode;
}

impl<T: GreenNodeSyntax> GreenTrait for T {
    #[inline]
    fn kind(&self) -> SyntaxKind {
        self.green().kind()
    }

    #[inline]
    fn diagnostics(&self) -> Option<&GreenDiagnostics> {
        self.green().diagnostics()
    }

    #[inline]
    fn text(&self) -> Vec<u8> {
        self.green().text()
    }

    #[inline]
    fn full_text(&self) -> Vec<u8> {
        self.green().full_text()
    }

    #[inline]
    fn width(&self) -> u32 {
        self.green().width()
    }

    #[inline]
    fn full_width(&self) -> u32 {
        self.green().full_width()
    }

    #[inline]
    fn leading_trivia(&self) -> Option<GreenNode> {
        self.green().leading_trivia()
    }

    #[inline]
    fn trailing_trivia(&self) -> Option<GreenNode> {
        self.green().trailing_trivia()
    }

    #[inline]
    fn slot_count(&self) -> usize {
        self.green().slot_count()
    }

    #[inline]
    fn slots(&self) -> Slots<'_> {
        self.green().slots()
    }

    #[inline]
    fn slot(&self, index: usize) -> Option<GreenElement> {
        self.green().slot(index)
    }

    #[inline]
    fn slot_offset(&self, index: usize) -> Option<u32> {
        self.green().slot_offset(index)
    }
}
