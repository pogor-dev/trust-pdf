---
title: Trivia
description: What trivia is in trust-pdf and why trivia is modeled in the green layer.
---
# Trivia in the green layer

In `trust-pdf`, **trivia** is syntax text that is not a semantic token value, but is still part of the original file.
In read/render-only workflows it can often be ignored, but for precise file manipulation and rewriting it is essential to preserve full fidelity.

Here, **full fidelity** means the syntax model preserves every original source byte (including whitespace, comments, and line endings) so we can reconstruct the original source exactly.

Typical trivia includes:

- White-space characters (`\x00`, `\t`, `\n`, `\f`, `\r`, ` `) (§7.2.3)
- End-of-line markers (`\r`, `\n`, `\r\n`) (§7.2.3)
- Comments (for example `% comment`) (§7.2.4)

## Why trivia exists

Trivia is not "extra" data. It is required for:

- Exact source reconstruction
- Stable formatting around malformed input
- Correct diagnostics and tooling ranges
- Incremental parsing that preserves untouched text

Without trivia, the compiler could still understand structure, but tools would lose user formatting and comment placement.

## When whitespace is required

Most trivia can be normalized during lexing, but ISO 32000-2 still has space or EOL-sensitive rules:

- Token boundaries (§7.2.3): whitespace and delimiters separate tokens.
- Comments as separators (§7.2.4): comments are treated like one whitespace during lexical conversion.
- Indirect object headers (§7.3.10): object number, generation number, and `obj` must be separated (for example, `12 0 obj`).
- Xref subsection headers (§7.5.4): `first-object count` must be separated by one SPACE and end with EOL (for example, `28 5`).
- Xref entry width (§7.5.4): each row is fixed at 20 bytes including EOL, with exact field widths (`10-digit offset`, SPACE, `5-digit generation`, SPACE, `n|f`, EOL).
- EOL semantics (§7.2.3 + §7.5.4): `CR`, `LF`, and `CRLF` are valid EOL markers; placement still matters in line-oriented grammar such as `xref`.

## Why this matters for `GreenTrivia`

Even when trivia is "mostly ignorable," PDF has edge cases where exact trivia bytes are structurally relevant:

- line-sensitive table records (`xref`)
- fixed-width entry formatting (20-byte rows)
- required token separation around object framing
- comment-to-whitespace conversion during lexing

Storing trivia as raw bytes in green nodes ensures these constraints are preserved for round-tripping, diagnostics, and strict validation.

## Why trivia is green

The green layer is immutable, shareable, and position-free. Trivia fits this model well:

- The same trivia bytes (like one space) occur repeatedly
- Immutable trivia can be reused across nodes and trees
- Position is supplied later by red wrappers

This keeps the foundational representation compact and reusable, while the red layer provides parent and position context when needed.

## Representation in `trust-pdf`

`GreenTrivia` stores:

- `kind`: trivia category (`WhitespaceTrivia`, `CommentTrivia`, ...)
- `flags`: internal state (`is missing`, `has diagnostics`)
- `text`: raw trivia bytes

The bytes are stored inline with the allocation header (`HeaderSlice`/`ThinArc`) to reduce allocation overhead and improve locality.

## Diagnostics on trivia

Some malformed trivia still needs to survive parsing so the tree remains usable. In those cases, diagnostics are attached via a side table keyed by the trivia data address.

This design keeps the green value immutable while still allowing diagnostics lifecycle management.

## Relation to red syntax

Green trivia has no absolute position and no parent pointer. Red wrappers add that context at access time.

That split gives both:

- Efficient immutable storage
- Convenient API-level navigation

## Practical example

Suppose a PDF token is preceded by four spaces and a comment:

```text
	% section marker
```

Those bytes are trivia. They are preserved exactly in green storage so formatting and comments remain intact even after incremental edits.

## Further reading

- [Green Nodes](./index.md)
- [Trivia Rust implementation](https://github.com/pogor-dev/trust-pdf/blob/main/rust/crates/code-analysis/src/syntax/green/trivia.rs)
- [Use the .NET Compiler Platform SDK syntax model](https://learn.microsoft.com/en-us/dotnet/csharp/roslyn-sdk/work-with-syntax)
- [PDF Compacted Syntax Matrix (SafeDocs)](https://github.com/pdf-association/safedocs/blob/a6fd37308c91a0d2c17ebcace970367181bc0da7/CompactedSyntax/CompactedPDFSyntaxMatrix.pdf)
