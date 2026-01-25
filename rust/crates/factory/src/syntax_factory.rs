use lexer::Lexer;
use parser::Parser;
use syntax::PdfDocumentSyntax;

pub fn parse_pdf_document(source: &[u8]) -> PdfDocumentSyntax {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    parser.parse_pdf_document()
}
