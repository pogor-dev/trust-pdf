pub struct Lexer<'source> {
    source: &'source [u8],
    position: usize,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source [u8]) -> Self {
        Self { source, position: 0 }
    }

    pub fn next_token(&mut self) -> Option<&'source [u8]> {
        None
    }
}
