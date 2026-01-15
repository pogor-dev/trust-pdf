pub struct Parser<'source> {
    source: &'source [u8],
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source [u8]) -> Self {
        Self { source }
    }
}
