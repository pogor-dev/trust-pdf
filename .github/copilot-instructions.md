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
- Rust 1.92.0 (pinned via `rust-toolchain.toml`)
- LLVM tools for coverage: `rustup component add llvm-tools-preview`

## Project-Specific Patterns

### Memory Management & Performance

**Performance and memory efficiency are critical priorities for this project.** The compiler processes potentially large PDF files and must maintain low overhead. All architectural decisions prioritize cache-friendly access and minimal allocations.

Key principles:
- **Arena allocation**: Syntax trees use arena allocation for cache-friendly access and efficient memory layout
- **Inline storage**: `GreenNodeData` stores children inline after the header to reduce indirection
- **Reference counting**: Uses `triomphe::Arc` for shared ownership with minimal overhead
- **Avoid redundant allocations**: Use `Vec::with_capacity()` for pre-sized collections and `vec![]` macro for single-element vectors instead of `Vec::new()` followed by `push()`
- **Stack-based operations**: Prefer stack allocation and iterators over heap allocations when possible
- **Memory pooling**: Cache and reuse frequently allocated structures

Consider performance implications in every change, especially in hot paths like lexing, parsing, and tree traversal.

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
- **Doc comments placement**: Always place `///` doc comments **before** all attributes (e.g., `#[derive]`, `#[repr]`, `#[inline]`)
  - Incorrect: `#[derive(...)]` `#[repr(...)]` `/// docs` `pub struct Foo`
  - Correct: `/// docs` `#[derive(...)]` `#[repr(...)]` `pub struct Foo`

### Testing Conventions
Test naming: `test_<function>_when_<condition>_expect_<result>`
- Cover both spec-compliant and malformed PDF cases
- Test proportional to code complexity
- Avoid testing implementation details

## SafeDocs and PDF Syntax Compliance

**Context**: SafeDocs is a DARPA-funded research program dedicated to improving detection and handling of invalid or maliciously crafted data in electronic documents, including PDF files. A key artifact of this program is the **PDF Compacted Syntax Matrix** and associated test suite.

**What it means**:
- The SafeDocs program produced detailed specification of all 121 possible adjacent PDF token pairings
- This matrix documents which tokens can appear next to each other without whitespace delimiters
- It validates that PDF parsers correctly handle minimal whitespace scenarios (e.g., `/Type/XObject`, `<</A/B>>>>`, `(cat)(sat)`)
- The test suite ensures lexical analyzers comply with ISO 32000-2:2020 clauses 7.2.3 and 7.3.3

**When referencing in comments**:
- Always cite the specific ISO 32000-2:2020 clause (e.g., §7.3.3, §7.3.4.2)
- Example: "ISO 32000-2:2020 clause 7.3.3: Numbers must be separated by token delimiters"
- When validation is based on SafeDocs test matrix insights, reference both the clause AND explain the specific constraint
- This maintains clarity for developers unfamiliar with SafeDocs while acknowledging the specification source

**Reference materials**:
- ISO 32000-2:2020 (official PDF 2.0 specification)
- PDF Association GitHub repository: SafeDocs test matrix and artifacts
- Use the official specification as the primary reference in code comments

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