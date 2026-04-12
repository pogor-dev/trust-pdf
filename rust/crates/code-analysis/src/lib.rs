#![recursion_limit = "256"]
#![allow(unused_imports)]

mod arc;
mod diagnostic_kind;
mod lexer;
mod parser;
mod syntax;
mod syntax_kind;

pub use crate::diagnostic_kind::DiagnosticKind;
pub use crate::syntax_kind::SyntaxKind;

pub(crate) use crate::{
    lexer::Lexer,
    parser::Parser,
    syntax::{
        DiagnosticSeverity, FileTrailerStartXrefSyntax, FileTrailerSyntax, GreenArrayElementExpressionSyntax, GreenArrayExpressionSyntax,
        GreenCompatibilityExpressionSyntax, GreenCst, GreenDiagnostic, GreenDiagnosticData, GreenDictionaryElementSyntax, GreenDictionaryExpressionSyntax,
        GreenDirectObjectExpressionSyntax, GreenDirectObjectOrIndirectReferenceExpressionSyntax, GreenExpressionSyntax, GreenFlags,
        GreenIndirectBodyExpressionSyntax, GreenIndirectObjectHeaderExpressionSyntax, GreenIndirectReferenceExpressionSyntax, GreenInlineImageSyntax,
        GreenListSyntax, GreenLiteralExpressionSyntax, GreenMarkedContentSyntax, GreenNode, GreenNodeData, GreenNodeElement, GreenNodeElementRef,
        GreenNodeSyntax, GreenPdfDocumentElementSyntax, GreenPdfDocumentSyntax, GreenPdfVersionSyntax, GreenStreamBodySyntax, GreenStreamExpressionSyntax,
        GreenStreamOperatorOperandExpressionSyntax, GreenStreamRawDataSyntax, GreenSyntaxFactory, GreenTextObjectSyntax, GreenToken, GreenTokenData,
        GreenTokenElement, GreenTokenElementRef, GreenTokenWithFloatValue, GreenTokenWithFloatValueAndTrailingTrivia,
        GreenTokenWithFloatValueAndTrailingTriviaData, GreenTokenWithFloatValueAndTrivia, GreenTokenWithFloatValueAndTriviaData, GreenTokenWithFloatValueData,
        GreenTokenWithIntValue, GreenTokenWithIntValueAndTrailingTrivia, GreenTokenWithIntValueAndTrailingTriviaData, GreenTokenWithIntValueAndTrivia,
        GreenTokenWithIntValueAndTriviaData, GreenTokenWithIntValueData, GreenTokenWithStringValue, GreenTokenWithStringValueAndTrailingTrivia,
        GreenTokenWithStringValueAndTrailingTriviaData, GreenTokenWithStringValueAndTrivia, GreenTokenWithStringValueAndTriviaData,
        GreenTokenWithStringValueData, GreenTokenWithTrailingTrivia, GreenTokenWithTrailingTriviaData, GreenTokenWithTrivia, GreenTokenWithTriviaData,
        GreenTokenWithValue, GreenTokenWithValueAndTrailingTrivia, GreenTokenWithValueAndTrailingTriviaData, GreenTokenWithValueAndTrivia,
        GreenTokenWithValueAndTriviaData, GreenTokenWithValueData, GreenTrait, GreenTrivia, GreenTriviaData, GreenXRefEntryExpressionSyntax,
        GreenXRefSectionSyntax, GreenXRefSubSectionSyntax, GreenXRefTableExpressionSyntax,
    },
};

pub use crate::syntax::{SyntaxNode, SyntaxToken, SyntaxTokenValueRef, SyntaxTrivia};
