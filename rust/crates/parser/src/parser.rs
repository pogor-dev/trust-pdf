use lexer::Lexer;
use syntax::PdfDocumentSyntax;

pub struct Parser<'source> {
    lexer: Lexer<'source>,
}

impl<'source> Parser<'source> {
    pub fn new(lexer: Lexer<'source>) -> Self {
        Self { lexer }
    }

    pub fn parse_pdf_document(&mut self) -> PdfDocumentSyntax {
        // Parsing logic goes here
        unreachable!()
    }
}
