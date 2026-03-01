//! Green token representation for missing tokens with optional trivia.
//!
//! This variant mirrors token-with-trivia layout but intentionally does **not**
//! set `GreenFlags::IS_NOT_MISSING`. It models parser-inserted synthetic tokens
//! used for error recovery.

use std::{
    borrow::Borrow,
    fmt,
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

#[derive(PartialEq, Eq, Hash)]
#[repr(C)]
struct GreenMissingTokenHead {
    leading_trivia: Option<GreenNode>,
    trailing_trivia: Option<GreenNode>,
    full_width: u16,
    kind: SyntaxKind,
    flags: GreenFlags,
    _c: Count<GreenMissingToken>,
}

type Repr = HeaderSlice<GreenMissingTokenHead, [u8]>;
type ReprThin = HeaderSlice<GreenMissingTokenHead, [u8; 0]>;

#[repr(transparent)]
pub(crate) struct GreenMissingTokenData {
    data: ReprThin,
}

impl GreenMissingTokenData {
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    #[inline]
    pub fn text(&self) -> &[u8] {
        self.kind().get_text()
    }

    #[inline]
    pub fn full_text(&self) -> Vec<u8> {
        self.write_to(true, true)
    }

    #[inline]
    pub fn width(&self) -> u8 {
        self.kind().get_text().len() as u8
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

impl PartialEq for GreenMissingTokenData {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

impl ToOwned for GreenMissingTokenData {
    type Owned = GreenMissingToken;

    #[inline]
    fn to_owned(&self) -> GreenMissingToken {
        let green = unsafe { GreenMissingToken::from_raw(ptr::NonNull::from(self)) };
        let green = ManuallyDrop::new(green);
        GreenMissingToken::clone(&green)
    }
}

impl fmt::Display for GreenMissingTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.text() {
            write!(f, "{}", byte as char)?;
        }
        Ok(())
    }
}

impl fmt::Debug for GreenMissingTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text_str = String::from_utf8_lossy(self.text());

        f.debug_struct("GreenMissingToken")
            .field("kind", &self.kind())
            .field("text", &text_str)
            .field("width", &self.width())
            .finish()
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
#[repr(transparent)]
pub(crate) struct GreenMissingToken {
    ptr: ThinArc<GreenMissingTokenHead, u8>,
}

impl Borrow<GreenMissingTokenData> for GreenMissingToken {
    #[inline]
    fn borrow(&self) -> &GreenMissingTokenData {
        self
    }
}

impl fmt::Display for GreenMissingToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenMissingTokenData = self;
        fmt::Display::fmt(data, f)
    }
}

impl fmt::Debug for GreenMissingToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data: &GreenMissingTokenData = self;
        fmt::Debug::fmt(data, f)
    }
}

impl GreenMissingToken {
    #[inline]
    pub fn new(kind: SyntaxKind, leading_trivia: Option<GreenNode>, trailing_trivia: Option<GreenNode>) -> Self {
        let flags = GreenFlags::NONE;

        let first_leading_width = leading_trivia.as_ref().map_or(0, |t| t.full_width()) as u16;
        let last_trailing_width = trailing_trivia.as_ref().map_or(0, |t| t.full_width()) as u16;
        let full_width = kind.get_text().len() as u16 + first_leading_width + last_trailing_width;

        let head = GreenMissingTokenHead {
            kind,
            flags,
            full_width,
            leading_trivia,
            trailing_trivia,
            _c: Count::new(),
        };

        let ptr = ThinArc::from_header_and_iter(head, std::iter::empty());
        GreenMissingToken { ptr }
    }

    #[inline]
    pub(crate) fn into_raw(this: GreenMissingToken) -> ptr::NonNull<GreenMissingTokenData> {
        let green = ManuallyDrop::new(this);
        let green: &GreenMissingTokenData = &green;
        ptr::NonNull::from(green)
    }

    #[inline]
    pub(crate) unsafe fn from_raw(ptr: ptr::NonNull<GreenMissingTokenData>) -> GreenMissingToken {
        let arc = unsafe {
            let arc = Arc::from_raw(&ptr.as_ref().data as *const ReprThin);
            mem::transmute::<Arc<ReprThin>, ThinArc<GreenMissingTokenHead, u8>>(arc)
        };
        GreenMissingToken { ptr: arc }
    }
}

impl ops::Deref for GreenMissingToken {
    type Target = GreenMissingTokenData;

    #[inline]
    fn deref(&self) -> &GreenMissingTokenData {
        unsafe {
            let repr: &Repr = &*self.ptr;
            let repr: &ReprThin = &*(repr as *const Repr as *const ReprThin);
            mem::transmute::<&ReprThin, &GreenMissingTokenData>(repr)
        }
    }
}

#[cfg(test)]
mod memory_layout_tests {
    use super::*;

    #[test]
    fn test_green_missing_token_head_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenMissingTokenHead>(), 24);
            assert_eq!(std::mem::align_of::<GreenMissingTokenHead>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenMissingTokenHead>(), 16);
            assert_eq!(std::mem::align_of::<GreenMissingTokenHead>(), 4);
        }
    }

    #[test]
    fn test_green_missing_token_data_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenMissingTokenData>(), 32);
            assert_eq!(std::mem::align_of::<GreenMissingTokenData>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenMissingTokenData>(), 20);
            assert_eq!(std::mem::align_of::<GreenMissingTokenData>(), 4);
        }
    }

    #[test]
    fn test_green_missing_token_memory_layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<GreenMissingToken>(), 8);
            assert_eq!(std::mem::align_of::<GreenMissingToken>(), 8);
        }

        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<GreenMissingToken>(), 4);
            assert_eq!(std::mem::align_of::<GreenMissingToken>(), 4);
        }
    }
}

#[cfg(test)]
mod green_missing_token_tests {
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
    fn test_new_when_created_expect_missing_flag_state() {
        let token = GreenMissingToken::new(SyntaxKind::TrueKeyword, None, None);
        assert!(!token.flags().contains(GreenFlags::IS_NOT_MISSING));
        assert_eq!(token.flags(), GreenFlags::NONE);
    }

    #[test]
    fn test_full_text_when_trivia_present_expect_includes_trivia_and_text() {
        let token = GreenMissingToken::new(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia());
        assert_eq!(token.full_text(), b" true\n");
        assert_eq!(token.width(), 4);
        assert_eq!(token.full_width(), 6);
    }

    #[test]
    fn test_write_to_when_flags_vary_expect_expected_bytes() {
        let token = GreenMissingToken::new(SyntaxKind::TrueKeyword, leading_trivia(), trailing_trivia());
        assert_eq!(token.write_to(false, false), b"true");
        assert_eq!(token.write_to(true, false), b" true");
        assert_eq!(token.write_to(false, true), b"true\n");
        assert_eq!(token.write_to(true, true), b" true\n");
    }

    #[test]
    fn test_into_raw_and_from_raw_when_roundtrip_expect_equal() {
        let token = GreenMissingToken::new(SyntaxKind::TrueKeyword, None, None);
        let ptr = GreenMissingToken::into_raw(token.clone());
        let reconstructed = unsafe { GreenMissingToken::from_raw(ptr) };
        assert_eq!(token, reconstructed);
    }
}
