---
title: Trivia
description: What trivia is in trust-pdf and why trivia is modeled in the green layer.
---
## Definition

Trivia represents the parts of the PDF binary data that are mainly insignificant for language syntax and they just help for separating syntactic constructs such as names, strings, etc.

```pdf
(sample string) 12345 % comment
```

In this example, we can note **spaces** and a **comment** (it starts with `%` character) added at the end of line.

Typical trivia includes:

- White-space characters (`\x00`, `\t`, `\n`, `\f`, `\r`, ` `) (ISO 32000-2:2020, §7.2.3)
- End-of-line markers (`\r`, `\n`, `\r\n`) (ISO 32000-2:2020, §7.2.3)
- Comments (for example `% comment`) (ISO 32000-2:2020, §7.2.4)
## Further reading

- [Green Nodes](./index.md)
- [Trivia Rust implementation](https://github.com/pogor-dev/trust-pdf/blob/main/rust/crates/code-analysis/src/syntax/green/trivia.rs)
- [Use the .NET Compiler Platform SDK syntax model](https://learn.microsoft.com/en-us/dotnet/csharp/roslyn-sdk/work-with-syntax)
- [PDF Compacted Syntax Matrix (SafeDocs)](https://github.com/pdf-association/safedocs/blob/a6fd37308c91a0d2c17ebcace970367181bc0da7/CompactedSyntax/CompactedPDFSyntaxMatrix.pdf)
