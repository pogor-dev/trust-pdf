/// Macro for building green syntax trees with a declarative syntax.
///
/// # Expansion Steps
///
/// The macro expands in the following order:
///
/// 1. **Entry point**: `tree! { ... }` creates a `GreenNodeBuilder`
/// 2. **Elements processing**: Recursively processes nodes and tokens via `@elements`
/// 3. **Node expansion**: `KIND => { children }` calls `start_node`, recurses on children, then `finish_node`
/// 4. **Token expansion**: `(KIND) => { content }` calls `start_token`, expands content, then `finish_token`
/// 5. **Token content**: Comma-separated function calls are converted to semicolon-separated statements
///
/// # Example Expansion
///
/// Input:
/// ```ignore
/// tree! {
///     PARENT => {
///         (TOKEN) => { space(), text(b"foo") },
///         CHILD => {
///             (TOKEN) => { text(b"bar") }
///         }
///     }
/// }
/// ```
///
/// Expands to:
/// ```ignore
/// {
///     let mut builder = GreenNodeBuilder::new();
///     
///     builder.start_node(PARENT);           // Start PARENT node
///     
///         builder.start_token(TOKEN);       // Start first TOKEN
///         { space(); text(b"foo"); }        // Execute token content
///         builder.finish_token();           // Finish first TOKEN
///         
///         builder.start_node(CHILD);        // Start CHILD node
///         
///             builder.start_token(TOKEN);   // Start nested TOKEN
///             { text(b"bar"); }             // Execute token content
///             builder.finish_token();       // Finish nested TOKEN
///             
///         builder.finish_node();            // Finish CHILD node
///         
///     builder.finish_node();                // Finish PARENT node
///     
///     builder.finish()                      // Return root GreenNode
/// }
/// ```
///
/// # Syntax Rules
///
/// - **Node**: `KIND => { children }` - no parentheses around KIND
/// - **Token**: `(KIND) => { content }` - parentheses around KIND
/// - **Token content**: Comma-separated calls to `text()`, `space()`, `linefeed()`, etc.
#[macro_export]
macro_rules! tree {
    // Public entry point: delegates to internal implementation
    ($($tt:tt)*) => {
        $crate::__tree_internal!($($tt)*)
    };
}

/// Internal implementation of the tree macro.
/// Not intended for direct use - use `tree!` instead.
#[doc(hidden)]
#[macro_export]
macro_rules! __tree_internal {
    // [Step 1a] Token expansion (last token in sequence)
    // Matches: (KIND) => { content }
    (@elements $builder:ident, ($kind:expr) => { $($content:tt)* }) => {{
        // TODO: Implement start_token/finish_token in GreenNodeBuilder
        $builder.start_token($kind);
        { $crate::__tree_internal!(@token_content $($content)*) }
        $builder.finish_token();
    }};

    // [Step 1b] Token expansion (followed by more elements)
    // Matches: (KIND) => { content }, rest...
    (@elements $builder:ident, ($kind:expr) => { $($content:tt)* }, $($rest:tt)*) => {{
        // TODO: Implement start_token/finish_token in GreenNodeBuilder
        $builder.start_token($kind);
        { $crate::__tree_internal!(@token_content $($content)*) }
        $builder.finish_token();
        $crate::__tree_internal!(@elements $builder, $($rest)*);
    }};

    // [Step 1c] Token shorthand (last token in sequence)
    // Matches: (KIND, text)
    (@elements $builder:ident, ($kind:expr, $text:expr)) => {{
        // TODO: Implement start_token/finish_token in GreenNodeBuilder
        $builder.start_token($kind);
        { text($text); }
        $builder.finish_token();
    }};

    // [Step 1d] Token shorthand (followed by more elements)
    // Matches: (KIND, text), rest...
    (@elements $builder:ident, ($kind:expr, $text:expr), $($rest:tt)*) => {{
        // TODO: Implement start_token/finish_token in GreenNodeBuilder
        $builder.start_token($kind);
        { text($text); }
        $builder.finish_token();
        $crate::__tree_internal!(@elements $builder, $($rest)*);
    }};

    // [Step 2a] Node expansion (last node in sequence)
    // Matches: KIND => { children }
    (@elements $builder:ident, $kind:expr => { $($children:tt)* }) => {{
        $builder.start_node($kind);
        $crate::__tree_internal!(@elements $builder, $($children)*);
        $builder.finish_node();
    }};

    // [Step 2b] Node expansion (followed by more elements)
    // Matches: KIND => { children }, rest...
    (@elements $builder:ident, $kind:expr => { $($children:tt)* }, $($rest:tt)*) => {{
        $builder.start_node($kind);
        $crate::__tree_internal!(@elements $builder, $($children)*);
        $builder.finish_node();
        $crate::__tree_internal!(@elements $builder, $($rest)*);
    }};

    // [Step 3] Helper: converts comma-separated token content to semicolon-separated statements
    // Input: space(), text(b"foo"), linefeed()
    // Output: { space(); text(b"foo"); linefeed(); }
    (@token_content $($item:expr),* $(,)?) => {{
        $( $item; )*
    }};

    // [Step 4] Base case: recursion ends when all elements are processed
    (@elements $builder:ident,) => {};

    // [Step 0] Entry point: creates builder, passes to @elements, returns finished tree
    // Must be last since ($($tt:tt)*) matches everything
    ($($tt:tt)*) => {{
        let mut builder = $crate::green::builder::GreenNodeBuilder::new();
        $crate::__tree_internal!(@elements builder, $($tt)*);
        builder.finish()
    }};
}

#[cfg(test)]
mod builder_tests {
    use rstest::rstest;

    use crate::SyntaxKind;

    // TODO: Implement these helper functions for token building
    #[allow(dead_code)]
    fn space() {
        // TODO: Add space trivia
    }

    #[allow(dead_code)]
    fn linefeed() {
        // TODO: Add linefeed trivia
    }

    #[allow(dead_code)]
    fn carriage_return() {
        // TODO: Add carriage return trivia
    }

    #[allow(dead_code)]
    fn text(_value: &[u8]) {
        // TODO: Set token text
    }

    #[allow(dead_code)]
    fn trivia(_kind: crate::SyntaxKind, _value: &[u8]) {
        // TODO: Add generic trivia with kind and value
    }

    const OBJECT: SyntaxKind = SyntaxKind(1000);
    const INDIRECT: SyntaxKind = SyntaxKind(1001);
    const NUMBER: SyntaxKind = SyntaxKind(1002);
    const KEYWORD: SyntaxKind = SyntaxKind(1003);
    const DELIMITER: SyntaxKind = SyntaxKind(1004);
    const NAME: SyntaxKind = SyntaxKind(1005);
    const DICTIONARY: SyntaxKind = SyntaxKind(1006);
    const KEYWORD_ENDOBJ: SyntaxKind = SyntaxKind(1007);
    const COMMENT: SyntaxKind = SyntaxKind(2000);

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
    #[rstest]
    fn test_macro() {
        let _tree = tree! {
            OBJECT => {
                INDIRECT => {
                    (NUMBER, b"1"),
                    (NUMBER) => {
                        space(),
                        text(b"0")
                    },
                    (KEYWORD) => {
                        space(),
                        text(b"obj"),
                        linefeed()
                    }
                },
                DICTIONARY => {
                    (DELIMITER) => {
                        linefeed(),
                        text(b"<<"),
                        linefeed()
                    },
                    (NAME) => {
                        space(),
                        space(),
                        text(b"/Type"),
                        space()
                    },
                    (NAME) => {
                        text(b"/Catalog"),
                        linefeed()
                    },
                    (DELIMITER) => {
                        text(b">>"),
                        linefeed()
                    }
                },
                (KEYWORD_ENDOBJ) => {
                    text(b"endobj"),
                    linefeed(),
                    trivia(COMMENT, b"% This is a comment")
                }
            }
        };
    }
}
