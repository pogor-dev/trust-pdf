/// Represents token text that can either reference static interned data
/// or own dynamic content, similar to C#'s string interning.

#[derive(Debug, Clone)]
pub enum TokenText {
    /// References static interned text (zero allocation)
    Interned(&'static [u8]),
    /// Owns dynamic text content (allocated)
    Owned(Vec<u8>),
}

impl TokenText {
    /// Returns the text content as a slice, regardless of storage type
    pub fn as_slice(&self) -> &[u8] {
        match self {
            TokenText::Interned(slice) => slice,
            TokenText::Owned(vec) => vec.as_slice(),
        }
    }

    /// Converts to owned Vec<u8>, avoiding allocation for interned text when possible
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            TokenText::Interned(slice) => slice.to_vec(),
            TokenText::Owned(vec) => vec.clone(),
        }
    }

    // Returns the length of the text content
    pub fn len(&self) -> usize {
        match self {
            TokenText::Interned(slice) => slice.len(),
            TokenText::Owned(vec) => vec.len(),
        }
    }
}

impl From<Vec<u8>> for TokenText {
    fn from(vec: Vec<u8>) -> Self {
        TokenText::Owned(vec)
    }
}

impl PartialEq for TokenText {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for TokenText {}
