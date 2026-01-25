#![recursion_limit = "256"]

mod arc;
mod diagnostic_kind;
mod green;
mod internal;
mod red;
mod syntax_factory;
mod syntax_kind;
mod utility_types;

pub use crate::{
    diagnostic_kind::DiagnosticKind,
    green::{
        DiagnosticSeverity, GreenDiagnostic, GreenDiagnostics, GreenElement, GreenNode, GreenNodeBuilder, GreenNodeData, GreenToken, GreenTokenData,
        GreenTrivia, GreenTriviaData, NodeCache, Slot,
    },
    red::{ChildIterator, SyntaxElement, SyntaxNode, SyntaxToken, SyntaxTrivia},
    syntax_kind::SyntaxKind,
    utility_types::NodeOrTokenOrTrivia,
};
