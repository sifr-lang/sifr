---
name: semantic-diagnostic-code-taxonomy-diag-4a-protocol-signature-diagnostics-review-pass-1
description: Review pass 1 — verify slice 2b.22 migrates the iterator/reversible protocol return-signature diagnostics from the SIFR-TYPE-0001 bridge to active SIFR-PROTO-0002 end-to-end.
---

# Review — `milestone_diag_4a` slice 2b.22: iterator/reversible protocol return-signature diagnostics

- Branch: `codex/semantic-diagnostics-diag-4a-protocol-signature-diagnostics`
- Scope: migrate the three `__iter__` / `__next__` / `__reversed__` *return-signature* diagnostics emitted by class lowering from the legacy `CompilePhase::TypeCheck` → `SIFR-TYPE-0001` transitional bridge to active `SIFR-PROTO-0002` via the existing `protocol_diagnostics` helper module.
- Pass: 1
- Prior related reviews:
  - [reviews/semantic-diagnostic-code-taxonomy-diag-4a-context-manager-diagnostics-review-pass-1.md](reviews/semantic-diagnostic-code-taxonomy-diag-4a-context-manager-diagnostics-review-pass-1.md) (slice 2b.21, the immediately preceding `protocol_diagnostics` slice for `SIFR-PROTO-0003`).
  - [reviews/semantic-diagnostic-code-taxonomy-diag-4a-protocol-bound-diagnostics-review-pass-2.md](reviews/semantic-diagnostic-code-taxonomy-diag-4a-protocol-bound-diagnostics-review-pass-2.md) (slice 2b.20, which introduced `protocol_diagnostics.rs`).

## Summary

The slice does what was advertised and stays within the declared narrow scope. It adds one helper (`protocol_diagnostics::invalid_return_signature`) alongside the previously merged `bound_not_satisfied` (`SIFR-PROTO-0001`) and `context_manager_missing` (`SIFR-PROTO-0003`) helpers, routes the three return-signature call sites in [crates/sifr_hir/src/lower/classes.rs:122-158](crates/sifr_hir/src/lower/classes.rs:122) through it with `DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE`, leaves the sibling parameter-shape and protocol-mismatch branches on raw `ctx.error(...)` (correct, per scope), corrects the pre-existing template-vs-emission drift on `SIFR-PROTO-0002` (matching the same kind of fixup slice 2b.21 made for `SIFR-PROTO-0003`), regenerates `docs/errors/SIFR-PROTO-0002.md` and the `internal_docs/diagnostic_codes.md` row, retargets the registry's representative fixture from the unrelated `reversed_iterator_not_reversible.sifr` to a fixture that actually exercises this code, re-keys the three `invalid_*_signature.sifr` fixtures from `SIFR-TYPE-0001` → `SIFR-PROTO-0002`, and adds three focused unit tests that assert both message and code identity.

The user-reported validation gates (`gen-error-docs`, `cargo fmt --check`, `check_diagnostic_docs_sync.py`, `check_diagnostic_schema_sync.py`, `check_hir_maintainability_guardrails.py`, `cargo test -p sifr_hir protocol_diagnostics`, `cargo test -p sifr_diagnostics`, `cargo test -p sifr --test e2e -- test_e2e_fail`, `cargo test -p sifr -- --skip test_e2e_pass`, `cargo clippy --workspace -- -D warnings`) cover the surface this slice touches; nothing else is at risk of regressing. The phase tracker bookkeeping (2b.21 → merged with PR 1693, 2b.22 → in progress, PR pending) is correct and matches the wording of prior slices.

I did not find any blockers. There is one minor naming nit on the new helper, plus a pair of pre-existing observations carried forward from earlier slices that this slice does not need to resolve. Findings 3 and 4 are also positive observations worth recording.

## Findings

### 1. Helper name is slightly more generic than the diagnostic code it emits (minor — readability nit, not blocking)

[crates/sifr_hir/src/lower/protocol_diagnostics.rs:28-33](crates/sifr_hir/src/lower/protocol_diagnostics.rs:28):

```rust
pub(super) fn invalid_return_signature(ctx: &mut LowerCtx, type_name: &str, expected: &str) {
    ctx.error_with_code(
        DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE,
        format!("class '{type_name}' must return {expected}"),
    );
}
```

The constant is `PROTO_INVALID_ITERATOR_SIGNATURE` and the registry description is "Invalid iterator or reversible protocol signature." (`SIFR-PROTO-0002`), but the helper is named just `invalid_return_signature` — generic enough that a future caller in a different protocol family (e.g., `__hash__`, `__lt__`) could reach for it and end up emitting an iterator-flavored code by accident. The two sibling helpers in this same module are domain-scoped (`bound_not_satisfied`, `context_manager_missing`); this one is the odd one out.

Also, the helper currently formats the entire message with `class '{type_name}' must return {expected}` baked in. That phrasing is fine for these three callers but isn't generalizable — it already locks the helper to the "method-on-a-class returned the wrong type" shape.

Not blocking, two zero-risk follow-ups for a future pass; either is fine, neither is required:

1. Rename to `iterator_invalid_return_signature` (or `iter_protocol_invalid_return_signature`) so the helper name and the diagnostic code agree. One-line rename, three call-site updates.
2. Leave the name and add a doc-comment naming the targeted protocols, mirroring the description text on the registry entry.

### 2. Slice scope correctly excludes the parameter-shape and protocol-mismatch branches (confirmation)

The `validate_iteration_protocol_methods` function at [classes.rs:111-188](crates/sifr_hir/src/lower/classes.rs:111) emits five distinct error categories. After this slice:

| Branch | Site | Status |
| --- | --- | --- |
| `__iter__` return type is not `Iterator[T]` / `Iterable[T]` / `Self` | [classes.rs:122-128](crates/sifr_hir/src/lower/classes.rs:122) | **Migrated** to `SIFR-PROTO-0002` |
| `__next__` return type is not `T \| None` | [classes.rs:137-143](crates/sifr_hir/src/lower/classes.rs:137) | **Migrated** to `SIFR-PROTO-0002` |
| `__reversed__` return type is not `Iterator[T]` / `Iterable[T]` / `Self` | [classes.rs:152-158](crates/sifr_hir/src/lower/classes.rs:152) | **Migrated** to `SIFR-PROTO-0002` |
| `__iter__` / `__next__` / `__reversed__` declare extra params besides `self` | [classes.rs:117-120](crates/sifr_hir/src/lower/classes.rs:117), [132-135](crates/sifr_hir/src/lower/classes.rs:132), [147-150](crates/sifr_hir/src/lower/classes.rs:147) | Unchanged — raw `ctx.error("class '{class_name}.<dunder>' must not declare parameters besides self")`, still routes through the `CompilePhase::TypeCheck` → `SIFR-TYPE-0001` bridge |
| `__iter__` element type disagrees with `__next__` / `__reversed__` element type | [classes.rs:161-187](crates/sifr_hir/src/lower/classes.rs:161) | Unchanged — raw `ctx.error("class '{class_name}' iteration protocol mismatch: …")`, still on the bridge |

This split matches the slice's stated narrow scope ("__iter__, __next__, and __reversed__ return signature diagnostics") and continues the migration cadence the prior slices have followed (one structured emission cluster per PR).

Note (not a finding for this slice): the unmigrated branches lack e2e fixture coverage today. `grep -rn "must not declare parameters besides self\|iteration protocol mismatch"` against `crates/sifr/tests/` returns no fixtures. This is a pre-existing gap that 2b.22 inherits but does not introduce. It will need attention before the `SIFR-TYPE-0001` bridge can be deleted (the slice 2b.21 review noted the analogous gap for the with-statement sibling branches), but absorbing that work into 2b.22 would expand the slice beyond its declared scope.

### 3. Pre-existing template-vs-emission drift on `SIFR-PROTO-0002` corrected (positive observation)

The previous registry template for `SIFR-PROTO-0002` was `"invalid {protocol} protocol signature for {type_name}"`, while the actual `ctx.error(...)` emissions in `validate_iteration_protocol_methods` were the three concrete `class '{name}.<dunder>' must return …` forms. These never matched — meaning the registry's template-string was effectively documentation-only and disagreed with what users actually saw in errors. The previous representative fixture was also wrong: `crates/sifr/tests/e2e/fail/reversed_iterator_not_reversible.sifr` actually emits `SIFR-TYPE-0001: reversed() argument must be reversible, got 'Iterator[int]'` (a built-in `reversed()` call-site type-check failure, unrelated to user-class signature validation), so the registry was citing a fixture that did not exercise the diagnostic at all.

This slice updates the template to `"class '{type_name}' must return {expected}"`, retargets the representative fixture to `crates/sifr/tests/e2e/fail/invalid_iter_signature.sifr` (which now actually emits `SIFR-PROTO-0002`), and regenerates `docs/errors/SIFR-PROTO-0002.md` and the `internal_docs/diagnostic_codes.md` row to match. The helper format-string in [protocol_diagnostics.rs:31](crates/sifr_hir/src/lower/protocol_diagnostics.rs:31) and the registry template at [codes.rs:990](crates/sifr_diagnostics/src/codes.rs:990) are now byte-identical (modulo the placeholder substitution). Drift between them would now be caught by `test_e2e_fail` since the fixture's `expect-error` substring assertion exercises the exact path.

This is good cleanup, not scope creep. It's the same kind of fixup slice 2b.21 made for `SIFR-PROTO-0003` — without a real call site, the previous template was only validated by the diagnostic-docs sync gates, and those just ensure registry ↔ docs alignment, not registry ↔ emission.

### 4. Registry / docs / internal_docs alignment (confirmation)

- [crates/sifr_diagnostics/src/codes.rs:78-79](crates/sifr_diagnostics/src/codes.rs:78): `DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE` already existed and is registered in the `DIAGNOSTIC_CODES` array at [codes.rs:1376](crates/sifr_diagnostics/src/codes.rs:1376). No new constant required.
- [crates/sifr_diagnostics/src/codes.rs:984-994](crates/sifr_diagnostics/src/codes.rs:984): registry entry — message template now matches emission, `declared_args` and `dedupe_args` correctly updated to `["type_name", "expected"]` (matching the two new placeholders), `representative_fixture_path` correctly retargeted to `crates/sifr/tests/e2e/fail/invalid_iter_signature.sifr`, `owner = "sifr_hir::lower::classes"` correctly identifies the call-site module (consistent with the convention slice 2b.21 followed for `SIFR-PROTO-0003` → `sifr_hir::lower::statements`).
- [docs/errors/SIFR-PROTO-0002.md:13-16](docs/errors/SIFR-PROTO-0002.md:13): regenerated message template, fixture path, declared args, and dedupe args all match.
- [internal_docs/diagnostic_codes.md:115](internal_docs/diagnostic_codes.md:115): row matches.

`grep -rn "must return 'Iterator\|must return 'T | None'"` against `--include="*.rs"`/`--include="*.md"`/`--include="*.sifr"` confirms the message texts exist in exactly these places — the helper format-string (one parameterized site producing all three messages), the registry template, the three `protocol_diagnostics.rs` unit-test assertions, the three `expressions_tests.rs` substring assertions (pre-existing — see finding 6), and the three `expect-error` lines in the re-keyed e2e fixtures — with no other call sites to migrate.

### 5. End-to-end wiring verified (confirmation)

The `LoweringError → CompileError → CompilerDiagnostic` pipeline preserves the structured code:

- [crates/sifr_hir/src/lower/mod.rs:237-244](crates/sifr_hir/src/lower/mod.rs:237): `error_with_code` records the structured `code: Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)` on the `LoweringError`.
- [crates/sifr_driver/src/frontend/module_lowering.rs:47-51](crates/sifr_driver/src/frontend/module_lowering.rs:47): when `error.code` is `Some`, it calls `CompileError::with_code(message, CompilePhase::TypeCheck, code)`, which sets `code: Some(code)`.
- [crates/sifr_driver/src/diagnostics.rs:125-141](crates/sifr_driver/src/diagnostics.rs:125): `diagnostic_code()` returns `code.code()` (i.e., `"SIFR-PROTO-0002"`) when `Some(code)` is present, only falling back to the phase-based `"SIFR-TYPE-0001"` bridge when `code.is_none()`.

Result: the diagnostic surfaces as `SIFR-PROTO-0002` end-to-end. The e2e fixture assertion (`failure.code == expected.code` with `expected.code == "SIFR-PROTO-0002"`) and the unit-test assertion (`error.code == Some(DiagnosticCode::PROTO_INVALID_ITERATOR_SIGNATURE)`) both gate this; either failing would surface immediately under `cargo test -p sifr --test e2e -- test_e2e_fail` or `cargo test -p sifr_hir protocol_diagnostics`.

I also walked the call-graph: `validate_iteration_protocol_methods` is reached only via the *second* `collect_class_type` pass at [crates/sifr_hir/src/lower/mod.rs:614](crates/sifr_hir/src/lower/mod.rs:614) (the first pass at [mod.rs:604](crates/sifr_hir/src/lower/mod.rs:604) is invoked with `validate_iteration_protocols = false`), so each diagnostic fires exactly once per offending class — no double-reporting risk introduced by the two-pass class lowering. This is unchanged by the slice but worth confirming because the migration changes how the message is materialized.

### 6. Unit-test coverage is complete; pre-existing message-only tests still pass (confirmation)

The new tests in [protocol_diagnostics.rs:88-122](crates/sifr_hir/src/lower/protocol_diagnostics.rs:88) assert both the exact message and the exact `DiagnosticCode` for all three dunders (`__iter__`, `__next__`, `__reversed__`). This is the structured-identity gate: a future regression that left the message alone but stripped the code (or vice versa) would fail these tests.

The pre-existing tests in [expressions_tests.rs:1227-1249](crates/sifr_hir/src/lower/expressions_tests.rs:1227) only assert message-substring equality, not code identity. They continue to pass under the migration (the message text is unchanged) and serve as additional regression coverage for the message wording. They do not gate code identity, but the new `protocol_diagnostics.rs` tests do, so there is no coverage gap.

There is no pre-existing unit test for `__reversed__` invalid-signature in `expressions_tests.rs` — the new `invalid_reversed_signature_has_proto_code` test in `protocol_diagnostics.rs` is the only structured unit-test coverage for that branch. The e2e fixture covers the integration path. Acceptable.

### 7. The `expected` argument value embeds literal single-quotes (minor — structured-arg styling, not blocking)

The three call sites pass `expected = "'Iterator[T]' or 'Iterable[T]'"` and `expected = "'T | None'"` — i.e., the single-quotes are part of the argument value, not part of the registry template (which is `class '{type_name}' must return {expected}`, with no quotes around `{expected}`). When the structured-argument schema is consumed downstream (LSP, IDE clients), the `expected` field will deserialize as `"'Iterator[T]' or 'Iterable[T]'"` rather than the bare `"Iterator[T] or Iterable[T]"` an IDE might prefer to render with its own quoting convention.

For comparison, the analogous `expected` argument on `SIFR-TYPE-0002` (`type mismatch: expected {expected}, got {actual}`) and on `SIFR-STDLIB-0002` (`invalid argument type for {symbol}: expected {expected}, got {actual}`) typically receives bare type-display strings (e.g., `"int"`, `"str | None"`) without embedded apostrophes. The format-string conventions diverge — this slice preserves the pre-existing rendered-text exactly, which is correct for fixture compatibility, but it does mean `SIFR-PROTO-0002`'s structured `expected` will look noisier than the type-mismatch family.

Not blocking. Two ways to address in a future pass if/when structured-arg consumers care:

1. Hoist the quotes into the registry template (`class '{type_name}' must return '{expected}'`) and pass `Iterator[T]` / `Iterable[T]` / `T | None` bare. This is technically a user-visible change because the `or` between `'Iterator[T]'` and `'Iterable[T]'` would now sit between two un-quoted type names — a different rendering — so it's not a free swap.
2. Leave it alone and document in the schema generator that `SIFR-PROTO-0002`'s `expected` arg is a free-form prose phrase, not a single type name.

Worth noting only because the slice is the right time to reconsider; there is no functional defect.

### 8. Owner-field convention (confirmation)

`SIFR-PROTO-0002`'s registry owner is now `sifr_hir::lower::classes`, pointing at the call site (`classes.rs`). The helper itself lives in `protocol_diagnostics.rs`. This matches the convention slice 2b.20 set for `SIFR-PROTO-0001` (owner: `sifr_hir::lower`, helper: `protocol_diagnostics.rs`) and slice 2b.21 set for `SIFR-PROTO-0003` (owner: `sifr_hir::lower::statements`, helper: `protocol_diagnostics.rs`). The owner field is documenting *where the diagnostic is raised*, not where its formatter lives. Consistent.

### 9. Phase tracker bookkeeping (confirmation)

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:55-57](issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:55) correctly:

- Marks slice 2b.21 as `[x] merged` with PR 1693 (verified against the recent commit `683e07f8 Migrate context manager diagnostic code (#1693)`).
- Adds slice 2b.22 as `[ ]` in-progress with `PR: pending`, matching the wording style used by previous in-progress slices.

No drift from the merged-PR record in `git log`.

## Verification I performed

- Read the diff against `main` for all nine changed files via `git diff` (no stale or orphaned hunks).
- Confirmed the helper format-string and registry template are byte-identical modulo placeholder substitution.
- Walked the `LoweringError → CompileError → CompilerDiagnostic` pipeline to verify the `Some(DiagnosticCode)` branch is taken and the code identity is preserved.
- Confirmed the call-graph: `validate_iteration_protocol_methods` is reached only via the second class-lowering pass, so diagnostics fire exactly once per offending class.
- Walked the three `class_*_element_type` helpers to confirm the conditions under which `is_none()` is true (and thus the helper is invoked) match the messages emitted.
- Cross-checked all places where the three migrated message texts appear in the repo — the helper format-string, the registry template, the three new unit tests, the three pre-existing message-substring tests in `expressions_tests.rs`, and the three e2e fixtures — to confirm no orphaned occurrences.
- Cross-checked the *previous* representative fixture (`reversed_iterator_not_reversible.sifr`) to confirm it does not actually emit `SIFR-PROTO-0002` (it emits `SIFR-TYPE-0001` for a different code path), validating that the retarget is a fix, not a downgrade.
- Confirmed the slice does not introduce any other call site that should be on `SIFR-PROTO-0002` but isn't, and does not migrate any call site that should remain elsewhere.
- Confirmed `DIAGNOSTIC_CODES` array membership for `PROTO_INVALID_ITERATOR_SIGNATURE` at `codes.rs:1376` — no orphan-code risk for schema generation.
- Did not re-run the validation gates the user already ran; relied on the user's report that `gen-error-docs`, `cargo fmt --check`, both `check_diagnostic_*_sync.py` scripts, `check_hir_maintainability_guardrails.py`, the targeted Cargo test invocations, and `cargo clippy --workspace -- -D warnings` all passed.

## Recommendation

Mergeable as-is. None of the findings are blockers; all are either confirmations, positive observations of pre-existing-drift cleanup, or minor stylistic notes that can be addressed in future passes without holding up this slice. The slice is correctly scoped, end-to-end wiring is preserved, and the structured-identity tests gate against future regressions.
