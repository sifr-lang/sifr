

Based on my review of all 13 files, here are my findings ordered by severity:

---

## CRITICAL (Blocker)

### C-1: No `check_linter_reuse_rules.py` validation script

**Phase:** Milestone 1 (`lint_reuse_contract_and_manifests`)

**What:** The validation plan ties `check_linter_reuse_rules.py` to AC-1 ("reuse matrix is encoded in machine-readable or checkable form"), but no such script exists in `verification/tooling/`. The `check_tooling_dependency_boundaries.py` catches `ruff_python_semantic`, `ty_project`, and `ruff_server` imports—it does **not** catch production imports of `ruff_linter::rules::*`, `ruff_linter::registry`, `ruff_linter::linter`, or `ruff_python_semantic` into `sifr_lint`.

**Why blocker:** AC-10 says `sifr_lint` must not import the Ruff Python rule engine. Without a script enforcing this at the Rust Cargo/package boundary before M1 closes, this is an unenforceable paper contract. A developer can merge an M2+ that adds `use ruff_linter::rules::*;` and nothing fails.

**Fix:** M1 must add `verification/tooling/check_linter_reuse_rules.py` that parses `crates/sifr_lint/Cargo.toml` for forbidden crate dependencies, runs `cargo tree -p sifr_lint` in restricted mode, or uses `cargo-geiger` to verify no forbidden paths transit.

---

### C-2: No explicit pre-M5 proof that parser-aware suppression blocks syntax/HIR rules

**Phase:** M3 (`parser_aware_suppression_engine`) gates M5 (`sifr_policy_rule_families`)

**What:** AC-5 says parser-aware suppression must be implemented "before any non-physical-line rules ship," and M3's scope correctly precedes M5. However, M5's scope (add representative syntax, HIR, statement-range rules) has no explicit negative validation: nothing proves M5 code **fails to compile or test** if M3 hasn't merged first.

**Why blocker:** Shared-team repositories do not enforce milestone sequencing at the mechanical level. M5 could land before M3 and the positive AC-5 criterion (parser-aware suppression exists) would still be provable—but the gate's protective intent ("non-trivial rules cannot ship with line-only suppression") would be violated.

**Fix:** Either (a) add a `check_syntax_hir_rules_require_parser_aware_suppression.py` that asserts M3 completion status before M5 tests pass, and wire it into the milestone gate, or (b) restructure M5's crate module to explicitly require the suppression module before the syntax/HIR checkers compile, making the dependency mechanical rather than advisory.

---

## HIGH

### H-1: Blanket suppression rule exists but blanket suppression itself is not gated with parser-aware suppression

**Phase:** Current `sifr_lint` implementation and M3

**What:** The current `sifr_lint/src/lib.rs:98-105` includes `"blanket-suppression"` as a policy rule that reports bare `# sifr: ignore` (no rule id listed) as a diagnostic. The reuse matrix row "Ruff file-level blanket `noqa`" → "reject unless reviewed later" is correct. But the blanket-suppression rule is a policy rule, not a hard gate. The quality contract's "blanket suppressions remain forbidden in this phase" has no mechanical enforcement beyond the diagnostic reporting.

**Why significant:** An M5 policy rule that spans a multi-line statement (function, class, match/case) could theoretically be blanket-suppressed on the first line of that construct if blanket suppression were re-enabled as a future option. The phase correctly keeps blanket suppression rejected, but the implementation detail (a policy rule is not a compiler gate) is not made explicit.

**Recommendation:** Add a note to M3 scope: blanket suppression reporting is a policy diagnostic, not a hard gate. Mechanical rejection (e.g., a hard compile-time flag or explicit feature flag) for blanket suppression is deferred to a future phase that explicitly revisits the blanket-rejection decision.

---

### H-2: `DiagnosticMode` semantics need explicit policy for LSP diagnostics modes

**Phase:** M2 (`lint_config_and_file_discovery`), M6 (`lint_fixes_and_code_actions`)

**What:** The current `DiagnosticMode` enum (`Off`, `OpenFiles`, `Workspace`) in `sifr_lint/src/lib.rs:34-38` matches the reuse contract, but the TOML config example (line 185: `preview = false`) includes `unsafe-fixes = "hint"` without explaining the enum values. The `safety contract` for `unsafe-fixes` (hint vs. enabled vs. disabled) is introduced in the TOML example before the fix engine is built (M6).

**Why significant:** M6 must implement `unsafe-fixes` applicability, but the "hint" enum is not defined elsewhere. If "hint" is the only value the TOML exposes, it effectively means "safe fixes only, unsafe require explicit opt-in." This is sound, but the phase does not explain the design decision.

**Recommendation:** Add a TOML config semantics note to M2 or M6 explaining that `unsafe-fixes = "hint"` means the lint engine shows unsafe fix applicability in diagnostics but never applies them automatically. Fix application follows the same hard-vs-policy split: hard diagnostics are never fixed, policy diagnostics are fixed only when `unsafe-fixes = "enabled"` and `Applicability = Safe` or explicitly user-confirmed unsafe.

---

### H-3: The `#[cfg_attr(test, allow(...))]` module attribute is anomalous**Phase:** Current `sifr_lint` implementation

**What:** `crates/sifr_lint/src/lib.rs:2` uses `#[cfg_attr(test, allow(clippy::expect_used, clippy::unwrap_used))]`. No other Sifr crate has this pattern. The `expect_used`/`unwrap_used` allowances are needed only in tests—the standard Sifr pattern is to use `.expect()` in tests where panics are acceptable (they're in controlled test environments), without suppressing lint warnings in the module itself.

**Why significant:** If a future `.unwrap()` or `.expect()` is added in non-test code, this attribute suppresses the lint, hiding the problem silently.

**Recommendation:** Remove the module-level attribute and address the specific test lines that require these suppressions individually. Run `cargo clippy --workspace -- -D warnings` after removal to identify which test assertions need review.

---

### H-4: `collect_sifr_files_inner` uses `fs::read_dir` without depth or budget limits

**Phase:** Current `sifr_lint` implementation, M2 (`lint_config_and_file_discovery`)

**What:** `crates/sifr_lint/src/lib.rs:189-228` recursively traverses directories without any traversal depth limit, budget, or cancellation token. This is fine for local workspaces but becomes problematic for large monorepos or malicious inputs. The audit review (`reviews/sifr-linter-ruff-file-discovery-review.md`) correctly identified the `ignore` crate's `WalkBuilder` as the fix, but the phase should explicitly scope this performance/safety concern.

**Why significant:** Without depth/budget limits, a pathological directory structure (deep symlink chains, millions of files) could wedge the lint process. The M2 validation does not include negative tests for adversarial file trees.

**Recommendation:** Add negative fixture coverage for deep/unbounded traversal in M2's file discovery milestone. Wire the discovered behavior into performance budgets.

---

## MEDIUM

### M-1: `RuleSelector` is deferred without a specific rule-count trigger

**Phase:** M5 (`sifr_policy_rule_families`)

**What:** The reuse matrix says `RuleSelector` prefix/specificity model is "adapt later" with no explicit trigger condition. The registry/code generation row says "static registry is acceptable until rule count justifies macro generation." Neither row specifies the threshold (e.g., "~50 rules" as suggested in the review).

**Why medium:** The review artifacts recommend hand-maintaining the static `RULES` slice until ~50 rules. The phase should commit to that number as the trigger, so future implementers know when to add `rule_selector.rs` and when to consider the macro-based registry.

**Recommendation:** Add to M5's scope a rule-count threshold section: "If rule count exceeds 50, revisit static registry for `RuleSelector` and macro-based registry generation."

---

### M-2: No explicit coverage for Sifr import organization and isort rejection

**Phase:** M5 (`sifr_policy_rule_families`) and M6 (`lint_fixes_and_code_actions`)

**What:** The phase correctly rejects Ruff's organize imports/isort as "not applicable; requires a separate Sifr import-organization lint/fix phase after Sifr import semantics are specified." However, the phase does not document the boundary: import-order diagnostics (e.g., "imports should be alphabetically ordered") would be the first candidate rule, but it requires Sifr import semantics (module resolution, `use` statements, extern imports) to be specified first.

**Why medium:** A future implementer could add an `isort`-style rule to M5 without recognizing that Sifr's import model differs from Python's (positional imports, `use` statements, extern). The phase's reuse matrix correctly flags this as reference-only/reject, but the implementation scope in M5 should explicitly say "workspace/import policy rules only where Sifr workspace/import semantics are already specified" — which the scope in line 361 does say. The confirmation is implicit.

**Recommendation:** Add a note to M5's scope explicitly calling out "import ordering rules are explicitly excluded until Sifr's `use`-statement semantics and extern import ordering policy are specified."

---

### M-3: Deferred code-action resolution is listed in M6 but is only a stub in current implementation

**Phase:** M6 (`lint_fixes_and_code_actions`)

**What:** The review artifacts (`reviews/sifr-linter-ruff-lsp-editor-review.md`) identify deferred code-action resolution as "HIGH — No Code Action Resolution Pattern." The current `sifr_analysis` implementation has no `WorkspaceEditTracker` or deferred resolution pattern. M6's scope correctly includes "deferred code-action resolution for expensive edits," but this capability depends on the `WorkspaceEditTracker` pattern from Ruff, which the reuse matrix classifies as "adapt."

**Why medium:** Deferred resolution is not technically a blocker for M6 scope (M6 can ship fix code actions as synchronous), but the phase should distinguish between "synchronous fix code actions" (M6 minimum viable) and "deferred resolve code-action / workspace edit tracking" (M6 full). The current implementation has neither.

**Recommendation:** Scope M6 as two sub-tracks: M6a (synchronous fix code actions with applicability, isolation, conflict resolution) and M6b (deferred resolution and workspace edit tracking). Document that M6a is the M6 minimum-gate and M6b is the enhanced completion.

---

### M-4: Docstring for suppression complexity levels needs a single definitive reference

**Phase:** Across the phase document

**What:** The suppression complexity model (physical-line, single-node, statement-range, symbol-workspace) is introduced in the phase document and referenced as a key design primitive. It is used correctly in M3's scope (parser-aware statement/range mapping) and M5's scope (suppression complexity classification per rule). However, the term "suppression complexity" appears in 4+ places with slightly different framing, and the term "statement-range" appears in a non-obvious context.

**Why medium:** Future implementers need one authoritative reference for the suppression complexity model. Currently it is described in prose blocks that could be reorganized.

**Recommendation:** After the phase is implemented and reviewed, consolidate the suppression complexity model into `sifr_lint`'s module docs or a dedicated `crates/sifr_lint/src/suppression.rs` module. The phase document should reference this module as the canonical definition.

---

## LOW (Information or Already Handled)

### L-1: Phase correctly inherits formatter's `unsafe-fixes` contract as reference**Finding:** The formatter phase (Phase 36.1) defines `unsafe-fixes` as a safety model for the formatter. This linter phase correctly adapts (not directly reuses) the pattern, as indicated by the TOML example showing `unsafe-fixes = "hint"`. The distinction between formatter's safe vs. unsafe fixes (affecting generated Rust) and linter's safe vs. unsafe policy fixes (affecting source text) is implicit but sound.

**Verdict:** Already correct. The phases are appropriately independent on this surface.

### L-2: Review log is comprehensive enough

**Finding:** The review log shows three agent review passes plus subsystem reviews covering6 areas. Hidden Python semantic dependencies were a primary review target. The three-pass review structure (boundary sound → revised strategy sound → code cross-check) is robust and catches the key failure mode (Python rule semantic leakage).

**Verdict:** Review coverage is adequate. None of the review artifacts surface unresolved hidden decisions.

---

## Summary**The phase is not yet implementation-ready.** Two critical blockers must be resolved before M1 starts:

1. **C-1 (check_linter_reuse_rules.py):** Without a machine-enforceable script to verify no `ruff_linter` Python rules, `ruff_linter::registry`, `ruff_linter::linter`, or `ruff_python_semantic` imports enter `sifr_lint`, the reuse matrix is unenforceable. AC-10's guardrail cannot be verified.

2. **C-2 (parser-aware suppression gate enforcement):** Without mechanical proof that syntax/HIR rules cannot compile or test-pass before M3 closes, the suppression gate is advisory, not enforced.

The hidden planning decisions review found **no hidden decisions**: the reuse matrix, suppression complexity model, hard-vs-policy split, fix engine gates, milestone ordering, and validation plan are all explicit. The reuse decisions correctly reject all Python semantic dependencies. The suppression gate is correctly positioned in M3 before non-physical-line rules in M5. The TOML config surface, phase-gated engine architecture, LSP diagnostic class, fix applicability, and editor code actions are all specified with appropriate Ruff-reuse vs. Sifr-owned boundaries.

With C-1 and C-2 addressed, the phase is implementation-ready. The remaining H-1 through H-4 and M-1 through M-4 findings are significant but do not block the phase from starting—they are precision improvements that should be addressed as implementation proceeds.

**Recommendation:** Author a `verification/tooling/check_linter_reuse_rules.py` during M1 planning lock and add explicit suppression-gate enforcement (mechanical module dependency or negative test) before M5 opens. Address H-1 through H-4 and M-1 through M-4 in the implementation PRs where the relevant code is written. The phase structure, reuse decisions, milestone sequencing, and acceptance criteria are sound.
