//! Green token representation with inline token text, typed value, and optional trivia.
//!
//! This variant combines `GreenTokenWithValue` payload semantics with optional
//! leading/trailing trivia links and cached full width.

use std::{
    borrow::Borrow,
    fmt,
    hash::{Hash, Hasher},
    mem::{self, ManuallyDrop},
    ops, ptr,
};

use crate::{
    GreenNode,
    arc::{Arc, HeaderSlice, ThinArc},
    green::flags::GreenFlags,
};
use countme::Count;

use crate::SyntaxKind;

pub(crate) type GreenTokenWithIntValueAndTrivia = GreenTokenWithValueAndTrivia<u32>;
pub(crate) type GreenTokenWithFloatValueAndTrivia = GreenTokenWithValueAndTrivia<f32>;
pub(crate) type GreenTokenWithStringValueAndTrivia = GreenTokenWithValueAndTrivia<String>;
pub(crate) type GreenTokenWithIntValueAndTriviaData = GreenTokenWithValueAndTriviaData<u32>;
pub(crate) type GreenTokenWithFloatValueAndTriviaData = GreenTokenWithValueAndTriviaData<f32>;
pub(crate) type GreenTokenWithStringValueAndTriviaData = GreenTokenWithValueAndTriviaData<String>;

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenTokenWithValueAndTriviaHead<T> {
    leading_trivia: Option<GreenNode>,           // 8 bytes on 64-bit targets, 4 bytes on 32-bit targets
    trailing_trivia: Option<GreenNode>,          // 8 bytes on 64-bit targets, 4 bytes on 32-bit targets
    full_width: u16,                             // 2 bytes
    kind: SyntaxKind,                            // 2 bytes (`repr(u16)`)
    flags: GreenFlags,                           // 1 byte
    value: T,                                    // size depends on T
    _c: Count<GreenTokenWithValueAndTrivia<()>>, // 0 bytes
}

#[repr(transparent)]
pub(crate) struct GreenTokenWithValueAndTriviaData<T> {
    data: ReprThin<T>,
}

impl<T> GreenTokenWithValueAndTriviaData<T> {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        self.data.slice()
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        self.write_to(true, true)
    }

    #[inline]
    pub fn value(&self) -> &T {
        &self.data.header.value
    }

    #[inline]
    pub fn width(&self) -> u8 {
        self.data.slice().len() as u8
    }

    #[inline]
    pub fn full_width(&self) -> u16 {
        self.data.header.full_width
    }

    #[inline]
    pub fn leading_trivia(&self) -> Option<GreenNode> {
        self.data.header.leading_trivia.clone()
    }

    #[inline]
    pub fn trailing_trivia(&self) -> Option<GreenNode> {
        self.data.header.trailing_trivia.clone()
    }

    #[inline]
    pub(crate) fn flags(&self) -> GreenFlags {
        self.data.header.flags
    }

    #[inline]
    pub(crate) fn write_to(&self, leading: bool, trailing: bool) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.full_width() as usize);

        if leading {
            if let Some(leading_trivia) = &self.data.header.leading_trivia {
                bytes.extend_from_slice(&leading_trivia.full_text());
            }
        }

        bytes.extend_from_slice(self.text());

        if trailing {
            if let Some(trailing_trivia) = &self.data.header.trailing_trivia {
                bytes.extend_from_slice(&trailing_trivia.full_text());
            }
        }

        bytes
    }
}

impl<T> PartialEq for GreenTokenWithValueAndTriviaData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl<T> fmt::Display for GreenTokenWithValueAndTriviaData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl<T> fmt::Debug for GreenTokenWithValueAndTriviaData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text_str = String::from_utf8_lossy(self.text());

        f.debug_struct("GreenTokenWithValueAndTrivia")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .field("full_width", &self.full_width())
            .finish()
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub(crate) struct GreenTokenWithValueAndTrivia<T> {
    ptr: ThinArc<GreenTokenWithValueAndTriviaHead<T>, u8>,
}

impl<T> PartialEq for GreenTokenWithValueAndTrivia<T> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind() && self.text() == other.text()
    }
}

impl<T> Eq for GreenTokenWithValueAndTrivia<T> {}

impl<T> Hash for GreenTokenWithValueAndTrivia<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind().hash(state);
        self.text().hash(state);
    }
}

impl<T> GreenTokenWithValueAndTrivia<T> {
    #[inline]
    pub fn new(
        kind: SyntaxKind,
        text: &[u8],
        value: T,
        leading_trivia: Option<GreenNode>,
        trailing_trivia: Option<GreenNode>,
    ) -> GreenTokenWithValueAndTrivia<T> {
        let flags = GreenFlags::IS_NOT_MISSING;
        let first_leading_width = leading_trivia.as_ref().map_or(0, |t| t.full_width()) as u16;
        let last_trailing_width = trailing_trivia.as_ref().map_or(0, |t| t.full_width()) as u16;
        let full_width = text.len() as u16 + first_leading_width + last_trailing_width;

        let head = GreenTokenWithValueAndTriviaHead::<T> {
            kind,
            flags,
            full_width,
            leading_trivia,
            trailing_trivia,
            value,
            _c: Count::new(),
        };
        let ptr = ThinArc::from_header_and_iter(head, text.iter().copied());
        GreenTokenWithValueAndTrivia { ptr }
    }
}

impl_green_boilerplate!(generic GreenTokenWithValueAndTriviaHead, GreenTokenWithValueAndTriviaData, GreenTokenWithValueAndTrivia, u8);

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_green_token_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTriviaHead<u32>>(), 32);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTriviaHead<u32>>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueAndTriviaData>(), 40);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueAndTriviaData>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTriviaHead<f32>>(), 32);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTriviaHead<f32>>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueAndTriviaData>(), 40);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueAndTriviaData>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTriviaHead<String>>(), 48);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTriviaHead<String>>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueAndTriviaData>(), 56);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueAndTriviaData>(), 8);

            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueAndTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueAndTrivia>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueAndTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueAndTrivia>(), 8);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueAndTrivia>(), 8);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueAndTrivia>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTriviaHead<u32>>(), 20);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTriviaHead<u32>>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueAndTriviaData>(), 24);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueAndTriviaData>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTriviaHead<f32>>(), 20);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTriviaHead<f32>>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueAndTriviaData>(), 24);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueAndTriviaData>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithValueAndTriviaHead<String>>(), 28);
            assert_eq!(std::mem::align_of::<GreenTokenWithValueAndTriviaHead<String>>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueAndTriviaData>(), 32);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueAndTriviaData>(), 4);

            assert_eq!(std::mem::size_of::<GreenTokenWithIntValueAndTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithIntValueAndTrivia>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithFloatValueAndTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithFloatValueAndTrivia>(), 4);
            assert_eq!(std::mem::size_of::<GreenTokenWithStringValueAndTrivia>(), 4);
            assert_eq!(std::mem::align_of::<GreenTokenWithStringValueAndTrivia>(), 4);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GreenTrivia;
    use pretty_assertions::assert_eq;

    fn leading_trivia() -> Option<GreenNode> {
        Some(GreenNode::new(
            SyntaxKind::List,
            vec![GreenTrivia::new(SyntaxKind::WhitespaceTrivia, b" ").into()],
        ))
    }

    fn trailing_trivia() -> Option<GreenNode> {
        Some(GreenNode::new(
            SyntaxKind::List,
            vec![GreenTrivia::new(SyntaxKind::EndOfLineTrivia, b"\n").into()],
        ))
    }

    #[test]
    fn test_new_when_numeric_with_trivia_expect_kind_text_value_and_full_text() {
        let token: GreenTokenWithIntValueAndTrivia =
            GreenTokenWithValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 42, leading_trivia(), trailing_trivia());
        assert_eq!(token.kind(), SyntaxKind::NumericLiteralToken);
        assert_eq!(token.text(), b"42");
        assert_eq!(token.value(), &42);
        assert_eq!(token.width(), 2);
        assert_eq!(token.full_width(), 4);
        assert_eq!(token.full_text(), b" 42\n");
    }

    #[test]
    fn test_write_to_when_trivia_flags_vary_expect_expected_output() {
        let token: GreenTokenWithStringValueAndTrivia =
            GreenTokenWithValueAndTrivia::new(SyntaxKind::NameLiteralToken, b"Type", "Type".to_string(), leading_trivia(), trailing_trivia());

        assert_eq!(token.write_to(false, false), b"Type");
        assert_eq!(token.write_to(true, false), b" Type");
        assert_eq!(token.write_to(false, true), b"Type\n");
        assert_eq!(token.write_to(true, true), b" Type\n");
    }

    #[test]
    fn test_eq_when_same_kind_and_text_expect_equal_ignoring_value() {
        let token1: GreenTokenWithIntValueAndTrivia = GreenTokenWithValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 1, None, None);
        let token2: GreenTokenWithIntValueAndTrivia = GreenTokenWithValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"42", 2, None, None);
        assert_eq!(token1, token2);
    }

    #[test]
    fn test_into_raw_and_from_raw_when_roundtrip_expect_equal() {
        let token: GreenTokenWithFloatValueAndTrivia = GreenTokenWithValueAndTrivia::new(SyntaxKind::NumericLiteralToken, b"3.5", 3.5, None, None);
        let ptr = GreenTokenWithValueAndTrivia::into_raw(token.clone());
        let reconstructed = unsafe { GreenTokenWithValueAndTrivia::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }

    #[test]
    fn test_borrow_when_name_with_trivia_expect_access_data() {
        let token: GreenTokenWithStringValueAndTrivia = GreenTokenWithValueAndTrivia::new(
            SyntaxKind::NameLiteralToken,
            b"Catalog",
            "Catalog".to_string(),
            leading_trivia(),
            trailing_trivia(),
        );

        let borrowed: &GreenTokenWithValueAndTriviaData<String> = token.borrow();
        assert_eq!(borrowed.kind(), SyntaxKind::NameLiteralToken);
        assert_eq!(borrowed.text(), b"Catalog");
        assert_eq!(borrowed.value(), "Catalog");
        assert!(borrowed.leading_trivia().is_some());
        assert!(borrowed.trailing_trivia().is_some());
    }
}
