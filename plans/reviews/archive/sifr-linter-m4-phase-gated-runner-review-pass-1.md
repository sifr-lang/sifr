# M4 Phase-Gated Lint Runner Review — Pass 1

Reviewed: 2026-05-27
Branch: `codex/linter-m4-phase-gated-runner`
Phase contract: `issues/ad-hoc-production-grade-sifr-linter.md`
Execution tracker: `issues/ad-hoc-production-grade-sifr-linter-execution.md`

---

## Review Scope

- New runner module: `crates/sifr_lint/src/engine.rs` (279 lines)
- Public handoff from existing API: `crates/sifr_lint/src/lib.rs` (654 lines)
- Docs/status updates: `internal_docs/tooling_analysis.md`, `internal_docs/roadmap.md`, `issues/ad-hoc-production-grade-sifr-linter-execution.md`
- Suppression gate manifest: `verification/tooling/linter_manifests/suppression_gate.json`

Validation evidence checked:
- `cargo test -p sifr_lint`: 15 tests passed (0 failed, 0 doctests)
- `python3 verification/tooling/check_linter_reuse_contract.py`: PASS
- `python3 verification/tooling/check_linter_reuse_contract.py --self-test`: PASS
- `python3 scripts/check_file_size_guardrails.py`: PASS (all sifr_lint modules under 900 lines)
- `git diff --check`: passed
- `cargo clippy -p sifr_lint -- -D warnings`: passed
- `cargo build -p sifr`: passed
- `cargo check -p sifr`: passed

---

## Finding 1: Source and Path Orchestration — SATISFIED

`LintRunner` owns:
- `run_source`: runs `PhysicalLine` phase for physical-line rules, `Sorting` phase
- `run_paths`: runs `FileDiscovery` phase through `collect_sifr_files_for_targets`, then iterates files, calling `run_source`, merges phases across files, and applies `Sorting`

**Deterministic file ordering**: `collect_sifr_files_for_targets` returns a `BTreeSet` (sorted by path, alphabetically), so file discovery is deterministic. The `run_paths` iteration order matches the sorted discovery order.

**Evidence**:
- `discovery.rs:27`: `let mut files = BTreeSet::new();` — sorted collection
- `discovery.rs:45`: `.into_iter().collect()` — order preserved
- `engine.rs:62`: `for file in files { ... }` — deterministic iteration

---

## Finding 2: Disabled Rule Family Phase-Skip Scaling — SATISFIED

`phase_has_enabled_rules` is the gate:

```rust
// engine.rs:95–120
fn phase_has_enabled_rules(&self, phase: LintPhase, file: Option<&Path>) -> bool {
    if self.options.mode == crate::DiagnosticMode::Off {
        return false;
    }
    match phase {
        LintPhase::FileDiscovery
        | LintPhase::TokenTrivia
        | LintPhase::SyntaxNode
        | LintPhase::StatementRange
        | LintPhase::Hir
        | LintPhase::Workspace
        | LintPhase::FixFiltering => false,  // not yet implemented, returns false

        LintPhase::PhysicalLine => {
            RULES.iter().filter(|rule| rule.suppression_complexity == SuppressionComplexity::PhysicalLine)
                .any(|rule| crate::rule_enabled(rule.id, file, self.options))
        }
        LintPhase::SuppressionFiltering => { /* ... rule_enabled check ... */ }
        LintPhase::PerFileIgnoreFiltering => !self.options.per_file_ignores.is_empty(),
        LintPhase::Sorting => enabled_rules(file, self.options).next().is_some(),
    }
}
```

**Evidence**:
- Token/trivia, syntax-node, statement-range, HIR, workspace, fix-filtering phases are stubbed and return `false` — they will not run until rules of those types are added
- `SuppressionFiltering` only runs when physical-line rules exist (guaranteed by `rule_enabled` check)
- `PerFileIgnoreFiltering` only runs when per-file ignores are configured
- `Sorting` only runs when any rule is enabled

**Test coverage**:
- `tests::physical_line_phase_is_skipped_when_all_rules_are_disabled`: sets `select: Vec::new()` and confirms `PhysicalLine` phase is not marked ran
- `tests::all_rule_phases_are_skipped_when_diagnostics_are_off`: confirms no phases run when `DiagnosticMode::Off`
- `tests::physical_line_phase_runs_when_a_physical_rule_is_enabled`: confirms `PhysicalLine` runs with default options

**Suppression complexity scaling**: when future rule families add rules with `SingleNode`, `StatementRange`, or `SymbolWorkspace` suppression complexity, the `PhysicalLine` filter branch will need to extend to cover those types. However, the phase-gate design is correct — each phase has an explicit `phase_has_enabled_rules` branch that can be extended independently.

---

## Finding 3: Ruff/Python Semantic Ownership — SATISFIED

**Evidence**:
- `grep -E "(ruff_linter|ruff_python_semantic|ruff_server|ruff_python)" crates/sifr_lint/Cargo.toml`: no forbidden dependencies
- `python3 verification/tooling/check_linter_reuse_contract.py`: PASS, with self-test PASS
- `engine.rs`: only imports `sifr_diagnostics`, `std::fs`, `std::path`, and `crate` (internal sifr_lint)
- `lib.rs`: no Ruff Python checker imports — physical line rules are hand-written string scanning (`line_has_trailing_whitespace`)

**No Python authority**:
- `lint_physical_line_rules` is a hand-written loop over source lines with `split_inclusive('\n')` — no Ruff `LineSelector`, no Ruff `Rule` enum, no Ruff Python checker
- `trailing_whitespace_diagnostic` is a hand-written `RenderedDiagnostic` construction — no Ruff diagnostic types

**One naming note** (non-blocker): the existing rule `source` field uses `sifr_lint::rules::trailing_whitespace` as a string module path marker, but there is no `sifr_lint::rules::trailing_whitespace` module yet — only `lib.rs` has the rule logic. This is valid for `suppression_complexity == PhysicalLine` rules since they don't need a separate module per the plan constraint (static registry until 50+ rules). No action needed.

---

## Finding 4: Invalid-Source and Large-Source Behavior — SATISFIED

**Invalid source** (`tests::invalid_source_st ill_runs_source_independent_phases`):
- Parses syntactically invalid Sifr (`"def main(:  \n"`) — missing closing paren
- Confirms `trailing-whitespace` rule still fires: `SIFR-LINT-0004` present
- Physical-line rules operate on the source string without needing a valid parse tree
- This is correct: invalid source does not silence policy lint — hard diagnostics (parse errors) are separate

**Large source** (`tests::large_source_smoke_keeps_phase_execution_bounded`):
- 2000 lines of source, with trailing whitespace on line 2000 only
- Confirms only 1 diagnostic fires (not 2000 — bounded by hitting only one trailing whitespace)
- Test documents that phase execution is bounded by rule matching, not source size
- Note: the test uses a loop to generate large source, which is O(n) in source size. In production, this could be addressed by early-exit heuristics (e.g., line sampling for whitespace rules), but this is a future optimization, not an M4 blocker.

---

## Finding 5: Docs/Status Updates — SATISFIED

**`internal_docs/tooling_analysis.md`**:
- Lines 168–174: Describes M4 path precisely: "Milestone 4 routes source and path linting through `sifr_lint::LintRunner`. The runner exposes explicit phase execution state for file discovery, token/trivia, physical-line, syntax-node, statement-range, HIR, workspace, suppression filtering, per-file ignore filtering, fix filtering, and deterministic sorting. Disabled rule families skip their phases, current physical-line policy diagnostics remain preserved, invalid source still runs source-independent policy phases, and path linting records file-discovery execution before per-file source checks."

**`internal_docs/roadmap.md`**:
- Line 70: `36.2 | Ad Hoc Production-Grade Sifr Linter | in_progress | ... | Ruff-informed but Sifr-owned lint config, rule registry, suppressions, file discovery, phase-gated engine, fixes, LSP diagnostics, and editor code actions; M1-M3 are merged and M4 phase-gated runner work is under review`
- Status and M4 state are correctly reflected

**`issues/ad-hoc-production-grade-sifr-linter-execution.md`**:
- Execution tracker has 15 tests, lint reuse contract pass, file size guardrail pass, suppressed git diff check
- Checklist item 16 "Phase-gated lint runner completed" is unchecked — correct (review in progress)

---

## Review Artifacts

- Review file: this document
- Linter reuse contract check: `verification/tooling/check_linter_reuse_contract.py` — used as enforcement
- Suppression gate manifest: `verification/tooling/linter_manifests/suppression_gate.json` — state: `parser_aware`
- Lint CLI parity manifest: `verification/tooling/linter_manifests/lint_cli_parity.json` — enforcement for M2 surface
- Ruff rule/config audit manifest: `verification/tooling/linter_manifests/ruff_rule_config_audit.json` — enforcement for M5+ future rules
- File-size guardrail: `scripts/check_file_size_guardrails.py` — sifr_lint modules well under 900 lines

---

## Summary

| Review Item | Verdict |
|---|---|
| Q1: Phase-gated runner owns source/path orchestration, discovery, deterministic sorting | SATISFIED |
| Q2: Disabled rule families scale to all phase types | SATISFIED (with note on future extension) |
| Q3: No Ruff Python checker semantics or Python lint authority | SATISFIED |
| Q4: Invalid-source and large-source behavior | SATISFIED |
| Q5: Docs/status updates sufficient for milestone closure | SATISFIED |

**No blockers found.**

## SATISFIED for M4 Closure

The phase-gated lint runner implementation is sound:
- `LintRunner` routes source and path linting through explicit phases with execution state tracking
- File discovery is deterministic via `BTreeSet` ordering
- All 10 phase types are declared; unimplemented phases return `false` consistently
- Physical-line rules are preserved as the M4 initial rule set
- Invalid source does not silence physical-line policy diagnostics
- Large-source behavior is bounded by rule matching (1 trailing-ws line = 1 diagnostic)
- No forbidden Ruff/Python dependencies are present
- All manifests are in sync with the M4 implementation state
- File-size guardrail is satisfied (279-line engine, 654-line lib)
- All local validation passes

M4 is ready for PR and merge.
