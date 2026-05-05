# Review — `milestone_diag_4a` slice 2b.20: protocol-bound diagnostics

- Branch: `codex/semantic-diagnostics-diag-4a-protocol-diagnostics`
- Scope: migrate generic-function TypeVar protocol-bound failures from the legacy `SIFR-TYPE-0001` bridge to active `SIFR-PROTO-0001` via a new `protocol_diagnostics` helper module.
- Pass: 1

## Summary

The slice does what was advertised: introduces a focused `protocol_diagnostics::bound_not_satisfied` helper, routes the lone bound-not-satisfied call site in [crates/sifr_hir/src/lower/expressions.rs:1909-1949](crates/sifr_hir/src/lower/expressions.rs:1909) through it with `DiagnosticCode::PROTO_BOUND_NOT_SATISFIED`, leaves the constraints-failure branch on the legacy `ctx.error(...)` path (correct, per scope), re-keys exactly the eight intended e2e fixtures from `SIFR-TYPE-0001` to `SIFR-PROTO-0001`, and aligns the PROTO-0001 registry/docs template + arg list with the emitted message. The phase tracker bookkeeping (2b.19 merged, 2b.20 in progress) is correct.

Validation gates the user reported (gen-error-docs, fmt, docs/schema sync, HIR guardrails, focused HIR + diagnostics + e2e fail tests, full HIR test minus e2e_pass, and clippy `-D warnings`) cover the surface this slice touches; no additional regression risk surfaced during this read.

I found one minor unrelated regression and a couple of stylistic / pre-existing-debt observations. None of them block the PR — the unrelated doc-comment deletion should be reverted before merge, the rest are notes for awareness.

## Findings

### 1. Unrelated doc-comment deletion on `LowerCtx` (minor — please revert)

[crates/sifr_hir/src/lower/mod.rs:109](crates/sifr_hir/src/lower/mod.rs:109) (pre-edit) carried:

```
/// The lowering context that tracks state during AST->HIR conversion.
pub(super) struct LowerCtx {
```

The diff removes that doc comment. The change is unrelated to slice 2b.20, has no rationale in any of the other touched files, and silently drops module documentation. There is no need for it as part of wiring the protocol-diagnostics helper. Recommend restoring the line so the slice diff stays scoped.

### 2. `owner_bounds.clone()` is now per-call hot-path overhead (minor — acceptable)

The original loop collected violations into a local `Vec<String>` so it could keep the immutable borrow on `ctx.type_param_bounds`, then drained into `ctx.error(...)` after the borrow released. Now that diagnostics are emitted in-loop via `protocol_diagnostics::bound_not_satisfied(ctx, …)` and `ctx.error(…)` (constraints branch), the new code clones the per-owner bounds map up-front to release the borrow:

```rust
if let Some(owner_bounds) = ctx.type_param_bounds.get(&func_name) {
    let owner_bounds = owner_bounds.clone();
    ...
}
```

This is mechanically correct and the bounds map per generic owner is small, but the clone is paid on every generic-function call site, even when there are no violations. If hot-path allocations matter later, a cheaper rewrite is to gather `(tv_name, concrete_ty.display_name(), bound, constraints)` tuples while the borrow is held and emit after the borrow ends — preserving the original "snapshot then emit" shape without the map clone. Not blocking; flag for the next pass if profiling shows pressure here.

Behaviorally I confirmed the new in-loop emission preserves the previous error ordering: bounds are emitted in `required_bounds` order, the constraint diagnostic is emitted last per TypeVar, and the outer `bindings` iteration order is unchanged.

### 3. Test placement diverges from sibling diagnostic modules (style only)

Other migrated diagnostic helpers in this directory keep tests in a separate `_tests.rs` file gated by `#[cfg(test)]`:

- `match_diagnostics.rs` + `match_diagnostics_tests.rs`
- `name_diagnostics.rs` + `import_diagnostics.rs` + `name_import_diagnostics_tests.rs`
- `ownership_diagnostics.rs` + `own_mut_*_tests.rs`

`protocol_diagnostics.rs` instead inlines `#[cfg(test)] mod tests`. The phase tracker explicitly says "Add unit tests inside `protocol_diagnostics.rs`," so this is intentional, but it's worth either (a) keeping the inline placement and accepting the convention drift, or (b) splitting into `protocol_diagnostics_tests.rs` to match the rest. Either is fine; flagging only so the divergence is a deliberate decision.

### 4. PROTO-0004's `representative_fixture_path` is now stale (pre-existing, not a regression)

[crates/sifr_diagnostics/src/codes.rs:1006-1015](crates/sifr_diagnostics/src/codes.rs:1006) declares `SIFR-PROTO-0004` ("Hashable or comparable protocol is required.") with `representative_fixture_path = "crates/sifr/tests/e2e/fail/generic_counter_unhashable.sifr"`. After this slice, that fixture's `expect-error` marker is `SIFR-PROTO-0001` (correct, since that's what the compiler now emits). PROTO-0004 has no emission site anywhere in `crates/sifr_hir/` — only the registry entry and the `DiagnosticCode::PROTO_HASHABLE_OR_COMPARABLE_REQUIRED` constant exist.

This is not introduced by 2b.20 — PROTO-0004 was already a placeholder pointing at a fixture whose actual emitted code is different — but the re-keying makes the inconsistency more visible. The `gen-error-docs --check` and `check_diagnostic_docs_sync.py` gates do not validate that a representative fixture's `expect-error` marker matches the entry's id, so this slice still passes them. Suggest tracking PROTO-0004 as separate work: either route a real emission into it, repoint its representative fixture, or retire/reserve it.

### 5. Migration coverage is complete for the stated scope (confirmation)

I grepped for every "does not implement protocol" / "does not satisfy" message under `crates/sifr_hir/src/`:

- The only protocol-bound emission for generic-function TypeVars is the one in `expressions.rs` — now structured.
- The constraints-failure branch at [crates/sifr_hir/src/lower/expressions.rs:1939-1946](crates/sifr_hir/src/lower/expressions.rs:1939) correctly remains on raw `ctx.error(...)`. Per the slice scope ("no dedicated active code exists in this slice"), this is intentional and will continue to flow through the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` transitional bridge in [crates/sifr_driver/src/diagnostics.rs:137](crates/sifr_driver/src/diagnostics.rs:137).
- The "does not implement the ContextManager protocol" emission in [crates/sifr_hir/src/lower/statements.rs:304-307](crates/sifr_hir/src/lower/statements.rs:304) is a separate `with`-statement check and out of scope here. (Note: that fixture, `crates/sifr/tests/e2e/fail/with_non_context_manager.sifr`, is still keyed `SIFR-TYPE-0001` and the emission still uses raw `ctx.error`. PROTO-0003 covers it in the registry but is not yet wired. Out of scope for 2b.20.)
- The two existing assertions in [crates/sifr_hir/src/lower/expressions_tests.rs:1888-1908](crates/sifr_hir/src/lower/expressions_tests.rs:1888) only check message substrings; the new helper-level tests now also assert `error.code == Some(DiagnosticCode::PROTO_BOUND_NOT_SATISFIED)`, which is a real upgrade in coverage.

### 6. Registry / docs / internal_docs alignment (confirmation)

- [crates/sifr_diagnostics/src/codes.rs:973-983](crates/sifr_diagnostics/src/codes.rs:973) registry entry: template + `declared_args` + `dedupe_args` now include `type_param`.
- [docs/errors/SIFR-PROTO-0001.md](docs/errors/SIFR-PROTO-0001.md): regenerated message template + declared args + dedupe args match.
- [internal_docs/diagnostic_codes.md:114](internal_docs/diagnostic_codes.md:114): row matches the new template / args.
- The emitted `format!` string in [protocol_diagnostics.rs:13-15](crates/sifr_hir/src/lower/protocol_diagnostics.rs:13) is byte-for-byte the same as the template (modulo the `{…}` placeholders). The eight fixtures' expected substrings match this exact format, so any drift between helper and template would be caught by `test_e2e_fail`.

### 7. Fixture re-keys (confirmation)

The eight re-keyed fixtures are exactly the set whose error originates in the migrated call site (generic call type-binding, including forwarded-TypeVar paths):

- [crates/sifr/tests/e2e/fail/generic_bounds_not_satisfied.sifr](crates/sifr/tests/e2e/fail/generic_bounds_not_satisfied.sifr)
- [crates/sifr/tests/e2e/fail/generic_counter_unhashable.sifr](crates/sifr/tests/e2e/fail/generic_counter_unhashable.sifr)
- [crates/sifr/tests/e2e/fail/generic_heapq_uncomparable.sifr](crates/sifr/tests/e2e/fail/generic_heapq_uncomparable.sifr)
- [crates/sifr/tests/e2e/fail/generic_wrong_type_arg.sifr](crates/sifr/tests/e2e/fail/generic_wrong_type_arg.sifr)
- [crates/sifr/tests/e2e/fail/protocol_bound_forwarding_non_conforming_typevar.sifr](crates/sifr/tests/e2e/fail/protocol_bound_forwarding_non_conforming_typevar.sifr)
- [crates/sifr/tests/e2e/fail/protocol_bound_unknown_forwarded_typevar.sifr](crates/sifr/tests/e2e/fail/protocol_bound_unknown_forwarded_typevar.sifr)
- [crates/sifr/tests/e2e/fail/stdlib_test_assert_gt_uncomparable.sifr](crates/sifr/tests/e2e/fail/stdlib_test_assert_gt_uncomparable.sifr)
- [crates/sifr/tests/e2e/fail/typevar_unknown_bound_rejected.sifr](crates/sifr/tests/e2e/fail/typevar_unknown_bound_rejected.sifr)

`grep` confirms exactly 8 `SIFR-PROTO-0001` markers under `crates/sifr/tests/e2e/`, matching the count.

### 8. Phase-tracker bookkeeping (confirmation)

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:54-55](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:54) flips 2b.19 from "implementation complete and reviewer-satisfied" to "merged" with PR link, and adds a new "2b.20 in progress: protocol-bound diagnostics migration to active `SIFR-PROTO-0001` with fixture coverage. PR: pending." entry. Both changes are exactly the wording the previous slices used and consistent with the rest of the checklist.

## Test coverage

- `concrete_type_missing_protocol_bound_has_proto_code` exercises the hot path: a class without `__lt__` passed to a `[T: Comparable]` function; asserts both exact message and `DiagnosticCode::PROTO_BOUND_NOT_SATISFIED`.
- `forwarded_typevar_missing_protocol_bound_has_proto_code` exercises the forwarded-TypeVar path: `relay_bad[U: Closable]` calling `take_readable[T: Readable](x)`, where `U` does not satisfy `Readable`. Same assertions.

These two tests, plus the eight e2e fail fixtures, give independent and high-fidelity coverage for the slice. I did not find a missing scenario worth adding in this pass.

## Ready for PR?

Yes — after the unrelated doc-comment deletion in [crates/sifr_hir/src/lower/mod.rs:109](crates/sifr_hir/src/lower/mod.rs:109) is reverted. The remaining items (#2 clone overhead, #3 test placement, #4 stale PROTO-0004 representative fixture) are non-blocking observations. None of them affects correctness, the diagnostic taxonomy contract, or test coverage for slice 2b.20.
