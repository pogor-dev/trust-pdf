use lexer::Lexer;
use parser::Parser;
use syntax::GreenPdfDocumentSyntax;

pub fn parse_pdf_document(source: &[u8]) -> GreenPdfDocumentSyntax {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    parser.parse_pdf_document()
}
