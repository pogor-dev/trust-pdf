use crate::{
    GreenCst, GreenDiagnostics, GreenDictionaryExpressionSyntax, GreenElement, GreenExpressionSyntax, GreenNode, GreenNodeSyntax, GreenToken, GreenTrait,
    SyntaxKind,
};

/// File trailer: trailer dictionary and startxref byte offset
/// ISO 32000-2:2020, 7.5.5 — File trailer
#[derive(Clone)]
pub struct FileTrailerSyntax(GreenExpressionSyntax);

impl FileTrailerSyntax {
    pub fn new(kind: SyntaxKind, trailer_token: GreenToken, body: GreenNode, start_xref: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Token(trailer_token), GreenElement::Node(body), GreenElement::Node(start_xref)];

        let green = GreenNode::new(kind, slots, diagnostics);
        FileTrailerSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn trailer_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(0) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn body(&self) -> Option<GreenDictionaryExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenDictionaryExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn start_xref(&self) -> Option<FileTrailerStartXrefSyntax> {
        match self.0.green().slot(2) {
            Some(GreenElement::Node(n)) => FileTrailerStartXrefSyntax::cast(n),
            _ => None,
        }
    }
}

impl GreenCst for FileTrailerSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::FileTrailerExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(FileTrailerSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

/// startxref byte offset: startxref <byte-offset> %%EOF
/// ISO 32000-2:2020, 7.5.5 — File trailer
#[derive(Clone)]
pub struct FileTrailerStartXrefSyntax(GreenExpressionSyntax);

impl FileTrailerStartXrefSyntax {
    pub fn new(
        kind: SyntaxKind,
        start_xref_token: GreenToken,
        xref_offset: GreenToken,
        end_of_file_token: GreenToken,
        diagnostics: Option<GreenDiagnostics>,
    ) -> Self {
        let slots = vec![
            GreenElement::Token(start_xref_token),
            GreenElement::Token(xref_offset),
            GreenElement::Token(end_of_file_token),
        ];

        let green = GreenNode::new(kind, slots, diagnostics);
        FileTrailerStartXrefSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn start_xref_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(0) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn xref_offset(&self) -> Option<GreenToken> {
        match self.0.green().slot(1) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }

    #[inline]
    pub fn end_of_file_token(&self) -> Option<GreenToken> {
        match self.0.green().slot(2) {
            Some(GreenElement::Token(t)) => Some(t),
            _ => None,
        }
    }
}

impl GreenCst for FileTrailerStartXrefSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::FileTrailerStartXrefExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(FileTrailerStartXrefSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}
