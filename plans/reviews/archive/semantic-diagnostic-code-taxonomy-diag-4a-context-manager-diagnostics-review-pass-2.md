---
name: semantic-diagnostic-code-taxonomy-diag-4a-context-manager-diagnostics-review-pass-2
description: Review pass 2 — verify the pass-1 closure-shadowing nit on slice 2b.21 is resolved and confirm no new regressions in the context-manager protocol diagnostic migration.
---

# Review — `milestone_diag_4a` slice 2b.21: context-manager protocol diagnostic

- Branch: `codex/semantic-diagnostics-diag-4a-protocol-surface-diagnostics`
- Scope: same as pass 1 — migrate the class-without-`__enter__`/`__exit__` diagnostic from the legacy `CompilePhase::TypeCheck` → `SIFR-TYPE-0001` transitional bridge to active `SIFR-PROTO-0003` via `protocol_diagnostics::context_manager_missing`.
- Pass: 2
- Prior review: [reviews/semantic-diagnostic-code-taxonomy-diag-4a-context-manager-diagnostics-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-4a-context-manager-diagnostics-review-pass-1.md).

## Summary

Pass 2 re-reads the same tree of changes after the author absorbed the single non-blocking finding from pass 1 (closure-parameter shadowing on the freshly-introduced outer `name` binding). The fix is exactly what was suggested: closures at [crates/sifr_hir/src/lower/statements.rs:299](crates/sifr_hir/src/lower/statements.rs:299) and [crates/sifr_hir/src/lower/statements.rs:302](crates/sifr_hir/src/lower/statements.rs:302) now bind `method_name` instead of reusing `name`, so the outer `name: &String` (passed to `protocol_diagnostics::context_manager_missing` at [statements.rs:309](crates/sifr_hir/src/lower/statements.rs:309)) is no longer shadowed inside the `.any(...)` closures. No other diff lines moved.

Everything else verified in pass 1 still holds: helper format-string ↔ registry template ↔ generated `docs/errors/SIFR-PROTO-0003.md` ↔ `internal_docs/diagnostic_codes.md` row ↔ fixture `expect-error` line ↔ unit-test message assertion are byte-identical for the migrated branch; the partial-protocol and non-class branches stay correctly on raw `ctx.error` (out of scope for this slice); the `SIFR-PROTO-0003` registry owner remains `sifr_hir::lower::statements`; the phase tracker correctly marks 2b.20 merged with PR 1692 and 2b.21 in progress.

The user-reported post-fix gates (`cargo fmt --check`, `cargo test -p sifr_hir protocol_diagnostics`, `cargo clippy --workspace -- -D warnings`) plus the earlier full-slice gates (`gen-error-docs`, `check_diagnostic_docs_sync.py`, `check_diagnostic_schema_sync.py`, `check_hir_maintainability_guardrails.py`, `cargo test -p sifr_diagnostics`, `cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo test -p sifr -- --skip test_e2e_pass`) cover every surface this slice touches.

No blockers. No new findings. Ready to merge.

## Verification of pass-1 finding

### Finding #1 from pass 1 — closure parameter shadowing (resolved)

Pass-1 source ([pass-1 review §1](reviews/semantic-diagnostic-code-taxonomy-diag-4a-context-manager-diagnostics-review-pass-1.md)):

```rust
let has_context_manager = if let Type::Class { name, methods, .. } = &val_ty {
    let has_enter = methods.iter().any(|(name, _)| name == "__enter__");
    let has_exit  = methods.iter().any(|(name, _)| name == "__exit__");
```

Pass-2 source ([crates/sifr_hir/src/lower/statements.rs:296-302](crates/sifr_hir/src/lower/statements.rs:296)):

```rust
let has_context_manager = if let Type::Class { name, methods, .. } = &val_ty {
    let has_enter = methods
        .iter()
        .any(|(method_name, _)| method_name == "__enter__");
    let has_exit = methods
        .iter()
        .any(|(method_name, _)| method_name == "__exit__");
```

The fix matches option 1 from pass-1 §1 verbatim — closures rebind to `method_name`, the outer `name: &String` is no longer shadowed, and the call to `protocol_diagnostics::context_manager_missing(ctx, name)` at [statements.rs:309](crates/sifr_hir/src/lower/statements.rs:309) reads the outer binding directly with no risk of confusion. Reformatted onto multiple lines (one `.iter().any(...)` chain per builder), which is a stylistic consequence of the longer parameter name; `cargo fmt --check` (user-reported PASS) is consistent with this.

Behavior is preserved: same `Type::Class` destructure, same `methods.iter().any(...)` predicate semantics, same comparison strings (`"__enter__"`, `"__exit__"`), same control flow into the three error branches, same call into the helper with the class name. No semantic delta.

I confirm the nit is fully resolved.

## Re-verification of pass-1 confirmations

I re-checked each pass-1 confirmation that could plausibly be invalidated by the closure rename. Only the immediate diff hunk in `statements.rs` changed between passes; everything else is byte-identical to pass 1.

- **Registry entry** [crates/sifr_diagnostics/src/codes.rs:1001](crates/sifr_diagnostics/src/codes.rs:1001): unchanged from pass 1, message template still `"type '{type_name}' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)"`, owner `"sifr_hir::lower::statements"`, `declared_args` and `dedupe_args` `["type_name"]`.
- **Helper format-string** [crates/sifr_hir/src/lower/protocol_diagnostics.rs:23](crates/sifr_hir/src/lower/protocol_diagnostics.rs:23): unchanged, byte-identical to the registry template (modulo `{type_name}` placeholder vs literal substitution).
- **Generated docs** [docs/errors/SIFR-PROTO-0003.md:13](docs/errors/SIFR-PROTO-0003.md:13) and **internal_docs row** [internal_docs/diagnostic_codes.md:116](internal_docs/diagnostic_codes.md:116): unchanged from pass 1, both match the registry template.
- **Fixture re-key** [crates/sifr/tests/e2e/fail/with_non_context_manager.sifr:1](crates/sifr/tests/e2e/fail/with_non_context_manager.sifr:1): unchanged, still `SIFR-TYPE-0001` → `SIFR-PROTO-0003`.
- **Unit test** [crates/sifr_hir/src/lower/protocol_diagnostics.rs:68-79](crates/sifr_hir/src/lower/protocol_diagnostics.rs:68): unchanged, still pins both message-string equality and `error.code == Some(DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING)`.
- **Phase-tracker bookkeeping** [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:55-56](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:55): unchanged, 2b.20 marked merged with PR 1692, 2b.21 in-progress.
- **Maintainability guardrails**: `statements.rs` is now 2180 lines (vs 2178 in pass 1 — net +2 from the line-break reformat of the two `.any` chains; cap is 2200, comfortable margin remains). `protocol_diagnostics.rs` is 80 lines (no per-file cap). `mod.rs` unchanged at 1200. The user-reported `check_hir_maintainability_guardrails.py` PASS is consistent.
- **Carry-over note from slice 2b.20 (test placement)**: still inline `#[cfg(test)] mod tests` rather than a sibling `_tests.rs`. Convention divergence persists across all three tests in the file; deferring to a future protocol-domain slice that can extract them all in one move remains the cleanest path. Not a finding for 2b.21.

## What's still out of scope (carry-over confirmation)

Same as pass 1 — these are intentional and correct for this slice's narrow scope:

- The partial-`__enter__`-or-`__exit__` branch at [statements.rs:305-307](crates/sifr_hir/src/lower/statements.rs:305) remains on raw `ctx.error` (no fixture coverage today; routes through the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge in `sifr_driver`).
- The non-class type branch at [statements.rs:312-315](crates/sifr_hir/src/lower/statements.rs:312) remains on raw `ctx.error` (same situation).

Both will need structured codes (or a justified reuse of `SIFR-PROTO-0003`) plus fixture coverage before the `SIFR-TYPE-0001` bridge can be deleted; that work is appropriately deferred to a later slice.

## Test coverage assessment

Unchanged from pass 1: one new unit test (`missing_context_manager_has_proto_code`) pinning both message and code on the migrated branch, plus the re-keyed e2e fixture asserting `SIFR-PROTO-0003` end-to-end. Coverage is appropriate for the slice's scope. The closure rename does not affect test surface.

## Recommendation

**Ready to merge.** The pass-1 readability nit has been cleanly addressed exactly as suggested, no new issues surfaced under pass 2, and the slice remains correctly scoped with full call-site ↔ helper ↔ registry ↔ docs ↔ fixture ↔ unit-test alignment. Open the PR.
