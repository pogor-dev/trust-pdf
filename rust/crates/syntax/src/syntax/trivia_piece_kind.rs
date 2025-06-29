//! # Trivia Piece Kind: Classification of Syntactic Trivia
//!
//! This module defines `TriviaPieceKind`, an enumeration that classifies different types
//! of syntactic trivia found in PDF files. Each kind represents a specific category of
//! non-semantic content that must be preserved for exact document reproduction.
//!
//! ## PDF Trivia Categories
//!
//! PDF syntax has specific requirements for different types of trivia:
//!
//! ### Line Endings
//! PDF supports multiple line ending formats, all of which are significant:
//! - **LF** (`\n`): Most common, required after `stream` keyword
//! - **CR** (`\r`): Less common, legacy system compatibility  
//! - **CRLF** (`\r\n`): Windows-style, treated as single line break
//!
//! ### Whitespace
//! Horizontal whitespace (spaces, tabs) used for:
//! - Token separation in content streams
//! - Fixed-width formatting in cross-reference tables
//! - Object header formatting
//!
//! ### Comments
//! PDF comments start with `%` and continue to end of line:
//! - File headers (`%PDF-1.7`)
//! - Metadata and tool markers
//! - Debug information
//!
//! ### Skipped Content
//! Content that parser couldn't interpret but must preserve:
//! - Malformed tokens
//! - Unknown keywords
//! - Recovery content
//!
//! ## ISO 32000-2 References
//! - §7.2.3: Character set and encoding
//! - §7.3.10: Object structure whitespace rules
//! - §7.3.8: Stream object line ending requirements
//! - §7.5.4: Cross-reference table formatting

/// Classifies different types of syntactic trivia found in PDF files.
///
/// This enumeration provides a way to categorize non-semantic content that appears
/// between tokens. Each variant represents a specific type of trivia that has
/// particular significance in PDF syntax.
///
/// # Design Rationale
///
/// The enum is designed to be:
/// - **Copy**: Trivia kinds are small values that should be cheaply copyable
/// - **Hash + Eq**: Used in collections and comparisons
/// - **Debug**: Essential for debugging parser issues
///
/// # Memory Layout
///
/// As a simple enum with no data, this takes only 1 byte of memory per instance.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum TriviaPieceKind {
    /// Carriage return character (`\r`).
    ///
    /// Found in PDF files from systems that use CR as line ending.
    /// Less common than LF but must be preserved for exact reproduction.
    ///
    /// # PDF Context
    /// While PDF specification allows CR as line ending, it's primarily seen in:
    /// - Legacy PDF files from older Mac systems
    /// - Files that have been processed by certain tools
    /// - Mixed line-ending scenarios
    CarriageReturn,

    /// Line feed character (`\n`).
    ///
    /// The most common line ending in PDF files. Required in specific syntactic
    /// locations such as after the `stream` keyword.
    ///
    /// # PDF Context
    /// Critical for proper PDF structure:
    /// - Required after `stream` keyword (ISO 32000-2 §7.3.8)
    /// - Separates object headers from content (ISO 32000-2 §7.3.10)
    /// - Terminates cross-reference entries (ISO 32000-2 §7.5.4)
    LineFeed,

    /// Carriage return followed by line feed (`\r\n`).
    ///
    /// Windows-style line ending that's treated as a single line break in PDF parsing.
    /// Common in PDF files created on Windows systems.
    ///
    /// # PDF Context
    /// Functionally equivalent to single LF for PDF syntax purposes, but must be
    /// preserved to maintain exact byte-for-byte reproduction capability.
    CarriageReturnLineFeed,

    /// Horizontal whitespace (spaces, tabs, etc.).
    ///
    /// Used for token separation and formatting. In PDF syntax, whitespace is
    /// often syntactically significant and required in specific locations.
    ///
    /// # PDF Context
    /// Essential for:
    /// - Separating tokens in content streams (ISO 32000-2 §8.1.1)
    /// - Fixed-width formatting in cross-reference tables
    /// - Object number and generation formatting
    /// - Dictionary key-value separation
    Whitespace,

    /// Comment content starting with `%` and continuing to end of line.
    ///
    /// PDF comments are used for metadata, tool markers, and the file header.
    /// They're ignored during processing but must be preserved.
    ///
    /// # PDF Context
    /// Comments serve several purposes:
    /// - File format identification (`%PDF-1.7`)
    /// - Binary marker (`%âãÏÓ`)
    /// - Tool and creator identification
    /// - Debugging and development notes
    ///
    /// # Note
    /// This variant represents comment content only, not including line breaks.
    /// Line breaks within or after comments are separate trivia pieces.
    Comment,

    /// Content that the parser couldn't interpret but needs to preserve.
    ///
    /// Used during error recovery to maintain document fidelity even when
    /// encountering malformed or unrecognized content.
    ///
    /// # PDF Context
    /// PDF files often contain:
    /// - Malformed tokens that should be preserved
    /// - Unknown keywords from newer PDF versions
    /// - Corrupted content that might be recoverable
    /// - Non-standard extensions from specific tools
    ///
    /// # Error Recovery
    /// The parser uses this category to continue processing while preserving
    /// problematic content that might be meaningful to other tools or contain
    /// recoverable information.
    Skipped,
}

impl TriviaPieceKind {
    /// Returns `true` if this trivia is a carriage return character.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece_kind::TriviaPieceKind;
    ///
    /// assert!(TriviaPieceKind::CarriageReturn.is_carriage_return());
    /// assert!(!TriviaPieceKind::LineFeed.is_carriage_return());
    /// ```
    pub const fn is_carriage_return(&self) -> bool {
        matches!(self, TriviaPieceKind::CarriageReturn)
    }

    /// Returns `true` if this trivia is a line feed character.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece_kind::TriviaPieceKind;
    ///
    /// assert!(TriviaPieceKind::LineFeed.is_line_feed());
    /// assert!(!TriviaPieceKind::CarriageReturn.is_line_feed());
    /// ```
    pub const fn is_line_feed(&self) -> bool {
        matches!(self, TriviaPieceKind::LineFeed)
    }

    /// Returns `true` if this trivia is a carriage return + line feed sequence.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece_kind::TriviaPieceKind;
    ///
    /// assert!(TriviaPieceKind::CarriageReturnLineFeed.is_carriage_return_line_feed());
    /// assert!(!TriviaPieceKind::LineFeed.is_carriage_return_line_feed());
    /// ```
    pub const fn is_carriage_return_line_feed(&self) -> bool {
        matches!(self, TriviaPieceKind::CarriageReturnLineFeed)
    }

    /// Returns `true` if this trivia represents any kind of line ending.
    ///
    /// This is useful for checking if trivia marks the end of a line, regardless
    /// of the specific line ending format used.
    ///
    /// # PDF Context
    /// Line endings are significant in PDF syntax for:
    /// - Object structure boundaries
    /// - Stream content boundaries  
    /// - Cross-reference table formatting
    /// - Comment termination
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece_kind::TriviaPieceKind;
    ///
    /// assert!(TriviaPieceKind::LineFeed.is_newline());
    /// assert!(TriviaPieceKind::CarriageReturn.is_newline());
    /// assert!(TriviaPieceKind::CarriageReturnLineFeed.is_newline());
    /// assert!(!TriviaPieceKind::Whitespace.is_newline());
    /// ```
    pub const fn is_newline(&self) -> bool {
        matches!(
            self,
            TriviaPieceKind::CarriageReturn
                | TriviaPieceKind::LineFeed
                | TriviaPieceKind::CarriageReturnLineFeed
        )
    }

    /// Returns `true` if this trivia is horizontal whitespace.
    ///
    /// # PDF Context
    /// Whitespace is critical for PDF syntax:
    /// - Token separation in content streams
    /// - Fixed-width cross-reference entries
    /// - Object header formatting
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece_kind::TriviaPieceKind;
    ///
    /// assert!(TriviaPieceKind::Whitespace.is_whitespace());
    /// assert!(!TriviaPieceKind::LineFeed.is_whitespace());
    /// ```
    pub const fn is_whitespace(&self) -> bool {
        matches!(self, TriviaPieceKind::Whitespace)
    }

    /// Returns `true` if this trivia is comment content.
    ///
    /// # PDF Context
    /// Comments in PDF serve multiple purposes:
    /// - File format headers (`%PDF-1.7`)
    /// - Binary content markers
    /// - Tool identification and metadata
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece_kind::TriviaPieceKind;
    ///
    /// assert!(TriviaPieceKind::Comment.is_comment());
    /// assert!(!TriviaPieceKind::Whitespace.is_comment());
    /// ```
    pub const fn is_comment(&self) -> bool {
        matches!(self, TriviaPieceKind::Comment)
    }

    /// Returns `true` if this trivia represents skipped/unparseable content.
    ///
    /// This indicates content that the parser couldn't interpret but preserved
    /// for potential recovery or inspection.
    ///
    /// # Error Recovery Context
    /// Skipped content allows the parser to continue processing while preserving
    /// potentially meaningful content that might be:
    /// - Malformed but recoverable
    /// - From newer PDF versions
    /// - Tool-specific extensions
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece_kind::TriviaPieceKind;
    ///
    /// assert!(TriviaPieceKind::Skipped.is_skipped());
    /// assert!(!TriviaPieceKind::Comment.is_skipped());
    /// ```
    pub const fn is_skipped(&self) -> bool {
        matches!(self, TriviaPieceKind::Skipped)
    }

    /// Returns `true` if this trivia represents content that affects line positioning.
    ///
    /// This includes both newlines and carriage returns, which affect cursor position
    /// during parsing and text reconstruction.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece_kind::TriviaPieceKind;
    ///
    /// assert!(TriviaPieceKind::LineFeed.affects_line_position());
    /// assert!(TriviaPieceKind::CarriageReturn.affects_line_position());
    /// assert!(!TriviaPieceKind::Whitespace.affects_line_position());
    /// ```
    pub const fn affects_line_position(&self) -> bool {
        self.is_newline()
    }

    /// Returns `true` if this trivia represents significant whitespace in PDF context.
    ///
    /// This includes both horizontal whitespace and line endings, both of which
    /// can be syntactically significant in PDF files.
    ///
    /// # PDF Context
    /// All whitespace types can be significant:
    /// - Horizontal: token separation, fixed-width formatting
    /// - Vertical: structure boundaries, required separators
    ///
    /// # Examples
    ///
    /// ```rust
    /// use syntax::syntax::trivia_piece_kind::TriviaPieceKind;
    ///
    /// assert!(TriviaPieceKind::Whitespace.is_space_or_newline());
    /// assert!(TriviaPieceKind::LineFeed.is_space_or_newline());
    /// assert!(!TriviaPieceKind::Comment.is_space_or_newline());
    /// ```
    pub const fn is_space_or_newline(&self) -> bool {
        self.is_whitespace() || self.is_newline()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carriage_return_identification() {
        assert!(TriviaPieceKind::CarriageReturn.is_carriage_return());
        assert!(!TriviaPieceKind::LineFeed.is_carriage_return());
        assert!(!TriviaPieceKind::CarriageReturnLineFeed.is_carriage_return());
        assert!(!TriviaPieceKind::Whitespace.is_carriage_return());
        assert!(!TriviaPieceKind::Comment.is_carriage_return());
        assert!(!TriviaPieceKind::Skipped.is_carriage_return());
    }

    #[test]
    fn test_line_feed_identification() {
        assert!(!TriviaPieceKind::CarriageReturn.is_line_feed());
        assert!(TriviaPieceKind::LineFeed.is_line_feed());
        assert!(!TriviaPieceKind::CarriageReturnLineFeed.is_line_feed());
        assert!(!TriviaPieceKind::Whitespace.is_line_feed());
        assert!(!TriviaPieceKind::Comment.is_line_feed());
        assert!(!TriviaPieceKind::Skipped.is_line_feed());
    }

    #[test]
    fn test_carriage_return_line_feed_identification() {
        assert!(!TriviaPieceKind::CarriageReturn.is_carriage_return_line_feed());
        assert!(!TriviaPieceKind::LineFeed.is_carriage_return_line_feed());
        assert!(TriviaPieceKind::CarriageReturnLineFeed.is_carriage_return_line_feed());
        assert!(!TriviaPieceKind::Whitespace.is_carriage_return_line_feed());
        assert!(!TriviaPieceKind::Comment.is_carriage_return_line_feed());
        assert!(!TriviaPieceKind::Skipped.is_carriage_return_line_feed());
    }

    #[test]
    fn test_newline_identification() {
        assert!(TriviaPieceKind::CarriageReturn.is_newline());
        assert!(TriviaPieceKind::LineFeed.is_newline());
        assert!(TriviaPieceKind::CarriageReturnLineFeed.is_newline());
        assert!(!TriviaPieceKind::Whitespace.is_newline());
        assert!(!TriviaPieceKind::Comment.is_newline());
        assert!(!TriviaPieceKind::Skipped.is_newline());
    }

    #[test]
    fn test_whitespace_identification() {
        assert!(!TriviaPieceKind::CarriageReturn.is_whitespace());
        assert!(!TriviaPieceKind::LineFeed.is_whitespace());
        assert!(!TriviaPieceKind::CarriageReturnLineFeed.is_whitespace());
        assert!(TriviaPieceKind::Whitespace.is_whitespace());
        assert!(!TriviaPieceKind::Comment.is_whitespace());
        assert!(!TriviaPieceKind::Skipped.is_whitespace());
    }

    #[test]
    fn test_comment_identification() {
        assert!(!TriviaPieceKind::CarriageReturn.is_comment());
        assert!(!TriviaPieceKind::LineFeed.is_comment());
        assert!(!TriviaPieceKind::CarriageReturnLineFeed.is_comment());
        assert!(!TriviaPieceKind::Whitespace.is_comment());
        assert!(TriviaPieceKind::Comment.is_comment());
        assert!(!TriviaPieceKind::Skipped.is_comment());
    }

    #[test]
    fn test_skipped_identification() {
        assert!(!TriviaPieceKind::CarriageReturn.is_skipped());
        assert!(!TriviaPieceKind::LineFeed.is_skipped());
        assert!(!TriviaPieceKind::CarriageReturnLineFeed.is_skipped());
        assert!(!TriviaPieceKind::Whitespace.is_skipped());
        assert!(!TriviaPieceKind::Comment.is_skipped());
        assert!(TriviaPieceKind::Skipped.is_skipped());
    }

    #[test]
    fn test_affects_line_position() {
        assert!(TriviaPieceKind::CarriageReturn.affects_line_position());
        assert!(TriviaPieceKind::LineFeed.affects_line_position());
        assert!(TriviaPieceKind::CarriageReturnLineFeed.affects_line_position());
        assert!(!TriviaPieceKind::Whitespace.affects_line_position());
        assert!(!TriviaPieceKind::Comment.affects_line_position());
        assert!(!TriviaPieceKind::Skipped.affects_line_position());
    }

    #[test]
    fn test_is_space_or_newline() {
        assert!(TriviaPieceKind::CarriageReturn.is_space_or_newline());
        assert!(TriviaPieceKind::LineFeed.is_space_or_newline());
        assert!(TriviaPieceKind::CarriageReturnLineFeed.is_space_or_newline());
        assert!(TriviaPieceKind::Whitespace.is_space_or_newline());
        assert!(!TriviaPieceKind::Comment.is_space_or_newline());
        assert!(!TriviaPieceKind::Skipped.is_space_or_newline());
    }

    #[test]
    fn test_pdf_specific_trivia_scenarios() {
        // Test PDF object header scenario: "10 0 obj\n"
        assert!(TriviaPieceKind::Whitespace.is_space_or_newline()); // spaces between numbers and obj
        assert!(TriviaPieceKind::LineFeed.is_newline()); // required newline after obj

        // Test PDF stream boundary: "stream\n"
        assert!(TriviaPieceKind::LineFeed.affects_line_position()); // required after stream

        // Test PDF header comment: "%PDF-1.7"
        assert!(TriviaPieceKind::Comment.is_comment()); // file format identifier

        // Test cross-reference padding
        assert!(TriviaPieceKind::Whitespace.is_whitespace()); // fixed-width padding
    }

    #[test]
    fn test_error_recovery_scenarios() {
        // Test malformed content handling
        assert!(TriviaPieceKind::Skipped.is_skipped());
        assert!(!TriviaPieceKind::Skipped.is_space_or_newline());
        assert!(!TriviaPieceKind::Skipped.affects_line_position());
    }

    #[test]
    fn test_trivia_kind_equality_and_hashing() {
        use std::collections::HashSet;

        let mut kinds = HashSet::new();
        kinds.insert(TriviaPieceKind::Whitespace);
        kinds.insert(TriviaPieceKind::LineFeed);
        kinds.insert(TriviaPieceKind::Comment);

        assert!(kinds.contains(&TriviaPieceKind::Whitespace));
        assert!(kinds.contains(&TriviaPieceKind::LineFeed));
        assert!(!kinds.contains(&TriviaPieceKind::Skipped));
    }

    #[test]
    fn test_trivia_kind_cloning() {
        let original = TriviaPieceKind::CarriageReturnLineFeed;
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert!(cloned.is_carriage_return_line_feed());
    }
}
