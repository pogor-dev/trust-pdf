use crate::{
    DiagnosticKind, DiagnosticSeverity, GreenDiagnostic, GreenNode, GreenToken, GreenTokenElement, GreenTokenWithIntValue,
    GreenTokenWithIntValueAndTrailingTrivia, GreenTokenWithIntValueAndTrivia, GreenTokenWithTrailingTrivia, GreenTokenWithTrivia, SyntaxKind,
};

pub(crate) fn make_diagnostic(severity: DiagnosticSeverity, code: DiagnosticKind, message: &str) -> GreenDiagnostic {
    GreenDiagnostic::new(code, severity, message)
}

pub(crate) fn make_expected_token(
    kind: SyntaxKind,
    text: &[u8],
    leading_trivia: Option<GreenNode>,
    trailing_trivia: Option<GreenNode>,
    diagnostics: Vec<GreenDiagnostic>,
) -> GreenTokenElement {
    let has_leading = leading_trivia.is_some();
    let has_trailing = trailing_trivia.is_some();
    let has_diagnostics = !diagnostics.is_empty();
    let is_known_token_kind = kind == SyntaxKind::EndOfFileToken || !kind.get_text().is_empty();

    if is_known_token_kind {
        return match (has_leading, has_trailing, has_diagnostics) {
            (false, false, false) => GreenToken::new(kind).into(),
            (false, false, true) => GreenToken::new_with_diagnostic(kind, diagnostics).into(),
            (false, true, false) => GreenTokenWithTrailingTrivia::new(kind, trailing_trivia).into(),
            (false, true, true) => GreenTokenWithTrailingTrivia::new_with_diagnostic(kind, trailing_trivia, diagnostics).into(),
            (_, _, false) => GreenTokenWithTrivia::new(kind, leading_trivia, trailing_trivia).into(),
            (_, _, true) => GreenTokenWithTrivia::new_with_diagnostic(kind, leading_trivia, trailing_trivia, diagnostics).into(),
        };
    }

    let value = 0u32;
    match (has_leading, has_trailing, has_diagnostics) {
        (false, false, false) => GreenTokenWithIntValue::new(kind, text, value).into(),
        (false, false, true) => GreenTokenWithIntValue::new_with_diagnostic(kind, text, value, diagnostics).into(),
        (false, true, false) => GreenTokenWithIntValueAndTrailingTrivia::new(kind, text, value, trailing_trivia).into(),
        (false, true, true) => GreenTokenWithIntValueAndTrailingTrivia::new_with_diagnostic(kind, text, value, trailing_trivia, diagnostics).into(),
        (_, _, false) => GreenTokenWithIntValueAndTrivia::new(kind, text, value, leading_trivia, trailing_trivia).into(),
        (_, _, true) => {
            GreenTokenWithIntValueAndTrivia::new_with_diagnostic(kind, text, value, leading_trivia, trailing_trivia, diagnostics).into()
        }
    }
}

#[macro_export]
macro_rules! tree {
    ($node_kind:expr => { $($entries:tt)* }) => {{
        let mut slots: Vec<$crate::GreenNodeElement> = Vec::new();
        let mut pending_diagnostics: Vec<$crate::GreenDiagnostic> = Vec::new();
        $crate::tree_entries!(slots, pending_diagnostics; $($entries)*);
        $crate::GreenNode::new($node_kind, slots)
    }};
}

#[macro_export]
macro_rules! tree_entries {
    ($slots:ident, $pending:ident; ) => {};
    ($slots:ident, $pending:ident; , $($rest:tt)*) => {
        $crate::tree_entries!($slots, $pending; $($rest)*);
    };
    ($slots:ident, $pending:ident; @diagnostic($severity:expr, $code:expr, $message:expr) $(, $($rest:tt)*)?) => {{
        let code: $crate::DiagnosticKind = $code;
        $pending.push($crate::green::tree::make_diagnostic($severity, code, $message));
        $crate::tree_entries!($slots, $pending; $($($rest)*)?);
    }};
    ($slots:ident, $pending:ident; ($kind:expr, $text:expr) $(, $($rest:tt)*)?) => {{
        let token = $crate::green::tree::make_expected_token($kind, $text, None, None, std::mem::take(&mut $pending));
        $slots.push($crate::GreenNodeElement::Token(token));
        $crate::tree_entries!($slots, $pending; $($($rest)*)?);
    }};
    ($slots:ident, $pending:ident; ($kind:expr) => { $($token_items:tt)* } $(, $($rest:tt)*)?) => {{
        let mut leading: Vec<$crate::GreenNodeElement> = Vec::new();
        let mut trailing: Vec<$crate::GreenNodeElement> = Vec::new();
        let mut text: &[u8] = b"";
        let mut seen_text = false;
        $crate::tree_token_items!(leading, trailing, text, seen_text; $($token_items)*);

        let leading_node = if leading.is_empty() {
            None
        } else {
            Some($crate::GreenNode::new($crate::SyntaxKind::List, leading))
        };
        let trailing_node = if trailing.is_empty() {
            None
        } else {
            Some($crate::GreenNode::new($crate::SyntaxKind::List, trailing))
        };

        let token = $crate::green::tree::make_expected_token(
            $kind,
            text,
            leading_node,
            trailing_node,
            std::mem::take(&mut $pending),
        );
        $slots.push($crate::GreenNodeElement::Token(token));
        $crate::tree_entries!($slots, $pending; $($($rest)*)?);
    }};
}

#[macro_export]
macro_rules! tree_token_items {
    ($leading:ident, $trailing:ident, $text:ident, $seen_text:ident; ) => {};
    ($leading:ident, $trailing:ident, $text:ident, $seen_text:ident; , $($rest:tt)*) => {
        $crate::tree_token_items!($leading, $trailing, $text, $seen_text; $($rest)*);
    };
    ($leading:ident, $trailing:ident, $text:ident, $seen_text:ident; text($bytes:expr) $(, $($rest:tt)*)?) => {{
        $text = $bytes;
        $seen_text = true;
        $crate::tree_token_items!($leading, $trailing, $text, $seen_text; $($($rest)*)?);
    }};
    ($leading:ident, $trailing:ident, $text:ident, $seen_text:ident; trivia($kind:expr, $bytes:expr) $(, $($rest:tt)*)?) => {{
        let trivia = $crate::GreenTrivia::new($kind, $bytes);
        if $seen_text {
            $trailing.push($crate::GreenNodeElement::Trivia(trivia));
        } else {
            $leading.push($crate::GreenNodeElement::Trivia(trivia));
        }
        $crate::tree_token_items!($leading, $trailing, $text, $seen_text; $($($rest)*)?);
    }};
}

#[cfg(test)]
mod tests {
    use crate::{DiagnosticKind, DiagnosticSeverity, GreenNodeElement, GreenTokenElement, SyntaxKind};

    #[test]
    fn test_tree_when_single_token_expect_token_slot() {
        let node = tree! {
            SyntaxKind::None => {
                (SyntaxKind::NumericLiteralToken, b"42")
            }
        };

        assert_eq!(node.kind(), SyntaxKind::None);
        assert_eq!(node.slot_count(), 1);

        let slot = &node.slots()[0];
        match slot {
            GreenNodeElement::Token(token) => {
                assert_eq!(token.kind(), SyntaxKind::NumericLiteralToken);
                assert_eq!(token.text(), b"42");
            }
            _ => panic!("expected token slot"),
        }
    }

    #[test]
    fn test_tree_when_token_with_diagnostic_expect_token_diagnostic() {
        let node = tree! {
            SyntaxKind::None => {
                @diagnostic(DiagnosticSeverity::Error, DiagnosticKind::UnbalancedHexString, "Unbalanced hex string"),
                (SyntaxKind::HexStringLiteralToken, b"<ABCD")
            }
        };

        let slot = &node.slots()[0];
        match slot {
            GreenNodeElement::Token(token) => {
                let diagnostics = token.diagnostics().expect("diagnostic should be present");
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].kind(), DiagnosticKind::UnbalancedHexString);
                assert_eq!(diagnostics[0].message(), "Unbalanced hex string");
            }
            _ => panic!("expected token slot"),
        }
    }

    #[test]
    fn test_tree_when_token_with_trivia_expect_trivia_preserved() {
        let node = tree! {
            SyntaxKind::None => {
                (SyntaxKind::NumericLiteralToken) => {
                    trivia(SyntaxKind::WhitespaceTrivia, b" "),
                    text(b"12"),
                    trivia(SyntaxKind::EndOfLineTrivia, b"\n")
                }
            }
        };

        let slot = &node.slots()[0];
        match slot {
            GreenNodeElement::Token(token) => {
                assert_eq!(token.kind(), SyntaxKind::NumericLiteralToken);
                assert_eq!(token.text(), b"12");
                assert!(matches!(token, GreenTokenElement::TokenWithIntValueAndTrivia(_)));
                assert!(token.leading_trivia().is_some());
                assert!(token.trailing_trivia().is_some());
            }
            _ => panic!("expected token slot"),
        }
    }
}
