# GitHub Copilot Instructions for trust-pdf

## Project Overview

This is a Rust-based PDF compiler implementing ISO 32000-2, following Rust Analyzer/Roslyn architecture patterns. The codebase is structured as a multi-phase compiler: `lexer` → `parser` → `syntax` → `semantic`, where each phase builds upon the previous one to process PDF files with full fidelity.

## Architecture & Component Boundaries

The project uses a **workspace-based crate structure** in `rust/crates/`:
- **lexer**: Tokenization of PDF input
- **syntax**: Concrete Syntax Tree (CST) with full trivia preservation 
- **parser**: Recursive-descent parser building CSTs
- **semantic**: High-level Intermediate Representation (HIR) with PDF-specific semantics

Key architectural decisions:
- Uses arena-allocated syntax trees (`la-arena`) for memory efficiency
- Green nodes (`GreenNode`) store immutable syntax tree data with inline children
- CST preserves all whitespace/trivia for full-fidelity PDF reconstruction
- HIR uses typed arenas (`ExprId`, `CatalogId`, etc.) for semantic analysis

## Development Workflows

### Testing & Coverage
```bash
# Run tests with coverage (HTML output)
cargo llvm-cov --ignore-filename-regex=".*_tests.rs" --html --open

# Generate LCOV format for CI
cargo llvm-cov --lcov --output-path target/lcov.info
```

### Toolchain Requirements
- Rust 1.90.0 (pinned via `rust-toolchain.toml`)
- LLVM tools for coverage: `rustup component add llvm-tools-preview`

## Project-Specific Patterns

### Memory Management
- **Arena allocation**: Syntax trees use arena allocation for cache-friendly access
- **Inline storage**: `GreenNodeData` stores children inline after the header
- **Reference counting**: Uses `triomphe::Arc` for shared ownership

### Error Handling & Resilience
- **Error recovery**: Parser continues processing after errors for IDE-like experience
- **Malformed PDF handling**: Gracefully handle corrupted/non-spec-compliant files
- **Incremental parsing**: Architecture supports incremental updates for large PDFs

### PDF-Specific Semantics
PDF syntax has strict whitespace rules that must be preserved:
- `obj` declarations: newline separates header from body
- `stream` keyword: requires newline after keyword
- `xref` entries: line-based, fixed-width formatting
- Content streams: space-separated tokens only

### Module Organization
- Place most important structs first in files
- Group struct implementations together
- Use `lib.rs` not `mod.rs` for modules
- Public API exposed through selective re-exports

### Documentation Standards
- Focus on **why** not **how** in code comments
- Explain PDF specification context for non-obvious code
- Document edge cases and error conditions
- Use analogies for complex compiler concepts
- Keep documentation concise (few lines per item)

### Testing Conventions
Test naming: `test_<function>_when_<condition>_expect_<result>`
- Cover both spec-compliant and malformed PDF cases
- Test proportional to code complexity
- Avoid testing implementation details

## Critical Dependencies

Minimal dependency philosophy - only essential crates:
- `hashbrown`: Efficient hash maps
- `la-arena`: Arena allocation for syntax trees  
- `countme`: Memory usage tracking
- `triomphe`: Reference-counted pointers
- `bumpalo`: Bump allocation for temporary data

## IDE Integration Considerations

The compiler is designed for rich IDE experiences:
- **LSP support**: Architecture enables language server implementation
- **Incremental compilation**: Salsa-ready for incremental computation
- **Rich diagnostics**: Full error recovery with precise error locations
- **Semantic analysis**: Type inference and PDF structure validation via Arlington model

## Future Extensibility

- **WebAssembly target**: Architecture supports WASM compilation
- **Plugin system**: Extensible analyzer/fixer framework planned
- **Viewer integration**: Compiler designed to support future PDF viewer
- **.NET interop**: Planned managed API via C ABI