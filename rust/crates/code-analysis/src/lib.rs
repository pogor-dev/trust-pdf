#![recursion_limit = "256"]

mod arc;
mod diagnostic_kind;
mod green;
mod syntax_kind;

pub use crate::diagnostic_kind::DiagnosticKind;
pub use crate::syntax_kind::SyntaxKind;

pub(crate) use crate::green::{
    DiagnosticSeverity, GreenDiagnostic, GreenDiagnosticData, GreenFlags, GreenNode, GreenNodeData, GreenNodeElement, GreenNodeElementRef, GreenToken,
    GreenTokenData, GreenTokenElement, GreenTokenElementRef, GreenTokenWithFloatValue, GreenTokenWithFloatValueAndTrailingTrivia,
    GreenTokenWithFloatValueAndTrailingTriviaData, GreenTokenWithFloatValueAndTrivia, GreenTokenWithFloatValueAndTriviaData, GreenTokenWithFloatValueData,
    GreenTokenWithIntValue, GreenTokenWithIntValueAndTrailingTrivia, GreenTokenWithIntValueAndTrailingTriviaData, GreenTokenWithIntValueAndTrivia,
    GreenTokenWithIntValueAndTriviaData, GreenTokenWithIntValueData, GreenTokenWithStringValue, GreenTokenWithStringValueAndTrailingTrivia,
    GreenTokenWithStringValueAndTrailingTriviaData, GreenTokenWithStringValueAndTrivia, GreenTokenWithStringValueAndTriviaData, GreenTokenWithStringValueData,
    GreenTokenWithTrailingTrivia, GreenTokenWithTrailingTriviaData, GreenTokenWithTrivia, GreenTokenWithTriviaData, GreenTokenWithValue,
    GreenTokenWithValueAndTrailingTrivia, GreenTokenWithValueAndTrailingTriviaData, GreenTokenWithValueAndTrivia, GreenTokenWithValueAndTriviaData,
    GreenTokenWithValueData, GreenTrivia, GreenTriviaData,
};
