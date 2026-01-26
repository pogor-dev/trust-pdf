use crate::{
    FileTrailerExpressionSyntax, GreenCst, GreenDiagnostics, GreenElement, GreenExpressionSyntax, GreenListSyntax, GreenNode, GreenNodeSyntax, GreenTrait,
    GreenXRefTableExpressionSyntax, SyntaxKind,
};

// TODO: lex the PDF version separately? Might be false positive inside the document
pub struct GreenPdfDocumentSyntax(GreenExpressionSyntax);

impl GreenPdfDocumentSyntax {
    pub fn new(kind: SyntaxKind, elements: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Node(elements)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenPdfDocumentSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn elements(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenListSyntax::cast(n),
            _ => None,
        }
    }
}

pub struct GreenPdfDocumentElementSyntax(GreenExpressionSyntax);

impl GreenPdfDocumentElementSyntax {
    pub fn new(kind: SyntaxKind, objects: GreenNode, xref_table: GreenNode, trailer: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Node(objects), GreenElement::Node(xref_table), GreenElement::Node(trailer)];
        let green = GreenNode::new(kind, slots, diagnostics);
        GreenPdfDocumentElementSyntax(GreenExpressionSyntax(green))
    }

    #[inline]
    pub fn objects(&self) -> Option<GreenListSyntax> {
        match self.0.green().slot(0) {
            Some(GreenElement::Node(n)) => GreenListSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn xref_table(&self) -> Option<GreenXRefTableExpressionSyntax> {
        match self.0.green().slot(1) {
            Some(GreenElement::Node(n)) => GreenXRefTableExpressionSyntax::cast(n),
            _ => None,
        }
    }

    #[inline]
    pub fn trailer(&self) -> Option<FileTrailerExpressionSyntax> {
        match self.0.green().slot(2) {
            Some(GreenElement::Node(n)) => FileTrailerExpressionSyntax::cast(n),
            _ => None,
        }
    }
}
