#![recursion_limit = "256"]

mod arc;
mod diagnostic_kind;
mod green;
mod green_nodes;
mod red;
mod syntax_kind;
mod utility_types;

pub use crate::{
    diagnostic_kind::DiagnosticKind,
    green::{
        DiagnosticSeverity, GreenDiagnostic, GreenDiagnostics, GreenElement, GreenNode, GreenNodeBuilder, GreenNodeData, GreenNodeSyntax, GreenToken,
        GreenTokenData, GreenTrait, GreenTrivia, GreenTriviaData, NodeCache, Slot,
    },
    green_nodes::*,
    red::{ChildIterator, SyntaxElement, SyntaxNode, SyntaxToken, SyntaxTrivia},
    syntax_kind::SyntaxKind,
    utility_types::NodeOrTokenOrTrivia,
};
