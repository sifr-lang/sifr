# Phase 35/36 Production Tooling Sequence Memo

## Question

The current Phase 36 still reads like a production foundation plus MVP LSP. The user explicitly does not want an MVP. They want a full, elegant, production-grade sequential implementation plan.

This memo decides whether to refine Phase 35/36 or add extra phases before implementation.

## Current gap

Phase 35 is mostly correct as a prerequisite: it creates `sifr_syntax`, `sifr_frontend`, deterministic cache invalidation, performance budgets, and split-brain guardrails. It should remain a compiler/frontend foundation phase.

Phase 36 is not yet sufficient for the user's target because it currently:

- calls the LSP milestone an MVP
- marks references, inlay hints, code actions, and formatting optional
- defers rename, auto-import, advanced code actions, test explorer, generated-Rust preview, formatter, linter, marketplace publication, and mature editor syntax assets
- says `open-files` diagnostics are enough for MVP
- has only four coarse milestones, which is too broad for sequential execution

## Should we add extra phases?

Recommendation: do not add a new numbered phase before Phase 37 yet. Instead, expand Phase 36 into a full production tooling phase with more sequential milestones.

Reasons:

- The roadmap already has Phase 37 as package management. Inserting a new phase would renumber or create an awkward `36.5`.
- The missing work is not a separate domain; it is the actual completion of developer tooling.
- Phase 36 can be made sequential and reviewable by splitting into stricter milestones.
- Package management is related to future external-package intelligence and auto-import coverage, but Sifr can still ship production-grade editor tooling for the current project/workspace model before Phase 37. Phase 36 should explicitly document that package-aware external dependency intelligence expands after Phase 37, while current-workspace auto-import and symbols are in scope.

Ad hoc implementation slices are still acceptable as PR boundaries inside Phase 36, but the phase contract should list the full sequence.

## Production-grade Phase 36 scope

Phase 36 should require:

- `sifr_analysis` full editor query layer:
  - diagnostics
  - completion
  - hover
  - definition
  - declaration where distinct
  - type definition where meaningful
  - references
  - rename
  - document symbols
  - workspace symbols for current workspace
  - semantic tokens
  - inlay hints
  - signature help
  - document highlights
  - folding ranges
  - code actions from diagnostic suggestions
  - generated Rust preview query/command surface
- `sifr_lsp` full production server:
  - stdio transport
  - LSP 3.17
  - initialization/capability negotiation
  - full and incremental document sync
  - cancellation and stale-version handling
  - push and pull diagnostics
  - workspace and open-file diagnostics modes
  - all editor features above wired through `sifr_analysis`
  - deterministic protocol errors
  - no split-brain imports
- formatter/linter integration:
  - `sifr_fmt` or formatter module over `sifr_syntax`
  - LSP formatting provider
  - `sifr_lint` or Sifr-owned policy-rule engine for configurable policy diagnostics
  - hard correctness diagnostics remain non-suppressible
  - Sifr-specific suppression syntax, unknown suppression diagnostics, unused suppression diagnostics
  - include/exclude support
- editor assets:
  - VS Code extension is implemented enough to build/test/package, not just architected
  - TextMate grammar and/or Tree-sitter grammar, validated against `sifr_syntax`
  - VS Code language config, settings, commands, trace/logging, binary discovery, restart server, generated Rust preview, explain diagnostic
  - Neovim config documentation or contribution-ready config
  - Zed/Helix/Emacs integration docs/configs where low-cost and LSP-standard
- verification:
  - parity snapshots for all query types
  - protocol smoke/stress tests
  - VS Code extension integration test or packaged extension contract
  - formatting/lint/suppression/exclusion tests
  - completion quality evaluation inspired by `ty_completion_eval`
  - LSP performance budgets for every production feature class
  - multi-file/workspace scale tests

## Proposed sequential Phase 36 milestones

### 36.1 Production Tooling Contract Lock

Lock crate names, extension repo boundary, LSP capability matrix, diagnostic/rule policy, formatting/lint strategy, syntax asset strategy, and package-management boundary. Create:

- `internal_docs/tooling_analysis.md`
- `internal_docs/lsp_server.md`
- `internal_docs/vscode_extension.md`
- `internal_docs/editor_integrations.md`
- update `internal_docs/tooling_reuse_strategy.md` if implementation needs a reviewed strategy change

### 36.2 Diagnostics, Rules, Suppressions, Exclusions, And Formatting Foundation

Implement Sifr-owned policy-rule registry, hard-vs-policy diagnostic classification, Sifr suppression parser, unused/unknown suppression diagnostics, include/exclude discovery behavior, and formatter foundation over `sifr_syntax`.

### 36.3 AnalysisHost And Symbol Index

Implement `sifr_analysis` with project/open-file session state, symbol index, source map handoff, type display contract, docs extraction where available, generated-Rust preview query source, and stale-version-safe document updates.

### 36.4 Full Editor Query Layer

Implement diagnostics, completion, hover, definition/declaration/type-definition, references, rename, document symbols, workspace symbols, semantic tokens, inlay hints, signature help, document highlights, folding ranges, code actions, formatting query, and generated Rust preview query.

### 36.5 Production LSP Server

Implement `sifr lsp --stdio` using `lsp-server`/`lsp-types`, adapted ty/Ruff shell patterns, full/incremental sync, push/pull diagnostics, cancellation, request scheduling, deterministic errors, all required capabilities, and performance instrumentation.

### 36.6 Multi-Editor Syntax And Integration Assets

Deliver Tree-sitter/TextMate strategy fully enough for Neovim/Zed/Helix/Emacs consumption, validated against `sifr_syntax`, with documented editor configs and drift checks.

### 36.7 VS Code Extension

Implement and package the extension after shared syntax assets are validated: language id, grammar, language configuration, LSP launcher, settings, commands, trace/logging, generated Rust preview, explain diagnostic, restart server, binary discovery, VS Code Test Explorer integration through Sifr test metadata, integration tests, `.vsix` packaging, and marketplace-readiness checklist.

### 36.8 Production Verification And Performance Closeout

Finalize parity, protocol, stress, performance budgets, completion quality evaluation, extension packaging tests, split-brain dependency checks, rules/suppression/exclusion tests, formatter tests, and full local validation.

## Attention before starting

Before Phase 35 implementation starts, the plan should explicitly answer:

- Does Phase 35 expose enough stable HIR/symbol/type views for references, rename, signature help, semantic tokens, and generated-Rust preview?
- Does `sifr_diagnostics` need rule metadata additions before Phase 36, or can they land in Phase 36 without schema churn?
- Is current workspace/project discovery enough for Phase 36 current-workspace features, with package-aware external intelligence deferred to Phase 37?
- Where does the VS Code extension live? The plan should decide before implementation, not during extension work.
- Does formatter implementation require concrete AST/trivia preservation commitments in `sifr_syntax` beyond the current Phase 35 contract?
- Does generated-Rust preview require frontend/codegen APIs that should be exposed in Phase 35 or early Phase 36?

## Recommendation

Refine Phase 35 slightly to ensure it exports the data needed by full production editor tooling, especially symbol/type/display/codegen handoff gaps.

Rewrite Phase 36 from "tooling foundation plus MVP LSP" into "production developer tooling and editor ecosystem." No extra numbered phase is needed before Phase 37 if Phase 36 is expanded with the sequential milestones above.
