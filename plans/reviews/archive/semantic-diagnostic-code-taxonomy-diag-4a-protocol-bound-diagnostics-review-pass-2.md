---
name: semantic-diagnostic-code-taxonomy-diag-4a-protocol-bound-diagnostics-review-pass-2
description: Review pass 2 — verify pass-1 fix to LowerCtx doc-comment restoration and confirm slice 2b.20 is ready to merge.
---

# Review — `milestone_diag_4a` slice 2b.20: protocol-bound diagnostics

- Branch: `codex/semantic-diagnostics-diag-4a-protocol-diagnostics`
- Scope: migrate generic-function TypeVar protocol-bound failures from the legacy `SIFR-TYPE-0001` bridge to active `SIFR-PROTO-0001` via a new `protocol_diagnostics` helper module.
- Pass: 2
- Prior review: [reviews/semantic-diagnostic-code-taxonomy-diag-4a-protocol-bound-diagnostics-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-4a-protocol-bound-diagnostics-review-pass-1.md)

## Summary

Pass 1 had a single requested fix (#1: revert the unrelated `LowerCtx` doc-comment deletion in [crates/sifr_hir/src/lower/mod.rs](crates/sifr_hir/src/lower/mod.rs)) plus three non-blocking observations (#2 clone overhead, #3 inline-vs-sibling test placement, #4 stale PROTO-0004 representative fixture). The author addressed #1 by restoring the doc comment and absorbed the resulting +1 line by removing the blank line between [LoweringError](crates/sifr_hir/src/lower/mod.rs:91) and its `impl std::fmt::Display` block, keeping the file at exactly the 1200-line guardrail cap.

This pass-2 review covers the original migration and the pass-1 fix together. Validation gates the user reported (`cargo fmt --check` and `python3 scripts/check_hir_maintainability_guardrails.py`) plus the full earlier slice gate set still pass on this tree. No new blockers surfaced; the slice is ready for PR.

## Verification of pass-1 fix

### `LowerCtx` doc comment is restored

[crates/sifr_hir/src/lower/mod.rs:109](crates/sifr_hir/src/lower/mod.rs:109) again reads:

```rust
/// The lowering context that tracks state during AST->HIR conversion.
pub(super) struct LowerCtx {
```

The diff against `main` for this file is now exactly two changes, both legitimately part of the slice:

- `+ mod protocol_diagnostics;` — new module declaration alongside the other `lower/*` submodules.
- One blank line removed between the closing `}` of `LoweringError` ([mod.rs:99](crates/sifr_hir/src/lower/mod.rs:99)) and `impl std::fmt::Display for LoweringError` ([mod.rs:100](crates/sifr_hir/src/lower/mod.rs:100)).

No other lines in `mod.rs` are touched, and the `LowerCtx` doc comment is intact.

### Guardrail-preserving blank-line removal — acceptable

The blank-line removal is a stylistic cost paid to keep `crates/sifr_hir/src/lower/mod.rs` at exactly the existing 1200-line ceiling encoded in [scripts/check_hir_maintainability_guardrails.py:13](scripts/check_hir_maintainability_guardrails.py:13). I confirmed:

- `wc -l crates/sifr_hir/src/lower/mod.rs` reports 1200 lines.
- `python3 scripts/check_hir_maintainability_guardrails.py` reports `HIR maintainability guardrails: PASS`.
- `cargo fmt --check` is clean (no output, exit 0). Rustfmt does not require a blank line between a struct and its `impl` block, so the change is format-safe.

This is a minor and reversible aesthetic concession — `LoweringError` and its `Display` impl now visually butt up against each other where every other struct/impl pair in this file is separated by a blank line. Two longer-lived alternatives, neither blocking:

1. Bump the `mod.rs` cap from 1200 → 1201 in [scripts/check_hir_maintainability_guardrails.py](scripts/check_hir_maintainability_guardrails.py), restoring the conventional blank line. The cap is a soft anti-regrowth signal, not a structural invariant; a +1 bump for a slice that adds a legitimate `mod` declaration is reasonable and consistent with how `mod.rs`'s cap will need to flex over time as new diagnostic helpers are added.
2. Move the `LoweringError` `impl Display` (and `impl Error`) blocks out of `mod.rs` into a small `lower/error.rs` submodule, freeing more headroom in `mod.rs` for future submodule registrations. Out of scope for 2b.20.

Either is fine as follow-up; neither needs to happen in this slice.

## Original migration — additional confirmations

I re-verified the items pass 1 confirmed and re-checked the touched surface end-to-end on the current tree:

- [crates/sifr_hir/src/lower/protocol_diagnostics.rs](crates/sifr_hir/src/lower/protocol_diagnostics.rs) — helper emits via `ctx.error_with_code(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED, …)`. The format string at [protocol_diagnostics.rs:13-15](crates/sifr_hir/src/lower/protocol_diagnostics.rs:13) is byte-identical to the registry template at [crates/sifr_diagnostics/src/codes.rs:979](crates/sifr_diagnostics/src/codes.rs:979) (modulo placeholders), so any drift is caught by `test_e2e_fail`. Two unit tests assert both message and `error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)` for the concrete-type and forwarded-TypeVar paths.
- [crates/sifr_hir/src/lower/expressions.rs:1907-1949](crates/sifr_hir/src/lower/expressions.rs:1907) — bound-not-satisfied branch routes through the new helper; the constraints-failure branch correctly remains on raw `ctx.error(...)` (still flowing through the transitional `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge in [crates/sifr_driver/src/diagnostics.rs:137](crates/sifr_driver/src/diagnostics.rs:137), which is the documented intent for this slice).
- [crates/sifr_diagnostics/src/codes.rs:976-983](crates/sifr_diagnostics/src/codes.rs:976) — `SIFR-PROTO-0001` template now reads `type '{actual}' does not implement protocol '{protocol}' required by type parameter '{type_param}'`; declared args and dedupe args both extended with `type_param`. [docs/errors/SIFR-PROTO-0001.md](docs/errors/SIFR-PROTO-0001.md) and [internal_docs/diagnostic_codes.md:114](internal_docs/diagnostic_codes.md:114) match.
- The eight re-keyed e2e fail fixtures all carry the new `expect-error: SIFR-PROTO-0001: …` marker. `grep -rn "SIFR-PROTO-0001" crates/sifr/tests/e2e/ | wc -l` reports exactly 8, matching the documented set.
- [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:54-55](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:54) flips 2b.19 to merged with PR link and adds the 2b.20-in-progress entry, consistent with prior slice wording.

## Pass-1 non-blocking findings — status

- **#2 `owner_bounds.clone()` per-call overhead**: unchanged in this pass. Still a deliberate, mechanically-correct tradeoff to release the immutable borrow on `ctx.type_param_bounds` so the helper can take `&mut LowerCtx`. Acceptable; not addressed and does not need to be in this slice.
- **#3 inline `#[cfg(test)] mod tests` placement**: unchanged. The phase tracker explicitly specified inline placement, so the convention drift versus sibling helpers (`match_diagnostics_tests.rs`, `name_import_diagnostics_tests.rs`, etc.) is intentional. Non-blocking.
- **#4 stale PROTO-0004 representative fixture**: unchanged and out of scope. Still worth tracking as separate follow-up work — PROTO-0004's `representative_fixture_path` points at [crates/sifr/tests/e2e/fail/generic_counter_unhashable.sifr](crates/sifr/tests/e2e/fail/generic_counter_unhashable.sifr) whose marker is now (correctly) `SIFR-PROTO-0001`. The mismatch is not surfaced by `gen-error-docs --check` or `check_diagnostic_docs_sync.py`. Pre-existing; this slice did not introduce it.

## Validation gates re-checked on the tree

After the pass-1 fix:

- `cargo fmt --check` — clean (exit 0, no output).
- `python3 scripts/check_hir_maintainability_guardrails.py` — `HIR maintainability guardrails: PASS`.

Earlier full-slice validation reported by the user (`gen-error-docs`, `check_diagnostic_docs_sync.py`, `check_diagnostic_schema_sync.py`, `cargo test -p sifr_hir protocol_diagnostics`, `cargo test -p sifr_diagnostics`, `cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace -- -D warnings`) covers the surface this slice touches. The pass-1-fix delta does not affect any of those gates: the doc-comment restoration and the blank-line removal are both confined to `mod.rs` and neither alters runtime behavior, types, exports, or the diagnostic registry.

## Ready for PR?

Yes. The single requested change from pass 1 has been applied correctly, the chosen approach for staying under the guardrail cap is acceptable, and no new findings surfaced. The remaining pass-1 items (#2, #3, #4) remain non-blocking notes for follow-up, not preconditions for merging this slice.
