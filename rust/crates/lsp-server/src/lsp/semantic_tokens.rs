use lsp_types::{SemanticTokenModifier, SemanticTokenType};

pub(crate) const SUPPORTED_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::KEYWORD,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::COMMENT,
];

pub(crate) const SUPPORTED_MODIFIERS: &[SemanticTokenModifier] = &[];
