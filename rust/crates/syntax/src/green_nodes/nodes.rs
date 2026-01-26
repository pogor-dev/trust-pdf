use crate::{GreenNode, SyntaxKind};

// TODO: lex the PDF version separately? Might be false positive inside the document
#[derive(Clone)]
pub struct PdfDocumentSyntax {
    kind: SyntaxKind,
    bodies: GreenNode,
}

#[derive(Clone)]
pub struct PdfDocumentInnerSyntax {
    kind: SyntaxKind,
    objects: GreenNode,
    xref_table: GreenNode,
    trailer: GreenNode,
}
