use crate::cursor::syntax_token::SyntaxToken;

#[derive(PartialEq, Eq, Clone, Hash)]
pub(crate) struct SyntaxTrivia {
    token: SyntaxToken,
    is_leading: bool,
}
