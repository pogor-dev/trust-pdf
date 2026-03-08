use crate::{
    FileTrailerSyntax, GreenCst, GreenDiagnostic, GreenExpressionSyntax, GreenListSyntax, GreenNode, GreenNodeElement, GreenNodeSyntax,
    GreenXRefTableExpressionSyntax, SyntaxKind,
};

// TODO: lex the PDF version separately? Might be false positive inside the document
/// Root PDF document structure
/// ISO 32000-2:2020, 7.5 — File structure
#[derive(Clone)]
pub(crate) struct GreenPdfDocumentSyntax(GreenExpressionSyntax);

impl GreenPdfDocumentSyntax {
    pub(crate) fn new(kind: SyntaxKind, elements: GreenNodeElement, diagnostics: Vec<GreenDiagnostic>) -> Self {
        let slots = vec![elements.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenPdfDocumentSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn elements(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenListSyntax::cast(n.clone()),
            _ => None,
        }
    }
}

impl GreenCst for GreenPdfDocumentSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::PdfDocument && node.slot_count() == 1
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenPdfDocumentSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}

/// Document element: collection of objects, xref table, and trailer
/// ISO 32000-2:2020, 7.5 — File structure
#[derive(Clone)]
pub(crate) struct GreenPdfDocumentElementSyntax(GreenExpressionSyntax);

impl GreenPdfDocumentElementSyntax {
    pub(crate) fn new(
        kind: SyntaxKind,
        objects: GreenNodeElement,
        xref_table: GreenNodeElement,
        trailer: GreenNodeElement,
        diagnostics: Vec<GreenDiagnostic>,
    ) -> Self {
        let slots = vec![objects.into(), xref_table.into(), trailer.into()];
        let green = GreenNode::new_with_diagnostic(kind, slots, diagnostics);
        GreenPdfDocumentElementSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub(crate) fn objects(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(0) {
            Some(GreenNodeElement::Node(n)) => GreenListSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn xref_table(&self) -> Option<GreenXRefTableExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenNodeElement::Node(n)) => GreenXRefTableExpressionSyntax::cast(n.clone()),
            _ => None,
        }
    }

    #[inline]
    pub(crate) fn trailer(&self) -> Option<FileTrailerSyntax> {
        match self.0.green().slot(2) {
            Some(GreenNodeElement::Node(n)) => FileTrailerSyntax::cast(n.clone()),
            _ => None,
        }
    }
}

impl GreenCst for GreenPdfDocumentElementSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::PdfDocumentElementExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(GreenPdfDocumentElementSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}
