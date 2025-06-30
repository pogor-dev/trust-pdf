use crate::green::GreenTriviaReprThin;

#[repr(transparent)]
pub(crate) struct GreenTokenData {
    data: GreenTriviaReprThin,
}
