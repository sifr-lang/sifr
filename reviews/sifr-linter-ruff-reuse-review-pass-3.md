# Sifr Lint / Ruff Reuse Strategy — Review Pass 3

## Summary

Cross-referenced pass-2 claims against the actual codebase. The strategy is **sound and production-grade** with one confirmed blocker (suppression complexity classification is not yet implemented in code) and one naming inconsistency to fix.

---

## Findings, Ordered by Severity

### 🔴 Blocker: Suppression complexity classification missing from implementation

**Strategy section E** requires classifying each rule by suppression complexity:
1. physical-line token rule
2. single-node syntax rule
3. multi-line statement/range rule
4. symbol/HIR/workspace rule

**Current implementation** (`crates/sifr_lint/src/lib.rs:247-281`): suppression parsing is purely line-based. A `# sifr: ignore[rule-id]` on line N only suppresses diagnostics on line N. This is correct for `trailing-whitespace` (a physical-line token rule) but is a hard blocker before adding syntax rules (e.g., a multi-line function with a redundant construct) or HIR rules (e.g., unused-variable spanning multiple lines).

The strategy's requirement that "before adding syntax/HIR/semantic lint rules, `sifr_lint` must integrate suppression parsing with `sifr_syntax`" is **correct and is a prerequisite gate** — but it is not yet implemented. `sifr_syntax::ParsedModule` (crates/sifr_syntax/src/lib.rs) already exposes `suite()` (statement list) and `tokens()`, so statement-boundary tracking is possible. This gate must close before expanding rule families.

**Resolution path**: `parse_suppressions` must be extended to attach each `# sifr: ignore` comment to a statement range (via `sifr_syntax`), not just a line index. Until then, the only rules that can be added are physical-line token rules.

---

### 🟡 Naming inconsistency: `sifr_formatter` vs `sifr_format`

**Strategy section G** and pass-2 references mention `sifr_formatter`. The actual crate is **`sifr_format`** (architecture.md lines 276-294). This is a cosmetic inconsistency but should be corrected in the strategy document to avoid confusion during implementation.

---

### 🟡 Fix/suggestion infrastructure exists but is not used

**Strategy section C** mentions "fix applicability model: safe/unsafe or equivalent." The `sifr_diagnostics::SuggestionApplicability` enum (`crates/sifr_diagnostics/src/model/mod.rs:134-139`) already defines `MachineApplicable`, `MaybeIncorrect`, `HasPlaceholders`, `Unspecified` — exactly matching Ruff's model. However, the current lint rules (e.g., `trailing_whitespace_diagnostic` at sifr_lint/src/lib.rs:353-387) return `suggestions: Vec::new()` with only `help` text.

This is not a blocker — the infrastructure is correctly in place. It is a known implementation gap that will close when fix-capable rules are added.

---

### 🟢 No blocker: Core boundary is production-grade and verified

The following claims from pass-2 are **confirmed correct** by codebase cross-reference:

| Claim | Validation |
|---|---|
| **A: Sifr-lint owns rule engine** | `crates/sifr_lint/src/lib.rs` — `RULES` registry, `RuleSeverity`, `RuleMetadata`, `LintOptions`, `lint_source()`, `lint_path()`. Ruff linter not imported. |
| **B: Reuse only through sifr_syntax** | `crates/sifr_syntax/src/lib.rs` — `ParsedModule`, `SyntaxToken`, `SourceText`, `TextRangeUtf`. Wraps `sifr_python_parser` (the Ruff fork). No raw Ruff linter imports. |
| **C: Policy rule infrastructure is Sifr-owned** | `RuleMetadata`, `RuleSeverity`, `RuleStatus`, `LintOptions` all in `sifr_lint`. Severity is Ignore/Warn/Error for Sifr policy rules only. |
| **D: Ruff linter rules rejected** | No `ruff_linter::rules::*`, no `ruff_linter::registry`, no `ruff_linter::linter` imports in sifr_lint. |
| **D: Ruff noqa rejected** | `crates/sifr_lint/src/lib.rs:250-280` — parses `sifr: ignore[rule-id]` only. No `noqa`, no `E`/`W`/`F` prefixes. |
| **E: Suppression boundary requirement** | Correctly identifies the current line-based parser as only sufficient for token rules. Pass-1 blocker is valid and unresolved. |
| **D: File discovery path utilities** | `collect_sifr_files()` (sifr_lint/src/lib.rs:162-228) uses `std::fs::read_dir`. No `ruff_linter::fs` dependency. `read_ignore_patterns()` reads `.gitignore` and `.ignore` files manually. Clean separation. |
| **F: Rule-family classification** | Four rule families described in strategy match the planned expansion path: token/trivia → syntax → HIR/frontend → workspace/import. |
| **G: Config ownership** | `sifr.toml` is the Sifr config contract. `type: ignore` does not suppress Sifr diagnostics. Only `# sifr: ignore[...]` does. Confirmed in `suppression_shape_diagnostics` — unknown rule ids trigger `SIFR-LINT-0001`. |

---

### 🟢 No blocker: ruff_linter::fs boundary is clean

The suggestion from pass-1 to "reconsider extracting/reimplementing Ruff path utilities from `ruff_linter::fs`" is addressed correctly. The current implementation (`collect_sifr_files_inner` at sifr_lint/src/lib.rs:189-228) uses only `std::fs::read_dir`, `Path::is_dir`, `Path::is_file`, and manual `.gitignore` parsing. If future file-discovery needs grow (glob patterns, symlink handling, workspace root detection), clean language-neutral pieces from Ruff can be adapted — but the boundary is clean and no import contamination exists.

---

## Answers to the Four Questions

### 1. Does this resolve the suppression parsing blocker from pass 1?

**Partially.** Pass-1 identified the line-based suppression parser as a blocker before adding syntax/HIR rules. Pass-2 correctly documents this as a prerequisite gate (section E). However, the gate is not yet implemented in code. The blocker is *documented* but not *resolved*. The strategy correctly identifies the resolution path: integrate with `sifr_syntax` statement ranges. This must happen before expanding rule families.

### 2. Is the reuse/non-reuse boundary now production-grade?

**Yes.** The boundary is confirmed clean:
- Ruff fork only through `sifr_syntax` (AST/token/trivia/source utilities)
- Sifr-owned rule infrastructure (`sifr_lint`, `sifr_diagnostics`)
- No Ruff linter imports anywhere
- No Python semantics imported
- File discovery self-contained
- Suppression model Sifr-owned

### 3. Are there remaining blockers before presenting to the user?

**One blocker**: suppression complexity classification is not implemented. The strategy correctly identifies this as a prerequisite, but the gate must close before expanding rule families. This is a known gap — present it with the understanding that rule expansion is gated on this integration.

### 4. Important nuance around fs, file discovery, fixes, imports, or config?

- **`sifr_format`, not `sifr_formatter`** — fix naming before distribution.
- **SuggestionApplicability exists but unused** — infrastructure is ready, lint rules currently just emit help text. Not a blocker.
- **Config loading is runtime via LintOptions** — no `sifr.toml` integration in the lint crate yet. This is a future integration point, not a current blocker.
- **sifr_frontend exists** — `crates/sifr_frontend/` provides HIR analysis. Ready for HIR-rule integration once suppression gates close.

---

## Conclusion

**The strategy is sound with no blockers for the current scope.** The one remaining blocker (suppression parsing must integrate with `sifr_syntax` statement ranges before syntax/HIR rules can be added) is correctly identified in section E and is a documented gate, not an undisclosed gap.

The boundary between Ruff reuse (syntax substrate only) and Sifr ownership (all lint semantics, suppression, rules, config) is clean, verified against the actual codebase, and production-grade.

Ready to present to the user with the following caveats:
1. The suppression complexity gate (section E) must close before expanding beyond token rules.
2. Fix suggestions are modeled but not yet emitted by lint rules.
3. Fix the `sifr_formatter` → `sifr_format` naming before distribution.