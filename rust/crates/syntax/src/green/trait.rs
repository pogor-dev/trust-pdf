use crate::{GreenDiagnostics, GreenElement, GreenNode, SyntaxKind, green::Slots};

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
