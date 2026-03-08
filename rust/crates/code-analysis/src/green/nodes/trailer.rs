use crate::{
    GreenCst, GreenDiagnostic, GreenDictionaryExpressionSyntax, GreenExpressionSyntax, GreenNode, GreenNodeElement, GreenNodeSyntax, GreenTokenElement,
    SyntaxKind,
};

/// File trailer: trailer dictionary and startxref byte offset
/// ISO 32000-2:2020, 7.5.5 — File trailer
#[derive(Clone)]
pub(crate) struct FileTrailerSyntax(GreenExpressionSyntax);

impl FileTrailerSyntax {
    pub(crate) fn new(kind: SyntaxKind, trailer_token: GreenNodeElement, body: GreenNodeElement, start_xref: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![trailer_token.into(), body.into(), start_xref.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        FileTrailerSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn trailer_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn body(&self) -> Option<GreenDictionaryExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenDictionaryExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn start_xref(&self) -> Option<FileTrailerStartXrefSyntax> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Node(n)) => FileTrailerStartXrefSyntax::cast(n.clone()),
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
pub(crate) struct FileTrailerStartXrefSyntax(GreenExpressionSyntax);

impl FileTrailerStartXrefSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        start_xref_token: GreenNodeElement,
        xref_offset: GreenNodeElement,
        end_of_file_token: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![start_xref_token.into(), xref_offset.into(), end_of_file_token.into()];

        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        FileTrailerStartXrefSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn start_xref_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn xref_offset(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn end_of_file_token(&self) -> Option<GreenTokenElement> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Token(t)) => Some(t.clone()),
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
