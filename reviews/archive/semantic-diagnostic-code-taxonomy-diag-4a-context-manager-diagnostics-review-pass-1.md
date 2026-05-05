---
name: semantic-diagnostic-code-taxonomy-diag-4a-context-manager-diagnostics-review-pass-1
description: Review pass 1 — verify slice 2b.21 migrates the with-statement context-manager-missing diagnostic from the SIFR-TYPE-0001 bridge to active SIFR-PROTO-0003 end-to-end.
---

# Review — `milestone_diag_4a` slice 2b.21: context-manager protocol diagnostic

- Branch: `codex/semantic-diagnostics-diag-4a-protocol-surface-diagnostics`
- Scope: migrate the class-without-context-manager diagnostic emitted by `with`-statement lowering from the legacy `CompilePhase::TypeCheck` → `SIFR-TYPE-0001` transitional bridge to active `SIFR-PROTO-0003` via the existing `protocol_diagnostics` helper module.
- Pass: 1
- Prior related review: [reviews/semantic-diagnostic-code-taxonomy-diag-4a-protocol-bound-diagnostics-review-pass-2.md](reviews/semantic-diagnostic-code-taxonomy-diag-4a-protocol-bound-diagnostics-review-pass-2.md) (slice 2b.20, which introduced `protocol_diagnostics.rs`).

## Summary

The slice does what was advertised and stays scoped: it adds one helper (`protocol_diagnostics::context_manager_missing`) alongside the previously merged `bound_not_satisfied`, routes the single class-without-`__enter__`/`__exit__` call site in [crates/sifr_hir/src/lower/statements.rs:305](crates/sifr_hir/src/lower/statements.rs:305) through it with `DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING`, leaves the partial-protocol and non-class branches on raw `ctx.error(...)` (correct, per scope), updates the `SIFR-PROTO-0003` registry message template to match the actually-emitted message, regenerates `docs/errors/SIFR-PROTO-0003.md` and the `internal_docs/diagnostic_codes.md` row, re-keys the lone fixture from `SIFR-TYPE-0001` → `SIFR-PROTO-0003`, and adds a focused unit test that asserts both message and code.

The user-reported validation gates (`gen-error-docs`, `cargo fmt --check`, `check_diagnostic_docs_sync.py`, `check_diagnostic_schema_sync.py`, `check_hir_maintainability_guardrails.py`, `cargo test -p sifr_hir protocol_diagnostics`, `cargo test -p sifr_diagnostics`, `cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace -- -D warnings`) cover the surface this slice touches; nothing else is at risk of regressing. The phase tracker bookkeeping (2b.20 → merged with PR 1692, 2b.21 → in progress) is correct and matches the wording of prior slices.

I did not find any blockers. There is one minor readability nit (closure-parameter shadowing on a freshly-introduced binding) that's worth flagging but does not block merge, and a pair of pre-existing observations carried forward from slice 2b.20 that this slice does not need to resolve.

## Findings

### 1. Closure parameter shadows the newly-introduced outer `name` (minor — readability nit, not blocking)

[crates/sifr_hir/src/lower/statements.rs:296-298](crates/sifr_hir/src/lower/statements.rs:296):

```rust
let has_context_manager = if let Type::Class { name, methods, .. } = &val_ty {
    let has_enter = methods.iter().any(|(name, _)| name == "__enter__");
    let has_exit = methods.iter().any(|(name, _)| name == "__exit__");
```

The destructuring on line 296 binds the class `name: &String` so it can be passed to the helper at [statements.rs:305](crates/sifr_hir/src/lower/statements.rs:305). The closures on lines 297–298 then introduce their own `name` parameter (the method name from the `(String, FunctionType)` tuple), shadowing the outer one within their scope.

This compiles, behaves correctly, and passes clippy under the workspace's pedantic configuration ([Cargo.toml:70-91](Cargo.toml:70)) — `clippy::shadow_unrelated` lives in `restriction`, not `pedantic`, so the workspace does not flag it. The closures don't capture the outer `name` (they only read their own parameter), so there's no borrow conflict either.

The pre-edit form sidestepped the shadow because the outer pattern only bound `methods` (`Type::Class { methods, .. }`) and re-extracted `name` later via a redundant `match` (which had a dead `_ => "unknown".to_string()` arm — removing that fallback is a real, if small, win for this slice).

Two zero-risk follow-ups for a future pass; either is fine, neither is required:

1. Rename the closure parameters: `|(method_name, _)| method_name == "__enter__"` (and `"__exit__"`). Two lines, eliminates the shadow, makes the role of each name explicit.
2. Compute the names list once and search by membership: `let has_enter = methods.iter().any(|m| m.0 == "__enter__");` — drops the inner pattern destructuring entirely.

Not blocking; flag only because the shadowing is *introduced* by this slice (the outer `name` is new), not pre-existing.

### 2. Slice scope correctly excludes the partial-protocol and non-class branches (confirmation)

The `with`-statement lowering site at [statements.rs:296-312](crates/sifr_hir/src/lower/statements.rs:296) has three error paths:

| Branch | Trigger | Status after this slice |
| --- | --- | --- |
| Class with neither `__enter__` nor `__exit__` | line 304 `else` | **Migrated** to `SIFR-PROTO-0003` via `protocol_diagnostics::context_manager_missing` |
| Class with exactly one of `__enter__`/`__exit__` | line 301 `else if` | Unchanged — raw `ctx.error("type used in 'with' statement must implement both __enter__ and __exit__ methods")` |
| Non-class type used in `with` | line 308 outer `else` | Unchanged — raw `ctx.error("type used in 'with' statement must implement the ContextManager protocol (__enter__/__exit__)")` |

Both unchanged branches continue to flow through the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` transitional bridge in `sifr_driver`. This is consistent with the slice's stated narrow scope ("context-manager missing protocol diagnostic only") and matches the migration cadence the prior slices have followed (one structured emission per PR, leaving sibling branches on the bridge until they're individually migrated).

Note (not a finding for this slice): neither sibling branch has e2e fixture coverage today. `grep` confirms no fixture asserts either of those two messages anywhere under `crates/sifr/tests/`. This is a pre-existing gap that 2b.21 inherits but does not introduce. It will need attention before the SIFR-TYPE-0001 bridge can be deleted, but absorbing that work into 2b.21 would expand the slice beyond its declared scope.

### 3. Pre-existing template-vs-emission drift on `SIFR-PROTO-0003` corrected (positive observation)

The previous registry template for `SIFR-PROTO-0003` was `"type {type_name} is not a context manager"`, while the actual `ctx.error(...)` emission was `"type '{}' does not implement the ContextManager protocol (missing __enter__ and __exit__ methods)"`. These never matched — meaning the registry's template-string was effectively documentation-only and disagreed with what users actually saw in errors.

This slice updates the template, the generated `docs/errors/SIFR-PROTO-0003.md`, and the `internal_docs/diagnostic_codes.md` row to match the emitted message byte-for-byte. The helper format-string in [protocol_diagnostics.rs:23](crates/sifr_hir/src/lower/protocol_diagnostics.rs:23) and the registry template in [codes.rs:1001](crates/sifr_diagnostics/src/codes.rs:1001) are now identical (modulo `{type_name}` placeholder vs literal substitution). Drift between them would be caught by `test_e2e_fail` since the fixture's `expect-error` substring assertion exercises this exact path.

This is good cleanup, not scope creep — without a real call site, the previous template was only validated by the diagnostic-docs sync gates, and those just ensure registry ↔ docs alignment, not registry ↔ emission.

### 4. Registry / docs / internal_docs alignment (confirmation)

- [crates/sifr_diagnostics/src/codes.rs:80](crates/sifr_diagnostics/src/codes.rs:80): `DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING` already existed and is registered in the `DiagnosticCode::ALL` array at [codes.rs:1377](crates/sifr_diagnostics/src/codes.rs:1377). No new constant required.
- [crates/sifr_diagnostics/src/codes.rs:995-1005](crates/sifr_diagnostics/src/codes.rs:995): registry entry — message template now matches emission, `declared_args` and `dedupe_args` correctly remain `["type_name"]` (only one substitution placeholder), `representative_fixture_path` correctly continues to point at `with_non_context_manager.sifr` (the fixture is still the right exemplar after re-keying), `owner = "sifr_hir::lower::statements"` correctly identifies the call-site module.
- [docs/errors/SIFR-PROTO-0003.md:13](docs/errors/SIFR-PROTO-0003.md:13): regenerated message template matches.
- [internal_docs/diagnostic_codes.md:116](internal_docs/diagnostic_codes.md:116): row matches.

`grep -rn "ContextManager protocol"` confirms the message text exists in exactly four places — the helper format-string, the registry template, the unit-test assertion, and the fixture's `expect-error` line — with no other call sites to migrate.

### 5. Owner-field convention (confirmation)

`SIFR-PROTO-0003`'s registry owner is `sifr_hir::lower::statements`, pointing at the call site (`statements.rs`). The helper itself lives in `protocol_diagnostics.rs`. This matches the convention slice 2b.20 set for `SIFR-PROTO-0001`, whose owner is `sifr_hir::lower` (the parent module of the call site `expressions.rs`), and whose helper is in the same `protocol_diagnostics.rs`. The owner field is documenting *where the diagnostic is raised*, not where its formatter lives. Consistent.

### 6. Test placement still inline rather than sibling `_tests.rs` (carry-over note from slice 2b.20)

The pass-1 review of slice 2b.20 flagged that other migrated diagnostic helpers in this directory keep tests in a separate `_tests.rs` file (e.g., `match_diagnostics_tests.rs`, `name_import_diagnostics_tests.rs`, `own_mut_param_tests.rs`), while `protocol_diagnostics.rs` inlines `#[cfg(test)] mod tests`. The phase-tracker plan explicitly directed the inline placement at the time, so it was accepted as a deliberate convention split.

Slice 2b.21 adds a third test (`missing_context_manager_has_proto_code`) to the same inline `mod tests` block. The convention divergence persists. Splitting now would touch infrastructure unrelated to the slice's stated scope, so I would not block on it. If the next protocol-domain slice extracts a `protocol_diagnostics_tests.rs`, doing so for all three tests in one move is the cleanest path — but that's a future-slice concern.

### 7. Unit test coverage (confirmation)

The added test `missing_context_manager_has_proto_code` at [protocol_diagnostics.rs:68-79](crates/sifr_hir/src/lower/protocol_diagnostics.rs:68) exercises the migrated branch: a class with one field and no protocol methods (`class PlainClass:\n    value: int`) used in `with PlainClass(42) as p`. It asserts both:

- exact message string equality (catches any drift between the helper format-string and the expected text), and
- `error.code == Some(DiagnosticCode::PROTO_CONTEXT_MANAGER_MISSING)` (catches mis-routing through `ctx.error` instead of `ctx.error_with_code`).

The fixture body in the unit test (no explicit `__init__`) differs slightly from the e2e fixture body (explicit `__init__(self, value: int)`), but both paths exercise the same diagnostic — the unit test relies on Sifr's auto-init for the missing constructor. The user-reported `cargo test -p sifr_hir protocol_diagnostics` passes, confirming auto-init does not interfere with the expected emission.

The earlier two tests in the file (`concrete_type_missing_protocol_bound_has_proto_code`, `forwarded_typevar_missing_protocol_bound_has_proto_code`) remain unchanged.

### 8. Fixture re-key (confirmation)

Exactly one fixture's `expect-error` marker changes:

- [crates/sifr/tests/e2e/fail/with_non_context_manager.sifr:1](crates/sifr/tests/e2e/fail/with_non_context_manager.sifr:1) — `SIFR-TYPE-0001` → `SIFR-PROTO-0003`. Message text after the colon is unchanged.

`grep -rn "expect-error.*SIFR-PROTO-0003"` confirms this is the only fixture asserting `SIFR-PROTO-0003` in the tree. `grep -rn "expect-error.*SIFR-TYPE-0001.*context"` returns nothing — no leftover fixtures still routing through the bridge for this diagnostic.

### 9. Phase-tracker bookkeeping (confirmation)

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:55-56](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:55):

- 2b.20 flipped from "implementation complete and reviewer-satisfied" to "merged" with PR 1692 link — matches PR 1692's actual merge status (visible in `git log` as commit `4ccce9aa`).
- 2b.21 entry added: "in progress: context-manager protocol diagnostics migration to active `SIFR-PROTO-0003` with fixture coverage. PR: pending." — wording matches the prior in-progress entries (e.g., what 2b.20 looked like before merge).

Both lines follow the established checklist conventions exactly.

### 10. Maintainability guardrails (confirmation)

[scripts/check_hir_maintainability_guardrails.py:13-20](scripts/check_hir_maintainability_guardrails.py:13) caps `crates/sifr_hir/src/lower/statements.rs` at 2200 lines and `crates/sifr_hir/src/lower/mod.rs` at 1200 lines.

- `wc -l` on `statements.rs`: 2178 (no slice-touched delta — the migration removed 4 lines and added 1 in this file, plus 1 import, for a net −2; well below the cap).
- `wc -l` on `mod.rs`: 1200 (no change in this slice; the `mod protocol_diagnostics;` declaration was already merged in 2b.20).
- `wc -l` on the new `protocol_diagnostics.rs` body: 80 lines, no per-file cap.

The user-reported `python3 scripts/check_hir_maintainability_guardrails.py` PASS is consistent with this.

## Test coverage assessment

The slice's test coverage is appropriate for its scope: one new unit test that pins both message and code on the migrated branch, plus the re-keyed e2e fixture. Together with the user-reported gates, regression coverage is solid.

What's *not* covered is intentional and out of scope:

- The partial-`__enter__`/`__exit__` branch — still on `ctx.error`, no fixture, will need both a structured code (or reuse of `SIFR-PROTO-0003`) and a fixture in a future slice.
- The non-class type branch (e.g., `with 42 as x:`) — same situation.

Both gaps existed before this slice and are appropriate to defer until the bridge-deletion slice (or earlier dedicated slices) tackles them.

## Recommendation

**Ready to merge.** The slice is correctly scoped, end-to-end aligned (call site ↔ helper ↔ registry ↔ docs ↔ fixture ↔ unit test), and the only finding (#1, closure-parameter shadowing) is a non-blocking readability nit that can be cleaned up in a follow-up if desired.

If the author wants to absorb the closure-rename in this slice (it's a one-line touch), do so before opening the PR; otherwise it's fine to land as-is.
