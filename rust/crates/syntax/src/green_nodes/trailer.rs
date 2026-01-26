use crate::{
    GreenCst, GreenDiagnostics, GreenDictionaryExpressionSyntax, GreenElement, GreenExpressionSyntax, GreenNode, GreenNodeSyntax, GreenToken, GreenTrait,
    SyntaxKind,
};

#[derive(Clone)]
pub struct FileTrailerExpressionSyntax(GreenExpressionSyntax);

impl FileTrailerExpressionSyntax {
    pub fn new(kind: SyntaxKind, trailer_token: GreenToken, body: GreenNode, start_xref: GreenNode, diagnostics: Option<GreenDiagnostics>) -> Self {
        let slots = vec![GreenElement::Token(trailer_token), GreenElement::Node(body), GreenElement::Node(start_xref)];

        let green = GreenNode::new(kind, slots, diagnostics);
        FileTrailerExpressionSyntax(GreenExpressionSyntax(green))
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
    pub fn start_xref(&self) -> Option<FileTrailerStartXrefExpressionSyntax> {
        match self.0.green().slot(2) {
            Some(GreenElement::Node(n)) => FileTrailerStartXrefExpressionSyntax::cast(n),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub struct FileTrailerStartXrefExpressionSyntax(GreenExpressionSyntax);

impl FileTrailerStartXrefExpressionSyntax {
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
        FileTrailerStartXrefExpressionSyntax(GreenExpressionSyntax(green))
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

impl GreenCst for FileTrailerStartXrefExpressionSyntax {
    #[inline]
    fn can_cast(node: &GreenNode) -> bool {
        node.kind() == SyntaxKind::FileTrailerStartXrefExpression && node.slot_count() == 3
    }

    #[inline]
    fn cast(node: GreenNode) -> Option<Self> {
        match Self::can_cast(&node) {
            true => Some(FileTrailerStartXrefExpressionSyntax(GreenExpressionSyntax(node))),
            false => None,
        }
    }
}
