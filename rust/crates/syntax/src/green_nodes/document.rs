use crate::{GreenNode, SyntaxKind};

// TODO: lex the PDF version separately? Might be false positive inside the document
#[derive(Clone)]
pub struct GreenPdfDocumentSyntax {
    kind: SyntaxKind,
    bodies: GreenNode,
}

pub struct GreenPdfDocumentInnerSyntax {
    kind: SyntaxKind,
    objects: GreenNode,
    xref_table: GreenNode,
    trailer: GreenNode,
}
