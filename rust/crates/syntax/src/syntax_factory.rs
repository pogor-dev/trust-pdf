// TODO: lexer/parser crates are using syntax crate, while syntax_factory is using parser crate.
// This creates a cyclic dependency. We should refactor the code to break this cycle.

pub fn parse_pdf_document(source: &[u8]) -> PdfDocumentSyntax {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    parser.parse_pdf_document()
}
