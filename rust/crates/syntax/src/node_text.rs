use std::{fmt, ops::Range};

use crate::{
    byte_to_string,
    red::{SyntaxNode, SyntaxToken},
};

#[derive(Clone)]
pub struct SyntaxText {
    node: SyntaxNode,
    range: Range<u64>,
}

impl SyntaxText {
    pub(crate) fn new(node: SyntaxNode) -> SyntaxText {
        let range = node.text_range();
        SyntaxText { node, range }
    }

    pub fn try_for_each_chunk<F: FnMut(&[u8]) -> Result<(), E>, E>(&self, mut f: F) -> Result<(), E> {
        self.try_fold_chunks((), move |(), chunk| f(chunk))
    }

    pub fn try_fold_chunks<T, F, E>(&self, init: T, mut f: F) -> Result<T, E>
    where
        F: FnMut(T, &[u8]) -> Result<T, E>,
    {
        self.tokens_with_ranges().try_fold(init, move |acc, (token, range)| {
            let usize_range = (range.start as usize)..(range.end as usize);
            f(acc, &token.full_text()[usize_range])
        })
    }

    fn tokens_with_ranges(&self) -> impl Iterator<Item = (SyntaxToken, Range<u64>)> + use<> {
        let text_range = self.range.clone();
        self.node
            .descendants_with_tokens()
            .filter_map(|element| element.into_token())
            .filter_map(move |token| {
                let token_range = token.text_range();
                let start = std::cmp::max(text_range.start, token_range.start);
                let end = std::cmp::min(text_range.end, token_range.end);
                if start >= end {
                    return None;
                }

                let range = (start - token_range.start)..(end - token_range.start);
                Some((token, range))
            })
    }

    pub fn full_text(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        for (token, range) in self.tokens_with_ranges() {
            let chunk = &token.full_text()[range.start as usize..range.end as usize];
            bytes.extend_from_slice(chunk);
        }
        bytes
    }
}

impl Eq for SyntaxText {}

impl PartialEq for SyntaxText {
    fn eq(&self, other: &SyntaxText) -> bool {
        if self.range.end - self.range.start != other.range.end - other.range.start {
            return false;
        }
        let mut lhs = self.tokens_with_ranges();
        let mut rhs = other.tokens_with_ranges();
        zip_texts(&mut lhs, &mut rhs).is_none() && lhs.all(|it| it.1.is_empty()) && rhs.all(|it| it.1.is_empty())
    }
}

impl fmt::Debug for SyntaxText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.to_string(), f)
    }
}

impl fmt::Display for SyntaxText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.try_for_each_chunk(|chunk| write!(f, "{}", byte_to_string(chunk)))
    }
}

impl From<SyntaxText> for Vec<u8> {
    fn from(text: SyntaxText) -> Vec<u8> {
        text.full_text()
    }
}

impl PartialEq<[u8]> for SyntaxText {
    fn eq(&self, mut rhs: &[u8]) -> bool {
        self.try_for_each_chunk(|chunk| {
            if !rhs.starts_with(chunk) {
                return Err(());
            }
            rhs = &rhs[chunk.len()..];
            Ok(())
        })
        .is_ok()
            && rhs.is_empty()
    }
}

impl PartialEq<SyntaxText> for [u8] {
    fn eq(&self, rhs: &SyntaxText) -> bool {
        rhs == self
    }
}

impl PartialEq<&'_ [u8]> for SyntaxText {
    fn eq(&self, rhs: &&[u8]) -> bool {
        self == rhs
    }
}

impl PartialEq<SyntaxText> for &'_ [u8] {
    fn eq(&self, rhs: &SyntaxText) -> bool {
        rhs == self
    }
}

fn zip_texts<I: Iterator<Item = (SyntaxToken, Range<u64>)>>(xs: &mut I, ys: &mut I) -> Option<()> {
    let mut x = xs.next()?;
    let mut y = ys.next()?;
    loop {
        while x.1.is_empty() {
            x = xs.next()?;
        }

        while y.1.is_empty() {
            y = ys.next()?;
        }

        let x_full_text = x.0.full_text();
        let y_full_text = y.0.full_text();
        let x_text = &x_full_text.as_slice()[x.1.start as usize..x.1.end as usize];
        let y_text = &y_full_text.as_slice()[y.1.start as usize..y.1.end as usize];

        if !(x_text.starts_with(y_text) || y_text.starts_with(x_text)) {
            return Some(());
        }

        let advance = std::cmp::min(x.1.end - x.1.start, y.1.end - y.1.start);
        x.1 = (x.1.start + advance)..(x.1.end);
        y.1 = (y.1.start + advance)..(y.1.end);
    }
}
