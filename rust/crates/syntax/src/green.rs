mod builder;
mod cache;
mod diagnostic;
mod diagnostics;
mod element;
mod flags;
#[path = "./green/trait.rs"]
mod green_trait;
mod macros;
mod node;
mod token;
mod trivia;

pub use self::element::GreenElement;

pub(crate) use self::element::GreenElementRef;

// TODO: consider adapting Roslyn style so green nodes are not public, and we only expose the syntax structs that wrap them.

pub use self::{
    builder::GreenNodeBuilder,
    cache::NodeCache,
    diagnostic::{DiagnosticSeverity, GreenDiagnostic},
    diagnostics::GreenDiagnostics,
    green_trait::{GreenCst, GreenNodeSyntax, GreenTrait},
    node::{GreenNode, GreenNodeData, Slot, Slots},
    token::{GreenToken, GreenTokenData},
    trivia::{GreenTrivia, GreenTriviaData},
};
