---
applyTo: "**/*.rs,**/Cargo.toml"
---

# Rust-Specific Implementation Instructions

## Rust Project Guidelines

- Follow Rust 2024 edition conventions and idioms
- Maintain compatibility with Rust 1.90.0 as specified in `rust-toolchain.toml`
- Prioritize zero-cost abstractions and memory safety

## Syntax Tree Implementation

- Use `GreenNode` for immutable syntax tree storage with inline children allocation
- Implement `repr(C)` structs for memory layout control in syntax nodes
- Preserve all trivia (whitespace, comments) in CST for full-fidelity reconstruction
- Use `countme::Count` for memory usage tracking in data structures
- Follow the pattern: `GreenNodeHead` + inline `GreenChild` array for efficient storage

## Semantic Analysis & HIR

- Use typed arena indices (`ExprId`, `CatalogId`, `PageTreeId`) for HIR references
- Implement `PartialEq` for `Literal` enum with special handling for NaN floats
- Structure HIR as nested enums: `Expr::Literal`, `Expr::Array`, `Expr::Dictionary`, etc.
- Use `Box<[T]>` for owned slices in HIR nodes for memory efficiency

## Rust-Specific API Design

- Use newtype patterns for strongly typed identifiers (`SyntaxKind(u16)`)
- Implement `Display`, `Debug`, and common traits for public types
- Use `inline` attributes for performance-critical accessors in hot paths
- Design APIs with `NonNull` pointers for unsafe optimizations when needed
- Structure crate boundaries with clear `lib.rs` re-exports

## Performance and Dependencies

- Prioritize performance and memory efficiency throughout the compiler's design
- Follow minimal dependency philosophy - only add essential crates from `workspace.dependencies`
- Use arena allocation patterns (`la-arena`) for syntax trees and semantic data structures
- Prefer `triomphe::Arc` over `std::sync::Arc` for reference counting

## Error Handling

- Implement robust error recovery and resilience mechanisms to handle malformed or corrupted PDF files gracefully.
- Ensure that the compiler can recover from errors and continue processing to the extent possible.

## Debugging Macro Expansion Issues

When encountering recursion limit errors or other issues with macro expansions:
- Use `cargo-expand` to inspect the expanded macro output and diagnose the problem
- Install with: `cargo install cargo-expand`
- Expand specific modules: `cargo expand path::to::module`
- This allows analysis of what the macro expansion produced and identification of recursion or other expansion issues

## Documentation Standards

- When adding code documentation, provide short and concise explanations that are accessible to developers without compiler design or PDF specification experience.
- Use analogies and real-world examples where appropriate to explain complex concepts.
- Document edge cases, error conditions, and their implications for PDF processing.
- When the code changes, ensure that the documentation is updated to reflect the new state of the codebase.
- When the function or module is complex, consider using diagrams or flowcharts to illustrate the logic and data flow.
- The documentation should be a few lines, focusing on the "why" rather than the "how" of the implementation.
- The documentation is not applied to tests.
- The documentation is applied to all public functions, structs, and modules.
- The documentation might be applied for `pub(super)`, `pub(crate)`, and private items, depending on their complexity and importance.

## Rust module internal organization

When having multiple structs in a single file, organize them as follows:
- Place the most important struct first, followed by less important ones.
- Structs are placed at the top of the file, followed by their implementations.
- The implementations related to one struct should be grouped together.
- This project is using `lib.rs`, not `mod.rs`.

## Testing and Validation

- When asked to write tests, ensure they cover:
  - Normal cases as per the PDF specification
  - Edge cases, including malformed PDFs and error conditions
- Ensure the number of tests is proportional to the complexity of the code being tested.
- Check the code coverage by using `cargo llvm-cov --lcov --output-path target/lcov.info`
- Test cases naming convention should follow the pattern `test_<function>_when_<condition>_expect_<expected_result>`, where:
  - `<function>` is the name of the function being tested
  - `<condition>` describes the specific scenario being tested, optional
  - `<expected_result>` describes the expected outcome of the test

## Rust Code Organization

- Place primary structs at the top of files, followed by implementations
- Group related `impl` blocks together for the same struct
- Use workspace dependencies from `Cargo.toml` rather than adding new ones
- Follow the pattern: `mod submodule; pub use submodule::PublicType;`