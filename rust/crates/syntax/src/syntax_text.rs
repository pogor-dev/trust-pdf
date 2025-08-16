//! Text access for PDF syntax trees optimized for byte-level operations.
//!
//! PDF files contain mixed content: text in various encodings, binary data, and structured elements.
//! Unlike typical programming languages that work with UTF-8 strings, PDF parsing requires
//! byte-oriented operations to handle this diverse content correctly.
//!
//! [`SyntaxText`] provides efficient access to text content from syntax tree nodes without
//! copying data, supporting operations like searching, slicing, and comparison at the byte level.
//!
//! ## Why Byte-Oriented?
//!
//! PDF files may contain:
//! - Text in various encodings (ASCII, Latin-1, UTF-16, etc.)
//! - Binary stream data
//! - Embedded fonts and images
//! - Control characters with semantic meaning
//!
//! Working with bytes ensures we can handle all content types without encoding assumptions.
//!
//! ## Example
//!
//! ```ignore
//! let text = node.text();
//!
//! // Search for PDF operators
//! if text.contains_byte(b'/') {
//!     let pos = text.find_byte(b'/').unwrap();
//!     println!("Found name token at position {}", pos);
//! }
//!
//! // Extract specific ranges
//! let header = text.slice(..4); // First 4 bytes
//! ```

use std::{fmt, ops::Range};

use crate::cursor::{node::SyntaxNode, token::SyntaxToken};

/// Zero-copy text view over syntax tree nodes with byte-level operations.
///
/// Provides access to text content from PDF syntax trees without materializing
/// the entire text in memory. Designed for PDF parsing where content mixing
/// text, binary data, and various encodings requires byte-oriented operations.
///
/// The text spans across multiple syntax tokens but appears as a single
/// contiguous byte sequence, enabling efficient searching and slicing.
#[derive(Clone)]
pub struct SyntaxText {
    /// The root syntax node containing the text tokens
    node: SyntaxNode,
    /// The byte range within the node's text span
    range: Range<u32>,
}

impl SyntaxText {
    /// Creates a text view covering the entire syntax node.
    ///
    /// Used internally when converting syntax nodes to text views.
    #[allow(dead_code)]
    pub(crate) fn new(node: SyntaxNode) -> SyntaxText {
        let range = node.full_span();
        SyntaxText { node, range }
    }

    /// Returns the text length in bytes.
    pub fn len(&self) -> u32 {
        self.range.len() as u32
    }

    /// Returns `true` if the text contains no bytes.
    pub fn is_empty(&self) -> bool {
        self.range.is_empty()
    }

    /// Returns `true` if the text contains the specified byte.
    ///
    /// Efficiently searches through text chunks without loading all content into memory.
    pub fn contains_byte(&self, c: u8) -> bool {
        self.try_for_each_chunk(|chunk| if chunk.contains(&c) { Err(()) } else { Ok(()) })
            .is_err()
    }

    /// Finds the first occurrence of a byte and returns its position.
    ///
    /// Returns `None` if the byte is not found. Position is relative to this text view.
    pub fn find_byte(&self, c: u8) -> Option<u32> {
        let mut acc: u32 = 0;
        let res = self.try_for_each_chunk(|chunk| {
            if let Some(pos) = chunk.iter().position(|&b| b == c) {
                let pos: u32 = pos as u32;
                return Err(acc + pos);
            }
            acc += chunk.len() as u32;
            Ok(())
        });
        found(res)
    }

    /// Returns the byte at the specified position.
    ///
    /// Returns `None` if the position is beyond the text length.
    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        let mut start: u32 = 0;
        let res = self.try_for_each_chunk(|chunk| {
            let end = start + chunk.len() as u32;
            if start <= offset && offset < end {
                let off: usize = (offset - start) as usize;
                return Err(chunk[off]);
            }
            start = end;
            Ok(())
        });
        found(res)
    }

    /// Creates a slice of this text within the specified range.
    ///
    /// Supports various range types: `1..4`, `1..`, `..4`, and `..` for convenience.
    /// The slice shares the underlying data without copying.
    pub fn slice<R: private::SyntaxTextRange>(&self, range: R) -> SyntaxText {
        let start = range.start().unwrap_or_default();
        let end = range.end().unwrap_or(self.len());
        assert!(start <= end);
        let len = end - start;
        let start = self.range.start + start;
        let end = start + len;
        assert!(
            start <= end,
            "invalid slice, range: {:?}, slice: {:?}",
            self.range,
            (range.start(), range.end()),
        );
        let range = start..end;
        assert!(
            self.range.start <= range.start && self.range.end >= range.end,
            "invalid slice, range: {:?}, slice: {:?}",
            self.range,
            range,
        );
        SyntaxText {
            node: self.node.clone(),
            range,
        }
    }

    /// Applies a fallible operation to text chunks, accumulating a result.
    ///
    /// Processes text in chunks corresponding to syntax tokens. Useful for building
    /// results from text content while handling potential errors during processing.
    pub fn try_fold_chunks<T, F, E>(&self, init: T, mut f: F) -> Result<T, E>
    where
        F: FnMut(T, &[u8]) -> Result<T, E>,
    {
        self.tokens_with_ranges()
            .try_fold(init, move |acc, (token, range)| {
                let token_text = token.full_text();
                let range_start = range.start as usize;
                let range_end = range.end as usize;
                f(acc, &token_text[range_start..range_end])
            })
    }

    /// Applies a fallible function to each text chunk.
    ///
    /// Stops processing and returns the first error encountered.
    pub fn try_for_each_chunk<F: FnMut(&[u8]) -> Result<(), E>, E>(
        &self,
        mut f: F,
    ) -> Result<(), E> {
        self.try_fold_chunks((), move |(), chunk| f(chunk))
    }

    /// Applies a function to each text chunk.
    ///
    /// For simple processing where errors are not expected. Use `try_for_each_chunk`
    /// when error handling is needed.
    pub fn for_each_chunk<F: FnMut(&[u8])>(&self, mut f: F) {
        enum Void {}
        match self.try_for_each_chunk(|chunk| {
            f(chunk);
            Ok::<(), Void>(())
        }) {
            Ok(()) => (),
            Err(void) => match void {},
        }
    }

    /// Collects all text content into a single byte vector.
    ///
    /// Returns the complete text as a `Vec<u8>`, materializing all chunks
    /// into a contiguous byte array. Use this when you need owned access
    /// to the raw byte content.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.len() as usize);
        self.for_each_chunk(|chunk| {
            bytes.extend_from_slice(chunk);
        });
        bytes
    }

    /// Returns an iterator over tokens and their byte ranges.
    ///
    /// Used internally by chunk processing methods to access the underlying
    /// syntax tokens and their corresponding text content.
    fn tokens_with_ranges(&self) -> impl Iterator<Item = (SyntaxToken, Range<u32>)> + use<> {
        let text_range = self.range.clone();
        self.node
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
            .filter_map(move |token| {
                let token_range = token.full_span();
                let range = range_intersection(text_range.clone(), token_range.clone())?;
                Some((
                    token,
                    (range.start - token_range.start)..(range.end - token_range.start),
                ))
            })
    }
}

/// Extracts a value from early-termination search results.
///
/// Search methods use `Err(value)` to break out of iteration when the target is found.
fn found<T>(res: Result<(), T>) -> Option<T> {
    match res {
        Ok(()) => None,
        Err(it) => Some(it),
    }
}

/// Computes the intersection of two byte ranges.
///
/// Returns the overlapping portion if ranges intersect, otherwise `None`.
fn range_intersection(a: Range<u32>, b: Range<u32>) -> Option<Range<u32>> {
    let start = std::cmp::max(a.start, b.start);
    let end = std::cmp::min(a.end, b.end);
    if start < end { Some(start..end) } else { None }
}

impl fmt::Debug for SyntaxText {
    /// Formats text content for debugging output.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.to_string(), f)
    }
}

impl fmt::Display for SyntaxText {
    /// Converts bytes to UTF-8 text with fallback to hex for invalid sequences.
    ///
    /// Invalid UTF-8 bytes are displayed as `\xff` escape sequences, ensuring
    /// all content can be safely displayed even for binary PDF data.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.try_for_each_chunk(|chunk| {
            // Convert bytes to string, handling invalid UTF-8 gracefully
            match std::str::from_utf8(chunk) {
                Ok(s) => write!(f, "{}", s),
                Err(_) => {
                    // If invalid UTF-8, display as hex bytes
                    for &byte in chunk {
                        write!(f, "\\x{:02x}", byte)?;
                    }
                    Ok(())
                }
            }
        })
    }
}

impl From<SyntaxText> for String {
    /// Converts to `String` using the `Display` implementation.
    fn from(text: SyntaxText) -> String {
        text.to_string()
    }
}

impl PartialEq<[u8]> for SyntaxText {
    /// Compares text content with a byte slice.
    ///
    /// Efficiently compares chunk-by-chunk without materializing the full text.
    fn eq(&self, mut rhs: &[u8]) -> bool {
        self.try_for_each_chunk(|chunk| {
            if !rhs.starts_with(chunk) {
                return Err(());
            }
            rhs = &rhs[chunk.len()..];
            Ok(())
        })
        .is_ok()
            && rhs.is_empty()
    }
}

impl PartialEq<SyntaxText> for [u8] {
    /// Provides symmetric equality for byte slices and `SyntaxText`.
    fn eq(&self, rhs: &SyntaxText) -> bool {
        rhs == self
    }
}

impl PartialEq<&'_ [u8]> for SyntaxText {
    /// Compares with a byte slice reference.
    fn eq(&self, rhs: &&[u8]) -> bool {
        self == *rhs
    }
}

impl PartialEq<SyntaxText> for &'_ [u8] {
    /// Provides symmetric equality for byte slice references and `SyntaxText`.
    fn eq(&self, rhs: &SyntaxText) -> bool {
        rhs == self
    }
}

impl PartialEq for SyntaxText {
    /// Compares two text instances for content equality.
    ///
    /// Handles cases where texts have different token boundaries but identical content
    /// by comparing overlapping chunks synchronously.
    fn eq(&self, other: &SyntaxText) -> bool {
        if self.range.len() != other.range.len() {
            return false;
        }
        let mut lhs = self.tokens_with_ranges();
        let mut rhs = other.tokens_with_ranges();
        zip_texts(&mut lhs, &mut rhs).is_none()
            && lhs.all(|it| it.1.is_empty())
            && rhs.all(|it| it.1.is_empty())
    }
}

/// Compares text content from two token iterators.
///
/// Advances through both iterators synchronously, comparing overlapping portions
/// even when token boundaries differ. Returns `Some(())` on mismatch, `None` if equal.
fn zip_texts<I: Iterator<Item = (SyntaxToken, Range<u32>)>>(xs: &mut I, ys: &mut I) -> Option<()> {
    let mut x = xs.next()?;
    let mut y = ys.next()?;
    loop {
        while x.1.is_empty() {
            x = xs.next()?;
        }
        while y.1.is_empty() {
            y = ys.next()?;
        }
        let x_text_full = x.0.full_text();
        let y_text_full = y.0.full_text();
        let x_text: &[u8] = &x_text_full[x.1.start as usize..x.1.end as usize];
        let y_text: &[u8] = &y_text_full[y.1.start as usize..y.1.end as usize];
        if !(x_text.starts_with(y_text) || y_text.starts_with(x_text)) {
            return Some(());
        }
        let advance = std::cmp::min(x.1.len(), y.1.len()) as u32;
        x.1 = x.1.start + advance..x.1.end;
        y.1 = y.1.start + advance..y.1.end;
    }
}

impl Eq for SyntaxText {}

mod private {
    use std::ops::{self, Range};

    pub trait SyntaxTextRange {
        fn start(&self) -> Option<u32>;
        fn end(&self) -> Option<u32>;
    }

    impl SyntaxTextRange for Range<u32> {
        fn start(&self) -> Option<u32> {
            Some(self.start)
        }
        fn end(&self) -> Option<u32> {
            Some(self.end)
        }
    }

    impl SyntaxTextRange for ops::RangeFrom<u32> {
        fn start(&self) -> Option<u32> {
            Some(self.start)
        }
        fn end(&self) -> Option<u32> {
            None
        }
    }

    impl SyntaxTextRange for ops::RangeTo<u32> {
        fn start(&self) -> Option<u32> {
            None
        }
        fn end(&self) -> Option<u32> {
            Some(self.end)
        }
    }

    impl SyntaxTextRange for ops::RangeFull {
        fn start(&self) -> Option<u32> {
            None
        }
        fn end(&self) -> Option<u32> {
            None
        }
    }
}
