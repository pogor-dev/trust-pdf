//! Syntax Trivia Management for PDF Parsing
//!
//! This module provides the `SyntaxTrivia` type, which represents the "trivia" associated with
//! PDF syntax tokens. In compiler design, trivia refers to syntactic elements that don't affect
//! the logical structure of the document but are crucial for maintaining the exact original format.
//!
//! ## What is Trivia in PDF Context?
//!
//! In PDF files, trivia includes whitespace, line breaks, and comments that have specific semantic
//! significance according to the ISO 32000-2 standard:
//!
//! - **Whitespace and line breaks** in `obj` declarations: The newline after the object header
//!   separates it from the object body (ISO 32000-2 §7.3.10)
//! - **Stream formatting**: The `stream` keyword must be followed by a newline (ISO 32000-2 §7.3.8)
//! - **Cross-reference table entries**: Fixed-width, space-padded formatting (ISO 32000-2 §7.5.4)
//! - **Content stream operators**: Space-separated tokens only (ISO 32000-2 §8.1.1)
//!
//! ## Leading vs Trailing Trivia
//!
//! Trivia can be either "leading" (appearing before a token) or "trailing" (appearing after a token).
//! For example, in this PDF snippet:
//! ```text
//! % This is a comment (leading trivia)
//! 42 0 obj   % Another comment (trailing trivia)
//! ```
//!
//! This design follows the Roslyn approach, ensuring that when reconstructing the original PDF,
//! all formatting and whitespace is preserved exactly as it appeared.

use std::{fmt, ops::Range};

use crate::{
    cursor::{token::SyntaxToken, trivia_pieces_iterator::SyntaxTriviaPiecesIterator},
    green::trivia::GreenTrivia,
};

/// Represents trivia (whitespace, comments, formatting) associated with a PDF syntax token.
///
/// In PDF parsing, trivia encompasses all the "non-essential" syntactic elements that don't
/// affect the logical structure but are crucial for preserving the exact original formatting.
/// This includes whitespace, line breaks, and comments that have specific semantic significance
/// in PDF files according to the ISO 32000-2 standard.
///
/// ## Why Trivia Matters in PDF
///
/// Unlike many programming languages where whitespace is purely cosmetic, PDF has strict
/// formatting requirements in certain contexts:
///
/// - Object declarations require specific newline placement
/// - Stream keywords must be followed by newlines
/// - Cross-reference tables use fixed-width formatting
/// - Content streams use space-separated tokens exclusively
///
/// ## Design Philosophy
///
/// This implementation follows the Roslyn compiler architecture, where trivia is attached
/// to tokens rather than being separate nodes in the syntax tree. This ensures:
///
/// - **Lossless round-trip**: The original PDF can be reconstructed exactly
/// - **Efficient memory usage**: Trivia doesn't create additional tree nodes
/// - **Clear ownership**: Each piece of trivia belongs to exactly one token
///
/// ## Example Usage
///
/// ```rust,ignore
/// // For a PDF token like "42 0 obj", the trivia might include:
/// // - Leading trivia: comments or whitespace before "42"
/// // - Trailing trivia: whitespace and comments after "obj"
/// let trivia = token.leading_trivia();
/// let text = trivia.text(); // Get the raw bytes of the trivia
/// let range = trivia.text_range(); // Get the position in the original file
/// ```
#[derive(PartialEq, Eq, Clone, Hash)]
pub(crate) struct SyntaxTrivia {
    /// The syntax token that owns this trivia.
    ///
    /// This creates a parent-child relationship where the trivia is always associated
    /// with a specific token in the syntax tree. The token provides context about
    /// where this trivia appears in the overall PDF structure.
    token: SyntaxToken,

    /// Whether this trivia appears before (`true`) or after (`false`) its associated token.
    ///
    /// Leading trivia includes all whitespace, comments, and formatting that appears
    /// immediately before the token. Trailing trivia includes everything that appears
    /// immediately after the token, up to (but not including) the next significant token.
    ///
    /// This distinction is important for PDF formatting rules. For example, in an object
    /// declaration like `42 0 obj`, the space before `obj` is trailing trivia of the `0`
    /// token, while any comment after `obj` would be trailing trivia of the `obj` token.
    is_leading: bool,
}

impl SyntaxTrivia {
    /// Extracts the raw byte content of this trivia from the original PDF source.
    ///
    /// This method returns the actual bytes that make up the trivia, which could include:
    /// - Whitespace characters (spaces, tabs)
    /// - Line endings (CR, LF, or CRLF sequences)
    /// - PDF comments (starting with `%` and extending to end of line)
    /// - Any other non-token content that preserves the original formatting
    ///
    /// ## Return Value
    ///
    /// Returns a byte slice containing the trivia content. The bytes are UTF-8 compatible
    /// for text content, but may contain arbitrary bytes for binary PDF content.
    ///
    /// ## Implementation Details
    ///
    /// This method calculates the relative position of the trivia within its parent token's
    /// text and extracts the corresponding slice. The calculation accounts for whether this
    /// is leading or trailing trivia and adjusts the range accordingly.
    ///
    /// ## Example
    ///
    /// For a PDF line like `% A comment\n42 0 obj`, if this trivia represents the comment
    /// and newline, `text()` would return the bytes for `% A comment\n`.
    pub(crate) fn text(&self) -> &[u8] {
        let trivia_range = self.text_range();
        let token_offset = self.token.data().offset;

        let relative_range = Range {
            start: (trivia_range.start - token_offset) as usize,
            end: (trivia_range.end - trivia_range.start) as usize,
        };

        &self.token.text()[relative_range]
    }

    /// Returns the number of individual trivia pieces contained within this trivia.
    ///
    /// Trivia can be composed of multiple distinct pieces, each representing a different
    /// type of non-token content. For example, a trivia might contain:
    /// - A comment piece (`% This is a comment`)
    /// - A whitespace piece (`   `)  
    /// - A newline piece (`\n`)
    ///
    /// ## Use Cases
    ///
    /// This count is useful for:
    /// - **Memory management**: Pre-allocating collections when iterating over pieces
    /// - **Diagnostics**: Reporting the complexity of formatting in a PDF section
    /// - **Analysis**: Understanding the formatting density of different PDF regions
    /// - **Optimization**: Deciding whether to process trivia piece-by-piece or as a whole
    ///
    /// ## Performance Considerations
    ///
    /// This method delegates to the underlying `GreenTrivia` structure, which maintains
    /// the count efficiently without needing to iterate through all pieces.
    ///
    /// ## Example
    ///
    /// ```rust,ignore
    /// // For trivia like "  % comment\n  ", this might return 3:
    /// // 1. Leading whitespace: "  "
    /// // 2. Comment: "% comment"  
    /// // 3. Trailing whitespace and newline: "\n  "
    /// let piece_count = trivia.len();
    /// ```
    #[allow(dead_code)] // This method will be used when implementing trivia analysis
    pub(crate) fn len(&self) -> usize {
        self.green_trivia().len()
    }

    /// Calculates the absolute text range that this trivia occupies in the original PDF source.
    ///
    /// This method returns the start and end positions (as byte offsets) where this trivia
    /// appears in the complete PDF file. The range accounts for whether this is leading or
    /// trailing trivia and positions it correctly relative to its associated token.
    ///
    /// ## Range Calculation
    ///
    /// - **Leading trivia**: Positioned immediately before the token, starting from the
    ///   token's start position and extending backwards by the trivia length
    /// - **Trailing trivia**: Positioned immediately after the token, starting from the
    ///   token's end position and extending forwards by the trivia length
    ///
    /// ## Use Cases
    ///
    /// The text range is essential for:
    /// - **Source mapping**: Linking trivia back to specific locations in the original PDF
    /// - **Error reporting**: Providing accurate position information in diagnostics
    /// - **Editor integration**: Enabling features like syntax highlighting and hover tooltips
    /// - **Incremental parsing**: Determining which trivia needs to be re-parsed after edits
    ///
    /// ## Return Value
    ///
    /// Returns a `Range<u64>` where:
    /// - `start`: The byte offset where this trivia begins in the source file
    /// - `end`: The byte offset where this trivia ends in the source file
    ///
    /// ## Example
    ///
    /// For a PDF with content `42 0 obj % comment\n`, if this trivia represents the
    /// comment and newline, the range might be `Range { start: 9, end: 19 }`.
    pub(crate) fn text_range(&self) -> Range<u64> {
        let length = self.green_trivia().text_len();
        let token_range = self.token.text_range();

        match self.is_leading {
            true => Range {
                start: token_range.start,
                end: length,
            },
            false => Range {
                start: token_range.end - length,
                end: length,
            },
        }
    }

    /// Retrieves the underlying green trivia data for this syntax trivia.
    ///
    /// This is a private helper method that accesses the low-level trivia representation
    /// from the green tree layer. The "green" layer in this architecture (inspired by
    /// Roslyn and Rust Analyzer) represents the immutable, memory-efficient tree structure
    /// without parent pointers or absolute positions.
    ///
    /// ## Leading vs Trailing Access
    ///
    /// Depending on whether this trivia is leading or trailing, this method accesses
    /// different trivia collections from the associated token:
    /// - Leading trivia is stored before the token content
    /// - Trailing trivia is stored after the token content
    ///
    /// ## Privacy Rationale
    ///
    /// This method is private because external code should interact with trivia through
    /// the high-level `SyntaxTrivia` API rather than the low-level green structures.
    /// This encapsulation allows for future optimizations and API evolution.
    fn green_trivia(&self) -> &GreenTrivia {
        match self.is_leading {
            true => self.token.green().leading_trivia(),
            false => self.token.green().trailing_trivia(),
        }
    }

    /// Creates an iterator over the individual pieces that comprise this trivia.
    ///
    /// Trivia often consists of multiple distinct pieces, each with its own type and content.
    /// For example, a complex trivia section might contain:
    /// - Whitespace (spaces, tabs)
    /// - Comments (PDF comments starting with `%`)
    /// - Line endings (various newline formats)
    /// - Mixed content combinations
    ///
    /// ## Iterator Behavior
    ///
    /// The returned iterator yields `SyntaxTriviaPiece` items that provide:
    /// - **Position information**: The absolute offset of each piece in the source file
    /// - **Content access**: The raw bytes and interpreted meaning of each piece
    /// - **Type classification**: Whether the piece is whitespace, comment, newline, etc.
    ///
    /// ## Use Cases
    ///
    /// Iterating over trivia pieces enables:
    /// - **Detailed formatting analysis**: Understanding the specific structure of whitespace
    /// - **Comment extraction**: Finding and processing PDF comments for documentation
    /// - **Whitespace normalization**: Converting between different spacing conventions
    /// - **Syntax highlighting**: Applying different colors to different trivia types
    /// - **Format preservation**: Maintaining exact spacing during PDF transformations
    ///
    /// ## Performance Considerations
    ///
    /// The iterator is lazy and only processes pieces as they are requested. For large
    /// trivia sections, this approach is more memory-efficient than collecting all pieces
    /// at once.
    ///
    /// ## Example Usage
    ///
    /// ```rust,ignore
    /// for piece in trivia.pieces() {
    ///     match piece.kind() {
    ///         TriviaKind::Whitespace => { /* handle spacing */ },
    ///         TriviaKind::Comment => { /* process PDF comment */ },
    ///         TriviaKind::Newline => { /* handle line breaks */ },
    ///     }
    /// }
    /// ```
    #[allow(dead_code)] // This method will be used when implementing detailed trivia analysis
    pub(crate) fn pieces(&self) -> SyntaxTriviaPiecesIterator {
        let range = self.text_range();
        SyntaxTriviaPiecesIterator {
            raw: self.clone(),
            next_index: 0,
            next_offset: range.start,
            end_index: self.len(),
            end_offset: range.end,
        }
    }
}

/// Debug implementation for `SyntaxTrivia` that provides useful diagnostic information.
///
/// This implementation focuses on the most important aspects of trivia for debugging:
/// the text range it occupies in the source file. This is typically the most useful
/// information when diagnosing parsing issues or tracking down formatting problems.
///
/// The output format is designed to be concise while providing the essential information
/// needed to locate and understand the trivia in the context of the overall PDF structure.
impl fmt::Debug for SyntaxTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = f.debug_struct("SyntaxTrivia");
        f.field("text_range", &self.text_range());
        f.finish()
    }
}

/// Display implementation for `SyntaxTrivia` that renders the trivia content as text.
///
/// This implementation converts the raw trivia bytes into a displayable string format,
/// handling the conversion from bytes to UTF-8 text. Since PDF files can contain mixed
/// binary and text content, this uses `String::from_utf8_lossy()` to gracefully handle
/// any non-UTF-8 sequences by replacing them with the Unicode replacement character (�).
///
/// ## Use Cases
///
/// The `Display` implementation is useful for:
/// - **Debugging**: Quickly seeing what whitespace or comments are present
/// - **Logging**: Including trivia content in diagnostic messages
/// - **Testing**: Verifying that the correct trivia was parsed
/// - **Development tools**: Showing trivia content in IDE previews
///
/// ## Encoding Handling
///
/// PDF files primarily use ASCII and Latin-1 encoding for text content, which are
/// compatible with UTF-8. However, some PDF content might use other encodings or
/// contain binary data. The lossy conversion ensures that display never fails,
/// even for malformed or binary content.
///
/// ## Example Output
///
/// For trivia containing `  % This is a PDF comment\n`, the display output would be
/// exactly that string, preserving all whitespace and formatting characters.
impl fmt::Display for SyntaxTrivia {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(self.text()))
    }
}
