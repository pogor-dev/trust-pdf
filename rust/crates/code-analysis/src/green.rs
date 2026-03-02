#[macro_use]
mod macros;
mod diagnostic;
mod diagnostics;
mod flags;
mod node;
mod node_element;
mod node_type;
mod token;
mod token_element;
mod token_type;
mod token_with_trailing_trivia;
mod token_with_trivia;
mod token_with_value;
mod token_with_value_and_trailing_trivia;
mod token_with_value_and_trivia;
mod trivia;

use std::hash::BuildHasherDefault;
use std::sync::{Mutex, OnceLock};

use rustc_hash::FxHasher;

pub(crate) use self::{
    diagnostic::{DiagnosticSeverity, GreenDiagnostic, GreenDiagnosticData},
    diagnostics::{GreenDiagnostics, GreenDiagnosticsData},
    flags::GreenFlags,
    node::{GreenNode, GreenNodeData},
    node_element::{GreenNodeElement, GreenNodeElementRef},
    node_type::NodeOrTokenOrTrivia,
    token::{GreenToken, GreenTokenData},
    token_element::{GreenTokenElement, GreenTokenElementRef},
    token_type::TokenType,
    token_with_trailing_trivia::{GreenTokenWithTrailingTrivia, GreenTokenWithTrailingTriviaData},
    token_with_trivia::{GreenTokenWithTrivia, GreenTokenWithTriviaData},
    token_with_value::{
        GreenTokenWithFloatValue, GreenTokenWithFloatValueData, GreenTokenWithIntValue, GreenTokenWithIntValueData, GreenTokenWithStringValue,
        GreenTokenWithStringValueData, GreenTokenWithValue, GreenTokenWithValueData,
    },
    token_with_value_and_trailing_trivia::{
        GreenTokenWithFloatValueAndTrailingTrivia, GreenTokenWithFloatValueAndTrailingTriviaData, GreenTokenWithIntValueAndTrailingTrivia,
        GreenTokenWithIntValueAndTrailingTriviaData, GreenTokenWithStringValueAndTrailingTrivia, GreenTokenWithStringValueAndTrailingTriviaData,
        GreenTokenWithValueAndTrailingTrivia, GreenTokenWithValueAndTrailingTriviaData,
    },
    token_with_value_and_trivia::{
        GreenTokenWithFloatValueAndTrivia, GreenTokenWithFloatValueAndTriviaData, GreenTokenWithIntValueAndTrivia, GreenTokenWithIntValueAndTriviaData,
        GreenTokenWithStringValueAndTrivia, GreenTokenWithStringValueAndTriviaData, GreenTokenWithValueAndTrivia, GreenTokenWithValueAndTriviaData,
    },
    trivia::{GreenTrivia, GreenTriviaData},
};

type HashMap<K, V> = hashbrown::HashMap<K, V, BuildHasherDefault<FxHasher>>;
type GreenDiagnosticTable = HashMap<usize, GreenDiagnostics>;

pub(crate) fn green_diagnostics_table() -> &'static Mutex<GreenDiagnosticTable> {
    static TABLE: OnceLock<Mutex<GreenDiagnosticTable>> = OnceLock::new();
    TABLE.get_or_init(|| Mutex::new(HashMap::default()))
}
