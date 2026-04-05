<!--
Sync Impact Report
Version change: template -> 1.0.0
Modified principles:
- Template Principle 1 -> I. Code Quality Is Non-Negotiable
- Template Principle 2 -> II. Tests Prove Behavior
- Template Principle 3 -> III. User Experience Must Stay Consistent
- Template Principle 4 -> IV. Performance Is a Product Requirement
Added sections:
- Implementation Standards
- Delivery Workflow
Removed sections:
- Template Principle 5 placeholder section
Templates requiring updates:
- ✅ .specify/templates/plan-template.md
- ✅ .specify/templates/spec-template.md
- ✅ .specify/templates/tasks-template.md
- ✅ .specify/templates/commands/*.md not present; no command template updates required
- ✅ .github/prompts/speckit.constitution.prompt.md reviewed; no update required
- ✅ README.md and rust/README.md reviewed; no update required
Follow-up TODOs:
- None
-->

# TRust PDF Constitution

## Core Principles

### I. Code Quality Is Non-Negotiable
All changes MUST preserve the compiler pipeline boundaries (`lexer -> parser -> syntax -> semantic`),
address root causes instead of layering workarounds, and stay minimal in scope.
Rust implementation work MUST follow `.github/instructions/rust.instructions.md` and
`.github/copilot-instructions.md`, including idiomatic Rust usage, memory-safe zero-cost
abstractions, selective re-exports, and concise why-focused documentation for public APIs and
complex internals. Reviewers MUST reject incidental refactors, unclear ownership, or public
behavior changes that are not documented in the same change.
Rationale: The repository is a compiler pipeline with tight architectural coupling, so unclear or
casual changes compound into correctness and maintenance failures quickly.

### II. Tests Prove Behavior
Every behavior change MUST add or update tests at the appropriate level before the work is
considered complete. Lexer, parser, syntax, and semantic changes MUST cover both
spec-compliant inputs and malformed PDF recovery cases; SafeDocs adjacency and whitespace cases
MUST be included whenever tokenization, delimiter handling, or trivia preservation changes.
Rust tests MUST follow the repository naming convention and use `pretty_assertions` for
equality-heavy comparisons so failures remain diagnosable. Validation MUST run through
`cargo nextest run` or a narrower command explicitly justified in the implementation plan.
Rationale: Parser resilience and PDF correctness cannot be established by code inspection alone.

### III. User Experience Must Stay Consistent
User-facing behavior across the Rust library, documentation, and editor integrations MUST use
consistent terminology, diagnostics, severity, and formatting. Any change to messages,
diagnostics, rendered structure, extension behavior, or documented workflows MUST update the
corresponding documentation and regression coverage in the same change. New user-visible flows
MUST match existing naming and recovery expectations unless the implementation plan records and
justifies a deliberate deviation.
Rationale: Users consume TRust PDF through multiple surfaces, and inconsistency makes correct
behavior look broken.

### IV. Performance Is a Product Requirement
Changes in hot paths such as lexing, parsing, tree allocation, and traversal MUST account for
allocation count, data layout, and cloning behavior up front. Implementations MUST preserve or
improve the repository's memory-oriented patterns, including arena-style storage, inline payloads,
pre-sized collections, and avoidance of redundant heap work. If a change risks throughput,
latency, or memory regression, the plan MUST state the expected impact and the verification
approach before implementation proceeds.
Rationale: Large or adversarial PDFs magnify small inefficiencies into product failures.

## Implementation Standards

- `.github/copilot-instructions.md` and `.github/instructions/rust.instructions.md` are
	normative implementation guidance and MUST be consulted for Rust and repository-wide work.
- Public APIs, public modules, and complex internal mechanisms MUST carry concise documentation
	that explains why the code exists or why a constraint matters.
- Dependencies MUST come from workspace-managed dependencies unless an exception is explicitly
	approved and recorded in the implementation plan.
- Feature plans MUST call out architecture boundaries, test scope, user-facing impact, and
	performance constraints before coding starts.

## Delivery Workflow

- Every specification MUST define independently testable user stories, identify malformed-input or
	recovery edge cases, and record measurable success criteria.
- Every plan MUST pass a constitution check covering code quality, test scope, UX consistency, and
	performance impact before Phase 0 research is considered complete.
- Every task list MUST include the validation work needed to prove the affected story, including
	tests and documentation updates whenever behavior changes.
- Review and merge decisions MUST treat unresolved constitution violations as blockers, not
	follow-up polish.

## Governance

This constitution supersedes local habits and ad hoc workflow notes. Amendments MUST be made in
`.specify/memory/constitution.md`, include an updated Sync Impact Report, and propagate changes to
dependent templates before adoption. Semantic versioning applies to this document: MAJOR for
removing or redefining a principle, MINOR for adding a principle or materially expanding
governance, PATCH for clarifications that preserve existing meaning. Every feature review MUST
explicitly verify compliance with this constitution, `.github/copilot-instructions.md`, and
`.github/instructions/rust.instructions.md`; deviations require written justification in the
implementation plan's Complexity Tracking section and reviewer approval.

**Version**: 1.0.0 | **Ratified**: 2026-04-05 | **Last Amended**: 2026-04-05
