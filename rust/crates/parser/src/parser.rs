use lexer::Lexer;
use syntax::GreenPdfDocumentSyntax;

pub struct Parser<'source> {
    lexer: Lexer<'source>,
}

// TODO: we should return red nodes instead, but as temporary measure we return green nodes
impl<'source> Parser<'source> {
    pub fn new(lexer: Lexer<'source>) -> Self {
        Self { lexer }
    }

    pub fn parse_pdf_document(&mut self) -> GreenPdfDocumentSyntax {
        // Parsing logic goes here
        unreachable!()
    }
}
