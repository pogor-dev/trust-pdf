//! # Trivia Piece: Syntactic Trivia Representation
//!
//! This module defines `TriviaPiece`, which represents individual pieces of syntactic trivia
//! in the PDF syntax tree. Trivia includes whitespace, comments, and line breaks that are
//! significant for maintaining exact fidelity of the original document.
//!
//! ## What is Trivia?
//!
//! In compiler terminology, "trivia" refers to parts of the source text that don't directly
//! contribute to the program's meaning but are important for formatting and exact reproduction:
//! - Whitespace (spaces, tabs)
//! - Line breaks (LF, CR, CRLF)
//! - Comments
//!
//! ## PDF-Specific Trivia Significance
//!
//! Unlike many programming languages where whitespace is mostly cosmetic, PDF syntax
//! has specific whitespace requirements that affect document validity:
//!
//! ### Critical Whitespace Locations
//! - **Object headers**: `10 0 obj<newline>` - newline separates header from body
//! - **Stream boundaries**: `stream<newline>...data...<newline>endstream`
//! - **Cross-reference entries**: Fixed-width format with specific spacing
//! - **Content streams**: Operators separated by whitespace or newlines
//!
//! ### ISO 32000-2 References
//! - §7.3.10: Object structure and whitespace requirements
//! - §7.3.8: Stream object format
//! - §7.5.4: Cross-reference table format
//! - §8.1.1: Content stream syntax
//!
//! ## Memory Efficiency
//!
//! `TriviaPiece` stores only the kind and length of trivia, not the actual text.
//! This saves memory while allowing reconstruction of the original text when needed.
//!
//! ## Example Usage
//!
//! ```rust
//! use syntax::syntax::trivia_piece::TriviaPiece;
//!
//! // PDF object header formatting
//! let space = TriviaPiece::whitespace(1);      // Single space
//! let newline = TriviaPiece::line_feed(1);     // Required newline after obj
//!
//! // Cross-reference table entry (20 characters wide)
//! let xref_padding = TriviaPiece::whitespace(3); // Pad to exact width
//! ```

use crate::syntax::trivia_piece_kind::TriviaPieceKind;

/// Represents a single piece of syntactic trivia with its kind and length.
///
/// This structure efficiently stores trivia information without duplicating the actual
/// text content. The trivia text can be reconstructed by knowing the kind and length.
///
/// # Memory Layout
///
/// The struct is designed to be compact:
/// - `kind`: 1 byte enum representing the type of trivia
/// - `length`: 4 bytes representing the length in bytes
///
/// Total size: 8 bytes (with padding) per trivia piece
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TriviaPiece {
    /// The kind of trivia (whitespace, newline, comment, etc.)
    pub(crate) kind: TriviaPieceKind,
    /// The length of this trivia piece in bytes
    pub(crate) length: u32,
}

impl TriviaPiece {
    /// Creates a new trivia piece with the specified kind and length.
    ///
    /// # Parameters
    /// - `kind`: The type of trivia this piece represents
    /// - `length`: The length in bytes (automatically converted from various integer types)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::{trivia_piece::TriviaPiece, trivia_piece_kind::TriviaPieceKind};
    ///
    /// let space = TriviaPiece::new(TriviaPieceKind::Whitespace, 4);
    /// let comment = TriviaPiece::new(TriviaPieceKind::Comment, 25);
    /// ```
    pub fn new<L: Into<u32>>(kind: TriviaPieceKind, length: L) -> Self {
        Self {
            kind,
            length: length.into(),
        }
    }

    /// Creates a whitespace trivia piece.
    ///
    /// Whitespace in PDF can include spaces, tabs, and other horizontal whitespace.
    /// The exact whitespace characters are context-dependent and may be significant
    /// for proper PDF formatting.
    ///
    /// # PDF Context
    /// Whitespace is used to separate tokens in PDF content streams and is required
    /// in specific locations like after object numbers and before object keywords.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece::TriviaPiece;
    ///
    /// let single_space = TriviaPiece::whitespace(1);
    /// let four_spaces = TriviaPiece::whitespace(4);
    /// let tab_equivalent = TriviaPiece::whitespace(8);
    /// ```
    pub fn whitespace<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::Whitespace, len)
    }

    /// Creates a carriage return trivia piece.
    ///
    /// Represents `\r` characters. In PDF context, this is primarily used for
    /// compatibility with systems that use CR as line endings.
    ///
    /// # PDF Context
    /// While PDF specification allows CR as line ending, it's less common than LF.
    /// Most PDF files use LF or CRLF line endings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece::TriviaPiece;
    ///
    /// let cr = TriviaPiece::carriage_return(1);
    /// ```
    pub fn carriage_return<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::CarriageReturn, len)
    }

    /// Creates a line feed trivia piece.
    ///
    /// Represents `\n` characters. This is the most common line ending in PDF files
    /// and is required in specific syntactic locations.
    ///
    /// # PDF Context
    /// Line feeds are critical in PDF syntax for:
    /// - Separating object headers from content
    /// - Marking stream boundaries
    /// - Structuring cross-reference tables
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece::TriviaPiece;
    ///
    /// let newline = TriviaPiece::line_feed(1);
    /// ```
    pub fn line_feed<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::LineFeed, len)
    }

    /// Creates a carriage return + line feed trivia piece.
    ///
    /// Represents `\r\n` sequences. Common in PDF files created on Windows systems
    /// or when explicitly using CRLF line endings.
    ///
    /// # PDF Context
    /// CRLF sequences are treated as single line breaks in PDF parsing and are
    /// equivalent to single LF characters for syntactic purposes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece::TriviaPiece;
    ///
    /// let windows_newline = TriviaPiece::carriage_return_line_feed(2);
    /// ```
    pub fn carriage_return_line_feed<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::CarriageReturnLineFeed, len)
    }

    /// Creates a comment trivia piece.
    ///
    /// Comments in PDF files start with `%` and continue to the end of the line.
    /// They are ignored during PDF processing but must be preserved for exact
    /// document reproduction.
    ///
    /// # PDF Context
    /// PDF comments are used for:
    /// - Metadata and annotations in the source
    /// - Debugging information
    /// - Tool-specific markers
    /// - Header information (e.g., `%PDF-1.7`)
    ///
    /// # Note
    /// This method creates comment trivia that does not contain line breaks.
    /// Multi-line comments should be represented as separate trivia pieces.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece::TriviaPiece;
    ///
    /// let header_comment = TriviaPiece::comment(8);  // "%PDF-1.7"
    /// let metadata_comment = TriviaPiece::comment(25); // "% Created by TrustPDF"
    /// ```
    pub fn comment<L: Into<u32>>(len: L) -> Self {
        Self::new(TriviaPieceKind::Comment, len)
    }

    /// Returns the length of this trivia piece in bytes.
    ///
    /// This represents the number of bytes the trivia occupies in the original
    /// PDF file. It can be used to reconstruct the original spacing and formatting.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece::TriviaPiece;
    ///
    /// let space = TriviaPiece::whitespace(4);
    /// assert_eq!(space.text_len(), 4);
    ///
    /// let comment = TriviaPiece::comment(15);
    /// assert_eq!(comment.text_len(), 15);
    /// ```
    pub fn text_len(&self) -> u32 {
        self.length
    }

    /// Returns the kind of trivia this piece represents.
    ///
    /// This is useful for determining how to handle the trivia during parsing,
    /// formatting, or reconstruction operations.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::{trivia_piece::TriviaPiece, trivia_piece_kind::TriviaPieceKind};
    ///
    /// let space = TriviaPiece::whitespace(2);
    /// assert_eq!(space.kind(), TriviaPieceKind::Whitespace);
    ///
    /// let newline = TriviaPiece::line_feed(1);
    /// assert_eq!(newline.kind(), TriviaPieceKind::LineFeed);
    /// ```
    pub fn kind(&self) -> TriviaPieceKind {
        self.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivia_piece_creation() {
        let whitespace = TriviaPiece::whitespace(4u32);
        assert_eq!(whitespace.kind(), TriviaPieceKind::Whitespace);
        assert_eq!(whitespace.text_len(), 4);

        let comment = TriviaPiece::comment(15u32);
        assert_eq!(comment.kind(), TriviaPieceKind::Comment);
        assert_eq!(comment.text_len(), 15);

        let newline = TriviaPiece::line_feed(1u32);
        assert_eq!(newline.kind(), TriviaPieceKind::LineFeed);
        assert_eq!(newline.text_len(), 1);
    }

    #[test]
    fn test_trivia_piece_new() {
        let piece = TriviaPiece::new(TriviaPieceKind::CarriageReturn, 2u32);
        assert_eq!(piece.kind(), TriviaPieceKind::CarriageReturn);
        assert_eq!(piece.text_len(), 2);
    }

    #[test]
    fn test_line_ending_types() {
        let cr = TriviaPiece::carriage_return(1u32);
        let lf = TriviaPiece::line_feed(1u32);
        let crlf = TriviaPiece::carriage_return_line_feed(2u32);

        assert_eq!(cr.kind(), TriviaPieceKind::CarriageReturn);
        assert_eq!(lf.kind(), TriviaPieceKind::LineFeed);
        assert_eq!(crlf.kind(), TriviaPieceKind::CarriageReturnLineFeed);

        assert_eq!(cr.text_len(), 1);
        assert_eq!(lf.text_len(), 1);
        assert_eq!(crlf.text_len(), 2);
    }

    #[test]
    fn test_pdf_object_header_trivia() {
        // Simulate "10 0 obj\n" - object header format
        let space1 = TriviaPiece::whitespace(1u32); // After object number
        let space2 = TriviaPiece::whitespace(1u32); // After generation number
        let newline = TriviaPiece::line_feed(1u32); // After "obj" keyword

        assert_eq!(
            space1.text_len() + space2.text_len() + newline.text_len(),
            3
        );
    }

    #[test]
    fn test_pdf_stream_boundary_trivia() {
        // Simulate "stream\n...data...\nendstream" format
        let stream_newline = TriviaPiece::line_feed(1u32); // After "stream"
        let endstream_newline = TriviaPiece::line_feed(1u32); // Before "endstream"

        assert_eq!(stream_newline.kind(), TriviaPieceKind::LineFeed);
        assert_eq!(endstream_newline.kind(), TriviaPieceKind::LineFeed);
    }

    #[test]
    fn test_pdf_xref_table_trivia() {
        // Cross-reference entries are fixed-width with specific spacing
        let xref_padding = TriviaPiece::whitespace(3u32); // Padding to 20 characters
        let xref_newline = TriviaPiece::line_feed(1u32); // End of xref entry

        assert_eq!(xref_padding.text_len(), 3);
        assert_eq!(xref_newline.text_len(), 1);
    }

    #[test]
    fn test_pdf_header_comment() {
        // PDF files start with "%PDF-1.7" or similar
        let pdf_header = TriviaPiece::comment(8u32); // "%PDF-1.7"
        assert_eq!(pdf_header.kind(), TriviaPieceKind::Comment);
        assert_eq!(pdf_header.text_len(), 8);
    }

    #[test]
    fn test_trivia_piece_equality() {
        let space1 = TriviaPiece::whitespace(4u32);
        let space2 = TriviaPiece::whitespace(4u32);
        let space3 = TriviaPiece::whitespace(2u32);

        assert_eq!(space1, space2);
        assert_ne!(space1, space3);
    }

    #[test]
    fn test_trivia_piece_cloning() {
        let original = TriviaPiece::comment(20u32);
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.kind(), cloned.kind());
        assert_eq!(original.text_len(), cloned.text_len());
    }

    #[test]
    fn test_length_conversion() {
        // Test various integer types can be converted to length
        let from_u8 = TriviaPiece::whitespace(10u8);
        let from_u16 = TriviaPiece::whitespace(10u16);
        let from_u32 = TriviaPiece::whitespace(10u32);

        assert_eq!(from_u8.text_len(), 10);
        assert_eq!(from_u16.text_len(), 10);
        assert_eq!(from_u32.text_len(), 10);
    }
}
