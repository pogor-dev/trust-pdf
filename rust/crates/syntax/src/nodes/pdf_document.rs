use rowan::GreenNode;

use crate::SyntaxKind;

// TODO: lex the PDF version separately? Might be false positive inside the document
#[derive(Clone)]
pub struct PdfDocumentSyntax {
    kind: SyntaxKind,
    bodies: PdfDocumentInnerSyntax,
}

#[derive(Clone)]
pub struct PdfDocumentInnerSyntax {
    kind: SyntaxKind,
    objects: GreenNode,
    xref_table: GreenNode,
    trailer: GreenNode,
}
