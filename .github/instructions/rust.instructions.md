---
applyTo: "**/*.rs,**/Cargo.toml"
---

# GitHub Copilot Custom Instructions for PDF Compiler Project

## General Guidelines

- This project implements a PDF compiler in Rust, conforming to the ISO 32000-2 standard.
- The architecture is inspired by Rust Analyzer (RA) and Roslyn, aiming for a balance between performance and extensibility.
- The compiler should support reading, editing, validating, and writing PDF files with full fidelity.

## Parsing and Syntax Analysis

- Utilize a recursive-descent parser to construct a Concrete Syntax Tree (CST) with full fidelity, including all syntactic trivia.
- Incorporate a modified version of `rowan` (e.g., from the Rome project) to attach trivia to syntax tokens, similar to Roslyn's approach.
- Ensure the parser supports incremental parsing to accommodate incremental updates to PDF files.
- Handle PDF-specific syntactic trivia where whitespace and line breaks are semantically significant, such as:
  - `obj` declarations: newline separates header from body (ISO 32000-2 §7.3.10)
  - `stream` keyword: newline required after stream (ISO 32000-2 §7.3.8)
  - `xref` entries: line-based, fixed-width formatting with spaces (ISO 32000-2 §7.5.4)
  - `startxref`: newline separates keyword and offset (ISO 32000-2 §7.5.5)
  - Content streams: space-separated tokens only (ISO 32000-2 §8.1.1)

## Semantic Analysis

- Implement semantic analysis following the Arlington model to validate the semantics of dictionaries and other PDF structures.
- Perform type inference where applicable to ensure correctness and adherence to the PDF specification.
- Decode and analyze PDF streams during the semantic analysis phase, considering memory efficiency.

## Extensibility and API Design

- Design the compiler to be highly extensible and pluggable, akin to .NET source generators, to allow for future analyzers and fixers.
- Provide a rich API to facilitate code generation for each PDF syntax and structure.
- Ensure the architecture supports deployment in various environments, including:
  - WebAssembly (WASM) for web applications
  - .NET via NuGet packages with managed API contracts and unmanaged code for the compiler

## IDE Integration

- Structure the compiler to be IDE-friendly, with considerations for:
  - Language Server Protocol (LSP) implementation
  - Syntax highlighting
  - Code folding (collapse/expand)
  - Diagnostics (e.g., errors, warnings)
  - Analyzers and fixers
  - IntelliSense
  - Hover previews (e.g., displaying images from encoded streams)

## Performance and Dependencies

- Prioritize performance and memory efficiency throughout the compiler's design.
- Aim for near-zero dependencies, incorporating only essential crates such as:
  - `hashbrown` for efficient hash maps
  - `salsa` for incremental computation
  - Modified `rowan` for syntax tree management

## Error Handling

- Implement robust error recovery and resilience mechanisms to handle malformed or corrupted PDF files gracefully.
- Ensure that the compiler can recover from errors and continue processing to the extent possible.

## Documentation Standards

- When adding code documentation, provide comprehensive explanations that are accessible to developers without compiler design or PDF specification experience.
- Include clear descriptions of:
  - What each module, function, or field does in plain language
  - Why specific design decisions were made
  - How components relate to PDF specification sections (with ISO references)
  - Examples of the data structures or operations being handled
  - Context about how the component fits into the overall compiler pipeline
- Use analogies and real-world examples where appropriate to explain complex concepts.
- Document edge cases, error conditions, and their implications for PDF processing.
- When the code changes, ensure that the documentation is updated to reflect the new state of the codebase.

## Future Considerations

- Design the architecture to accommodate a future PDF viewer, enabling visualization and interaction with PDF files.
- Ensure that the compiler's extensibility supports the integration of viewing capabilities without significant restructuring.