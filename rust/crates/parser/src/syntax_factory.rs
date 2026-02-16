use lexer::Lexer;
use syntax::GreenPdfDocumentSyntax;

use crate::Parser;

pub fn parse_pdf_document(source: &[u8]) -> GreenPdfDocumentSyntax {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    parser.parse_pdf_document()
}
