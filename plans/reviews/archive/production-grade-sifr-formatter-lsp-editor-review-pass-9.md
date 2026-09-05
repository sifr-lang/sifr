

# Review: Ad Hoc Production-Grade Sifr Formatter Phase

Reviewer: agent — Pass 9
Files reviewed: `issues/ad-hoc-production-grade-sifr-formatter.md`, `issues/ad-hoc-production-grade-sifr-formatter-execution.md`, `internal_docs/phases/36_developer_tooling_and_ecosystem_hooks.md`, `internal_docs/lsp_server.md`, `internal_docs/editor_integrations.md`, `internal_docs/tooling_analysis.md`, `internal_docs/vscode_extension.md`, `verification/tooling/lsp_protocol_matrix.json`, `demos/formatter_showcase/main.sifr.input`, `reviews/production-grade-sifr-formatter-lsp-editor-review-pass-9.md`

---

## Findings (Ordered by Severity)

### No Blockers Found

The plan is implementation-ready with no remaining blockers.

### Detailed Verification

**Expectation 2 — Missing decisions, vague milestone boundaries, missing validation, split-brain formatter risk, editor integration gaps, unsupported Ruff parity:**

- **Ruff capability parity**: Covered by the locked capability matrix in the execution tracker with 40 rows, each classified as `supported`, `adapted`, `not-applicable`, or `not-exposed`. The `not-exposed` rows (Python `--target-version`) and `not-applicable` rows (import sorting, notebook formatting) have written rationale. The no-new-unclassified-row rule is explicit.
- **Split-brain formatter risk**: The plan explicitly forbids the fallback whitespace formatter path (M3 closes the W-1 work item), forbids source post-processing for parameter conventions (M2 review gate), and requires one Ruff-backed formatter core shared by CLI, analysis, and LSP. Forbidden behaviors are enumerated with no ambiguity.
- **Milestone boundaries**: Each of the 7 milestones has specific entry criteria, scope, outputs, validation commands, and an external review gate. The sequential dependency is explicit: later milestones depend on earlier contracts being executable, not merely documented.
- **Validation**: The execution tracker wires in formatter AST coverage guardrail, formatter corpus checks, LSP smoke/stress, editor asset checks, and CLI/config fixtures. The showstopper demo (`demos/formatter_showcase/main.sifr.input`) is checked in and validated in the execution log.
- **Missing decisions**: The execution tracker logs 8 review passes and explicitly states pass 6 resolved the last deferred stdin-without-filename decision. No deferred planning decisions remain.

**Expectation 3 — Editor integrations and LSP primary path:**

- **All 5 editors covered**: Neovim, Zed, Helix, Emacs, and VS Code each have documented formatter support in the execution tracker with explicit LSP-based formatter paths. The forbidden behavior section explicitly bans direct `sifr fmt` as the primary editor formatting provider.
- **LSP primary path**: The execution tracker locks the contract that editors launch `sifr lsp --stdio` and request formatting through standard LSP methods. `sifr_format` is the shared formatter core; `sifr_analysis` routes formatting queries; `sifr_lsp` is a protocol adapter only. The protocol matrix has positive/negative rows for `textDocument/formatting` and `textDocument/rangeFormatting` with the `lsp-formatting` budget.
- **VS Code**: The extension uses the native LSP client document formatting provider and `editor.formatOnSave`, with no extension-owned formatter implementation. The extension contract is locked in `internal_docs/vscode_extension.md` and `verification/tooling/vscode_extension_rules.json`.

**Expectation 4 — Phase 36 extension vs. modification:**

- The phase plan explicitly builds on Phase 36: "based on the current Sifr and Ruff contracts", "replace the conservative Phase 36 formatter foundation" (the conservative foundation is the current state to be replaced, not Phase 36's intent). The execution tracker reinforces this by recording W-1 as "the conservative Phase 36 implementation" that M3 replaces. No Phase 36 contract is modified; Phase 36's `sifr_format`, `sifr_analysis`, `sifr_lsp`, `sifr_lint`, and editor integration contracts remain intact and are extended.

**Expectation 5 — Explicit readiness statement:**

- The phase is ready for implementation with no remaining blockers.

---

## Minor Observations (Not Blockers)

These are resolved design decisions confirmed by the review log, recorded here for completeness:

- `mut own` → `own mut` canonicalization is handled by a Ruff formatter rule in the fork (M2 review gate), not by Sifr wrapper post-processing. This is explicit and correct.
- Diff mode is Sifr CLI wrapper behavior around `format_source`, reusing Ruff source/diff utilities for unified diff generation. This is explicit in the Ruff Formatter Integration API section.
- Stdin without a filename uses current-directory config context and `.sifr` source kind. This is locked in the CLI parity contract.
- Formatter cache is adapted Ruff behavior with Sifr-versioned cache keys. This is in the capability matrix as `adapted`.
- Docstring code formatting is supported but disabled by default, matching Ruff's default. This is explicit in the docstring section.
- Pragma support (`fmt: off/on/skip`, `yapf: disable/enable`) is adapted to Sifr comment positions. The scope of expression-level non-suppression is explicit.

---

## Verdict

**The ad-hoc phase is implementation-ready. No blockers remain.**

The plan correctly extends Phase 36, makes LSP the primary editor formatter path, covers all 5 existing editor integrations, achieves Ruff formatter capability parity with explicit non-applicability rationale, locks the CLI and config parity contracts, defines clear milestone boundaries with validation evidence requirements, and prevents split-brain formatter paths through explicit forbidden behavior rules. The 8-review-pass review log demonstrates thorough validation. The execution tracker addendum locks the planning decisions before implementation. The showstopper demo is checked in and validated.

**Implementation can proceed when the team is ready.**
