/// Macro for building green syntax trees with a declarative syntax.
///
/// This macro provides a clean DSL for constructing syntax trees by automatically
/// translating high-level syntax into `GreenNodeBuilder` method calls. It handles
/// the entire builder lifecycle, from initialization to final tree construction.
///
/// # Expansion Steps
///
/// The macro expands through the following phases:
///
/// 1. Entry point (`tree! { ... }`):
///    - Creates a `GreenNodeBuilder` instance
///    - Delegates to `@elements` to process the tree structure
///    - Calls `builder.finish()` to return the root `GreenNode`
///
/// 2. Elements processing (`@elements`):
///    - Recursively processes nodes and tokens in document order
///    - Dispatches to node or token-specific rules based on syntax
///
/// 3. Node expansion (`KIND => { children }`):
///    - Calls `builder.start_node(KIND)`
///    - Recursively processes child elements
///    - Calls `builder.finish_node()`
///
/// 4. Token expansion (`(KIND) => { content }`):
///    - Calls `builder.start_token(KIND)`
///    - Delegates to `@token_content` to process trivia and text
///    - Calls `builder.finish_token()`
///
/// 5. Token shorthand (`(KIND, text)`):
///    - Direct shorthand for tokens with only text, no trivia
///    - Expands to `start_token`, `token_text`, `finish_token` sequence
///
/// 6. Token content processing (`@token_content`):
///    - Translates `text(value)` → `builder.token_text(value)`
///    - Translates `trivia(kind, value)` → `builder.trivia(kind, value)`
///    - Translates `diagnostic(...)` → `builder.diagnostic(...)`
///    - Processes calls sequentially, preserving order for leading/trailing trivia
///
/// 7. Diagnostic decorators (`@diagnostic(...)`):
///    - Can be placed above nodes or tokens
///    - Multiple diagnostics are supported
///    - Diagnostics are added to the element immediately following them
///
/// # Example Usage
///
/// ```ignore
/// use syntax::tree;
/// use syntax::SyntaxKind;
///
/// const PARENT: SyntaxKind = SyntaxKind(1);
/// const CHILD: SyntaxKind = SyntaxKind(2);
/// const TOKEN: SyntaxKind = SyntaxKind(3);
/// const SPACE: SyntaxKind = SyntaxKind(4);
///
/// let tree = tree! {
///     PARENT => {
///         @diagnostic(DiagnosticSeverity::Error, 1001, "Error message"),
///         (TOKEN) => { trivia(SPACE, b" "), text(b"foo") },
///         CHILD => {
///             (TOKEN, b"bar")
///         }
///     }
/// };
/// ```
///
/// # Syntax Rules
///
/// - Node: `KIND => { children }` - no parentheses around KIND
/// - Token: `(KIND) => { content }` - parentheses around KIND  
/// - Token shorthand: `(KIND, text)` - direct text assignment without trivia
/// - Token content: Comma-separated calls to `text()` and `trivia(kind, value)`
/// - Trivia placement: Calls before `text()` become leading trivia, calls after become trailing trivia
/// - Diagnostics: `@diagnostic(severity, code, "message")` - placed above tokens or nodes
#[macro_export]
macro_rules! tree {
    // [Diagnostic Collection Phase] Recursively collect consecutive @diagnostic decorators
    // Accumulates all diagnostics before processing the element they're attached to

    // Collect another diagnostic and continue collecting
    (@collect_diagnostics $builder:ident, [$($diags:tt)*], @diagnostic($($diag:tt)*), $($rest:tt)*) => {
        $crate::tree!(@collect_diagnostics $builder, [$($diags)* ($($diag)*),], $($rest)*);
    };

    // Done collecting diagnostics, found token shorthand (last element)
    (@collect_diagnostics $builder:ident, [$($diags:tt)*], ($kind:expr, $text:expr)) => {{
        $builder.start_token($kind);
        $builder.token_text($text);
        $builder.finish_token();
        $crate::tree!(@apply_diagnostics $builder, $($diags)*);
    }};

    // Done collecting diagnostics, found token shorthand (with more elements)
    (@collect_diagnostics $builder:ident, [$($diags:tt)*], ($kind:expr, $text:expr), $($rest:tt)*) => {{
        $builder.start_token($kind);
        $builder.token_text($text);
        $builder.finish_token();
        $crate::tree!(@apply_diagnostics $builder, $($diags)*);
        $crate::tree!(@elements $builder, $($rest)*);
    }};

    // Done collecting diagnostics, found expanded token (last element)
    (@collect_diagnostics $builder:ident, [$($diags:tt)*], ($kind:expr) => { $($content:tt)* }) => {{
        $builder.start_token($kind);
        { $crate::tree!(@token_content $builder, $($content)*); }
        $builder.finish_token();
        $crate::tree!(@apply_diagnostics $builder, $($diags)*);
    }};

    // Done collecting diagnostics, found expanded token (with more elements)
    (@collect_diagnostics $builder:ident, [$($diags:tt)*], ($kind:expr) => { $($content:tt)* }, $($rest:tt)*) => {{
        $builder.start_token($kind);
        { $crate::tree!(@token_content $builder, $($content)*); }
        $builder.finish_token();
        $crate::tree!(@apply_diagnostics $builder, $($diags)*);
        $crate::tree!(@elements $builder, $($rest)*);
    }};

    // Done collecting diagnostics, found node (last element)
    (@collect_diagnostics $builder:ident, [$($diags:tt)*], $kind:expr => { $($children:tt)* }) => {{
        $builder.start_node($kind);
        $crate::tree!(@elements $builder, $($children)*);
        $builder.finish_node();
        $crate::tree!(@apply_diagnostics $builder, $($diags)*);
    }};

    // Done collecting diagnostics, found node (with more elements)
    (@collect_diagnostics $builder:ident, [$($diags:tt)*], $kind:expr => { $($children:tt)* }, $($rest:tt)*) => {{
        $builder.start_node($kind);
        $crate::tree!(@elements $builder, $($children)*);
        $builder.finish_node();
        $crate::tree!(@apply_diagnostics $builder, $($diags)*);
        $crate::tree!(@elements $builder, $($rest)*);
    }};

    // [Apply Diagnostics] Recursively apply each collected diagnostic
    (@apply_diagnostics $builder:ident, ($($diag:tt)*), $($rest:tt)*) => {
        $builder.add_diagnostic($($diag)*).expect("Element already added in tree! macro");
        $crate::tree!(@apply_diagnostics $builder, $($rest)*);
    };

    // Base case: no more diagnostics to apply
    (@apply_diagnostics $builder:ident,) => {};

    // [Entry to diagnostic collection] Start collecting when we see @diagnostic
    (@elements $builder:ident, @diagnostic($($diag:tt)*), $($rest:tt)*) => {
        $crate::tree!(@collect_diagnostics $builder, [($($diag)*),], $($rest)*);
    };

    // [Step 1a] Token expansion (last token in sequence)
    // Matches: (KIND) => { content }
    (@elements $builder:ident, ($kind:expr) => { $($content:tt)* }) => {{
        $builder.start_token($kind);
        { $crate::tree!(@token_content $builder, $($content)*); }
        $builder.finish_token();
    }};

    // [Step 1b] Token expansion (followed by more elements)
    // Matches: (KIND) => { content }, rest...
    (@elements $builder:ident, ($kind:expr) => { $($content:tt)* }, $($rest:tt)*) => {{
        $builder.start_token($kind);
        { $crate::tree!(@token_content $builder, $($content)*); }
        $builder.finish_token();
        $crate::tree!(@elements $builder, $($rest)*);
    }};

    // [Step 1c] Token shorthand (last token in sequence)
    // Matches: (KIND, text)
    (@elements $builder:ident, ($kind:expr, $text:expr)) => {{
        $builder.start_token($kind);
        $builder.token_text($text);
        $builder.finish_token();
    }};

    // [Step 1d] Token shorthand (followed by more elements)
    // Matches: (KIND, text), rest...
    (@elements $builder:ident, ($kind:expr, $text:expr), $($rest:tt)*) => {{
        $builder.start_token($kind);
        $builder.token_text($text);
        $builder.finish_token();
        $crate::tree!(@elements $builder, $($rest)*);
    }};

    // [Step 2a] Node expansion (last node in sequence)
    // Matches: KIND => { children }
    (@elements $builder:ident, $kind:expr => { $($children:tt)* }) => {{
        $builder.start_node($kind);
        $crate::tree!(@elements $builder, $($children)*);
        $builder.finish_node();
    }};

    // [Step 2b] Node expansion (followed by more elements)
    // Matches: KIND => { children }, rest...
    (@elements $builder:ident, $kind:expr => { $($children:tt)* }, $($rest:tt)*) => {{
        $builder.start_node($kind);
        $crate::tree!(@elements $builder, $($children)*);
        $builder.finish_node();
        $crate::tree!(@elements $builder, $($rest)*);
    }};

    // [Step 3] Token content processing: translates text() and trivia() calls to builder methods
    // Processes token content sequentially, maintaining order for leading/trailing trivia.
    // Input: trivia(SPACE, b" "), text(b"foo"), trivia(LINEFEED, b"\n")
    // Output: builder.trivia(SPACE, b" "); builder.token_text(b"foo"); builder.trivia(LINEFEED, b"\n");

    // [Step 3a] Translate text() to builder.token_text()
    // Matches: text(expression)
    (@token_content $builder:ident, text($text:expr) $(, $($rest:tt)*)?) => {
        $builder.token_text($text);
        $($crate::tree!(@token_content $builder, $($rest)*);)?
    };

    // [Step 3b] Translate trivia() to builder.trivia()
    // Matches: trivia(kind, value)
    (@token_content $builder:ident, trivia($kind:expr, $value:expr) $(, $($rest:tt)*)?) => {
        $builder.trivia($kind, $value);
        $($crate::tree!(@token_content $builder, $($rest)*);)?
    };

    // [Step 3c] Base case: no more token content to process
    // Matches: empty token content
    (@token_content $builder:ident,) => {};

    // [Step 4] Base case: recursion ends when all elements are processed
    // Matches: empty element list
    (@elements $builder:ident,) => {};

    // [Step 0] Entry point: creates builder, passes to @elements, returns finished tree
    // Matches: any token tree (catch-all pattern)
    // Must be last since ($($tt:tt)*) matches everything
    ($($tt:tt)*) => {{
        let mut builder = $crate::GreenNodeBuilder::new();
        $crate::tree!(@elements builder, $($tt)*);
        builder.finish()
    }};
}

#[cfg(test)]
mod builder_tests {
    use crate::{GreenTrait, SyntaxKind, green::DiagnosticSeverity};
    use pretty_assertions::assert_eq;

    // Using existing SyntaxKind variants for tests
    const OBJECT: SyntaxKind = SyntaxKind::BadToken;
    const INDIRECT: SyntaxKind = SyntaxKind::IndirectObjectKeyword;
    const NUMBER: SyntaxKind = SyntaxKind::NumericLiteralToken;
    const KEYWORD: SyntaxKind = SyntaxKind::IndirectObjectKeyword;
    const DELIMITER: SyntaxKind = SyntaxKind::OpenDictToken;
    const NAME: SyntaxKind = SyntaxKind::NameLiteralToken;
    const DICTIONARY: SyntaxKind = SyntaxKind::DictionaryExpression;
    const KEYWORD_ENDOBJ: SyntaxKind = SyntaxKind::IndirectEndObjectKeyword;
    const COMMENT: SyntaxKind = SyntaxKind::CommentTrivia;
    const SPACE: SyntaxKind = SyntaxKind::WhitespaceTrivia;
    const LINEFEED: SyntaxKind = SyntaxKind::EndOfLineTrivia;

    /// Example of building a PDF object structure with the tree macro.
    ///
    /// The tree preserves all whitespace and trivia, producing the final string:
    /// ```text
    /// 1 0 obj
    /// <<
    ///   /Type /Catalog
    /// >>
    /// endobj
    /// ```
    ///
    /// Each token's trivia is carefully placed:
    /// - Leading trivia appears before the token text
    /// - Trailing trivia appears after the token text
    /// - The `text()` call divides leading from trailing trivia
    #[test]
    fn test_macro() {
        let tree = tree! {
            OBJECT => {
                INDIRECT => {
                    (NUMBER, b"1"),
                    (NUMBER) => {
                        trivia(SPACE, b" "),
                        text(b"0")
                    },
                    (KEYWORD) => {
                        trivia(SPACE, b" "),
                        text(b"obj"),
                        trivia(LINEFEED, b"\n")
                    }
                },
                DICTIONARY => {
                    (DELIMITER) => {
                        text(b"<<"),
                        trivia(LINEFEED, b"\n")
                    },
                    (NAME) => {
                        trivia(SPACE, b" "),
                        trivia(SPACE, b" "),
                        text(b"/Type"),
                        trivia(SPACE, b" ")
                    },
                    (NAME) => {
                        text(b"/Catalog"),
                        trivia(LINEFEED, b"\n")
                    },
                    (DELIMITER) => {
                        text(b">>")
                    }
                },
                (KEYWORD_ENDOBJ) => {
                    text(b"endobj"),
                    trivia(LINEFEED, b"\n"),
                    trivia(COMMENT, b"% This is a comment")
                }
            }
        };

        let expected = b"1 0 obj\n<<\n  /Type /Catalog\n>>endobj\n% This is a comment";
        assert_eq!(tree.to_string(), String::from_utf8_lossy(expected));
        assert_eq!(tree.full_text(), expected);
    }

    /// Example of building a tree with diagnostic decorators.
    ///
    /// Demonstrates using `@diagnostic()` decorators above tokens and nodes.
    /// Multiple diagnostics can be attached to the same element.
    #[test]
    fn test_macro_with_diagnostics() {
        use DiagnosticSeverity::{Error, Info, Warning};
        let tree = tree! {
            OBJECT => {
                @diagnostic(Error, 1, "Unexpected token in object"),
                @diagnostic(Warning, 2, "Missing object number"),
                @diagnostic(Error, 3, "Invalid object syntax"),
                @diagnostic(Error, 4, "Object header malformed"),
                (KEYWORD, b"obj"),
                @diagnostic(Warning, 5, "Dictionary missing key"),
                DICTIONARY => {
                    @diagnostic(Error, 6, "Invalid delimiter"),
                    (DELIMITER, b"<<"),
                    (NAME, b"/Type"),
                    @diagnostic(Info, 7, "Invalid name value"),
                    @diagnostic(Warning, 8, "Type should be specified"),
                    @diagnostic(Error, 9, "Name format incorrect"),
                    (NAME, b"/Catalog"),
                    (DELIMITER, b">>")
                }
            }
        };

        let expected = b"obj<</Type/Catalog>>";
        assert_eq!(tree.full_text(), expected);

        // Verify diagnostics were attached
        assert!(tree.diagnostics().is_none()); // Root doesn't have diagnostics

        // First child (token) should have diagnostics
        if let Some(first_element) = tree.slots().next() {
            match first_element {
                crate::green::node::Slot::Token { token, .. } => {
                    assert!(token.diagnostics().is_some());
                    let diags = token.diagnostics().unwrap();
                    assert_eq!(diags.len(), 4); // 4 diagnostics on first token
                }
                _ => panic!("Expected first element to be a token"),
            }
        }
    }
}
