use crate::SyntaxKind;

pub trait GreenNode {
    fn kind(&self) -> SyntaxKind;

    // fn to_string(&self) -> Cow<'a, [u8]>;

    // fn to_full_string(&self) -> Cow<'a, [u8]>;

    // fn width(&self) -> usize;

    // fn full_width(&self) -> usize;

    // fn slot(&self, _index: usize) -> Option<&Self::GreenNodeType>;

    // fn slot_count(&self) -> usize;

    // fn is_token(&self) -> bool;

    // fn is_trivia(&self) -> bool;

    // fn is_list(&self) -> bool;

    // fn leading_trivia(&self) -> Option<&Self::GreenNodeType>;

    // fn trailing_trivia(&self) -> Option<&Self::GreenNodeType>;

    // fn leading_trivia_width(&self) -> usize;

    // fn trailing_trivia_width(&self) -> usize;

    // fn has_leading_trivia(&self) -> bool;

    // fn has_trailing_trivia(&self) -> bool;

    // fn write_token_to<W: io::Write>(&self, _writer: &mut W, _leading: bool, _trailing: bool) -> io::Result<()>;
    // fn write_trivia_to<W: io::Write>(&self, _writer: &mut W) -> io::Result<()>;

    // fn write_to<W: io::Write>(&self, writer: &mut W, leading: bool, trailing: bool) -> io::Result<()>;

    // fn get_first_non_null_child_index(node: &Self) -> usize;

    // fn get_last_non_null_child_index(node: &Self) -> usize;

    // // Default implementations for terminal finding
    // fn get_first_terminal(&self) -> Option<&Self::GreenNodeType>;
    // fn get_last_terminal(&self) -> Option<&Self::GreenNodeType>;
}
