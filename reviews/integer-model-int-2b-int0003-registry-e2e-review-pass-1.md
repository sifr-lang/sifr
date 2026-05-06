# Review: INT-2B — `SIFR-INT-0003` registry placement and e2e fail fixture — Pass 1

Reviewer scope: review-only. No files modified.

Branch: `int-2b-int0003-registry-e2e`

Phase issue: [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md)
Design doc: [internal_docs/integer_model.md](internal_docs/integer_model.md)
Prior pass that scoped these follow-ups: [reviews/integer-model-int-2a-reserved-width-diagnostic-review-pass-2b.md](reviews/integer-model-int-2a-reserved-width-diagnostic-review-pass-2b.md) (N3, N5).

## Summary

The slice closes the two concrete follow-ups carried forward from INT-2A's pass-2b review:

- **N3** — Move the four active `INT` entries (`SIFR-INT-0001`, `0003`, `0004`, `0011`) so they sit after the last `DECIMAL` active entry and before the first `CALL` active entry, matching the family declaration order in [crates/sifr_diagnostics/src/codes.rs:386](crates/sifr_diagnostics/src/codes.rs:386) and the families summary table.
- **N5** — Add a negative `.sifr` e2e fixture at [crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr](crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr) and re-point `SIFR-INT-0003`'s `representative_fixture_path` from the unit test to that fixture.

The two regenerated reference docs and the per-code page are byte-consistent with the registry change, and the recorded local validation (gen-error-docs, `cargo test -p sifr_diagnostics`, `cargo test -p sifr --test e2e test_e2e_fail`, `scripts/run_all_tests.sh --profile quick`) passes.

No blockers.

## What was changed

### Registry ordering — [crates/sifr_diagnostics/src/codes.rs](crates/sifr_diagnostics/src/codes.rs)

The four active `INT` entries previously appeared in the registry between `SIFR-TYPE-0902` and `SIFR-DECIMAL-0001` (i.e. before the `DECIMAL` block). The diff moves them as a block to immediately after `SIFR-DECIMAL-0008` and immediately before `SIFR-CALL-0001`. The relative order within the block (`0001`, `0003`, `0004`, `0011`) is preserved and matches the declaration order of the `DiagnosticCode` constants at [codes.rs:62-65](crates/sifr_diagnostics/src/codes.rs:62) and the family declaration order at [codes.rs:386-403](crates/sifr_diagnostics/src/codes.rs:386).

The active entry sequence after the change is `PARSE → NAME → IMPORT → TYPE → DECIMAL → INT → CALL → OWN → FLOW → MATCH → PROTO → CLASS → RESULT → STDLIB → WORKSPACE → BUILD → INTERNAL`, which is exactly the `DIAGNOSTIC_FAMILIES` declaration order (with no active CODEGEN entries yet, as expected). This closes N3.

### Representative fixture pointer — `SIFR-INT-0003` only

The `active_entry!` macro's fifth slot is the `representative_fixture_path`. For `SIFR-INT-0003` it changes from `crates/sifr_hir/src/lower/type_alias_tests.rs::test_reserved_integer_width_annotations_have_int_code` to `crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr`. The other three INT entries (`0001`, `0004`, `0011`) intentionally keep their existing in-crate fixture pointers, which is correct because none of them have an e2e fixture today (`0001` and `0004` are HIR-internal const-folding tests and `0011` is a driver-surface warning). Scope of the fixture-pointer edit matches the scope of the new fixture file.

The unit test [crates/sifr_hir/src/lower/type_alias_tests.rs:113](crates/sifr_hir/src/lower/type_alias_tests.rs:113) (`test_reserved_integer_width_annotations_have_int_code`) and the nested-position variant at [type_alias_tests.rs:138](crates/sifr_hir/src/lower/type_alias_tests.rs:138) are unchanged and still run; they are strictly more thorough than the e2e fixture (cover `uint128` in addition to `int128`, both type-alias and function-parameter positions, and nested generics). Pointing the registry at the e2e fixture surfaces the user-facing emission path, while the unit tests retain their column-precise lock at `range_for_after(...)`. The change does not weaken coverage.

### New e2e fixture — [crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr](crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr)

```
# expect-error[col=12]: SIFR-INT-0003

def main():
    value: int128 = 0
```

- Marker syntax matches the harness parser at [crates/sifr/tests/e2e.rs:614-642](crates/sifr/tests/e2e.rs:614) (`# expect-error[col=<u32>]: <code>`).
- `col=12` is the 1-based column of the leading `i` in `int128` on line 4: four spaces of indent (1–4), `value` (5–9), `:` (10), space (11), `i` of `int128` (12). The diagnostic is emitted at [crates/sifr_hir/src/lower/typing_and_functions.rs:412-418](crates/sifr_hir/src/lower/typing_and_functions.rs:412) over the type-name range, so the harness's primary-span column comparison at [e2e.rs:867-868](crates/sifr/tests/e2e.rs:867) will see column 12. The reported `cargo test -p sifr --test e2e test_e2e_fail` pass (265 fail tests) confirms this.
- File name (`reserved_int128_annotation.sifr`) aligns with sibling fixture naming and accurately scopes what is asserted (`int128` in an annotation, not `uint128` or a nested context — those remain locked by the unit tests).
- Trailing newline present, no Unicode/whitespace anomalies.

The single-error / single-column shape avoids ambiguity with the harness's contradictory-marker detection at [e2e.rs:645-682](crates/sifr/tests/e2e.rs:645).

### Generated reference docs

Three doc files are byte-consistent with `cargo run -q -p sifr_diagnostics --bin gen-error-docs`:

- [docs/errors/SIFR-INT-0003.md:14](docs/errors/SIFR-INT-0003.md:14) — `Representative fixture` row now reads `crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr`. All other fields (template, owner, declared/dedupe args) unchanged.
- [docs/errors/diagnostic-codes.md:75-78](docs/errors/diagnostic-codes.md:75) — the four `SIFR-INT-*` rows now sit between the last `SIFR-DECIMAL-*` and first `SIFR-CALL-*` row, matching the `## Families` table order at [diagnostic-codes.md:13-30](docs/errors/diagnostic-codes.md:13).
- [internal_docs/diagnostic_codes.md:90-100](internal_docs/diagnostic_codes.md:90) — same row reordering, plus the `SIFR-INT-0003` row's `Fixture` column is updated to the new e2e path.

The `Reserved Codes` table at the bottom of `diagnostic-codes.md` and the family-base rows in `internal_docs/diagnostic_codes.md` are untouched, which matches what the regenerator should emit (no reserved entries changed).

## Correctness checks performed

1. **Registry order matches family declaration order.** Verified by listing all `active_entry!(...)` IDs in [codes.rs:404-1550](crates/sifr_diagnostics/src/codes.rs:404) and checking the family transitions land in the exact order of `DIAGNOSTIC_FAMILIES` at [codes.rs:386-403](crates/sifr_diagnostics/src/codes.rs:386). DECIMAL (`0001`–`0008`) immediately precedes INT (`0001`, `0003`, `0004`, `0011`) which immediately precedes CALL (`0001`–`0005`).

2. **Within-family order is unchanged.** The four INT entries appear in numeric order (`0001`, `0003`, `0004`, `0011`) — same as before the move, and same as the `DiagnosticCode` constants in [codes.rs:62-65](crates/sifr_diagnostics/src/codes.rs:62).

3. **Macro arity preserved.** The macro at [codes.rs:366-383](crates/sifr_diagnostics/src/codes.rs:366) takes a positional `fixture` literal in slot 5; the only field-value change for `SIFR-INT-0003` is the literal at that slot. Other entries' fixture literals are unchanged. The const-context invariants asserted at [codes.rs:1751-1755](crates/sifr_diagnostics/src/codes.rs:1751) (every active entry must have a representative fixture) and [codes.rs:1763-1767](crates/sifr_diagnostics/src/codes.rs:1763) (reserved entries must not) still hold.

4. **Fixture path is enforced to exist.** [scripts/check_diagnostic_code_coverage.py:111-129](scripts/check_diagnostic_code_coverage.py:111) splits the fixture string at `::` and checks `(ROOT / path_part).exists()`. The new file at `crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr` is present, satisfying that gate (which is part of the quick-profile validation lane). The previous unit-test path also still exists, so swapping does not break the `existence` check transitively for any other code.

5. **Column expectation lands on the diagnostic primary span.** The emit site at [typing_and_functions.rs:412-418](crates/sifr_hir/src/lower/typing_and_functions.rs:412) passes the type-name `range` directly. The unit test at [type_alias_tests.rs:122](crates/sifr_hir/src/lower/type_alias_tests.rs:122) already locks the start of `int128` as the primary range. Column 12 in the new fixture aligns with that emission point. Confirmed by the recorded `test_e2e_fail` pass (265 fail tests).

6. **Doc index files are sorted by registry order, not alphabetic.** [crates/sifr_diagnostics/src/bin/gen-error-docs.rs:163-173](crates/sifr_diagnostics/src/bin/gen-error-docs.rs:163) iterates `DIAGNOSTIC_REGISTRY` directly, so the table reordering in `diagnostic-codes.md` and `internal_docs/diagnostic_codes.md` is the deterministic consequence of the registry move — no hand-editing required, and the diff stays minimal (only INT rows shifted relative to DECIMAL/CALL).

## Validation

User-recorded quick-profile run is sufficient for the kind of change made:

- `cargo run -q -p sifr_diagnostics --bin gen-error-docs` — regenerates the three reference doc files; idempotent re-run confirms they match the registry.
- `cargo test -p sifr_diagnostics -- --nocapture` — exercises `assert_registry_strings_are_markdown_safe`, the active/reserved invariants, family-order consistency, and template/declared-arg checks.
- `cargo test -p sifr --test e2e test_e2e_fail -- --nocapture` — 265 fail fixtures including the new one; column expectation pass implies the `[col=12]` qualifier matches the reported primary span.
- `cargo fmt` — no formatting drift.
- `scripts/run_all_tests.sh --profile quick` — `report_signature=e1bf653aaa770517`, `wall_time=64.53s`. This profile runs the diagnostic schema, coverage (which includes the fixture-path-exists check), and baseline-hygiene gates that would catch a stale or missing fixture pointer.

Re-running the full profile is not required: the change touches no Rust source under HIR/codegen/driver, no fixture path string except the one whose target file is being created, and no schema-tracked file outside the regenerated docs.

## Coverage value of the new fixture

- It exercises the user-facing path end-to-end: parser → HIR lowering → diagnostic engine → CLI rendering. The unit tests assert internal `RenderedDiagnostic` shape but bypass the `sifr` CLI and the e2e harness's column-comparison machinery, so the fixture adds genuine coverage on top of the existing tests.
- The single-position assertion (`col=12` on `value: int128`) is a useful canonical anchor; broader coverage (`uint128`, type-alias position, nested generics) is intentionally left to the unit tests, which is the right division — e2e fixtures should be representative, not exhaustive.

## Closure of cited follow-ups

From the prior pass-2b review's non-blocking findings carried into INT-2B:

- **N3** (registry ordering) — closed.
- **N5** (e2e fail fixture) — closed.
- **N4** (reserved-name shadowing policy) — explicitly out of scope per the phase issue's INT-2B item ("decide reserved-name shadowing policy during `bigint` cleanup"); not touched here, and that is correct.

The phase issue checklist line at [issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:435](issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md:435) names exactly these follow-ups; this slice fully addresses the registry-placement and e2e-fixture portions.

## PR readiness

Self-contained, minimal, correct. Suitable for merge.

Suggested PR description points:

- N3/N5 cleanup from INT-2A pass-2b review.
- Move four `SIFR-INT-*` active entries between DECIMAL and CALL; regenerate three reference doc files.
- Add `crates/sifr/tests/e2e/fail/reserved_int128_annotation.sifr` (`col=12`); re-point `SIFR-INT-0003`'s representative fixture there.
- Quick-profile validation: `report_signature=e1bf653aaa770517`, `wall_time=64.53s`.

## Non-blocking observations (informational only — do not act on this slice)

- **Single-name e2e coverage.** The new fixture covers `int128` only. The diagnostic also fires for `uint128` from the same code path; a sibling fixture (or a second `expect-error` line in the same file) would round out e2e coverage to match the unit-test scope. Not required for this slice — fold into a future INT-2B touch if convenient. The unit tests in `type_alias_tests.rs` already lock both names.
- **CODEGEN family has no active entries.** The `DIAGNOSTIC_FAMILIES` order includes `CODEGEN` between `WORKSPACE` and `BUILD`, but no active CODEGEN entry exists yet. This is informational; gaps are allowed by the registry invariants.

VERDICT: SATISFIED
