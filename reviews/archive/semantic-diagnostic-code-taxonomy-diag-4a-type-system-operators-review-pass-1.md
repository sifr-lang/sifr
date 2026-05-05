---
name: Slice 2b.2 Type-System Operator Diagnostic Migration — Pass 1 Review
description: Reviews the slice 2b.2 migration of `crates/sifr_type_system::check` operator diagnostics to active `SIFR-TYPE-0005` / `SIFR-TYPE-0006` and the four affected fail-fixture re-keys; reviewer is satisfied with no blocking findings.
type: review
---

# Review: milestone_diag_4a — Slice 2b.2 Type-System Operator Diagnostic Migration (Pass 1)

Branch: `codex/semantic-diagnostics-diag-4a-type-system` (working tree on top of `053def1f`, the slice 2b.1 merge)
Issue: [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md)
Prior reviews referenced for context:
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-slice2-preimplementation-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-slice2a-transport-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-slice2a-transport-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-slice2a-transport-review-pass-2.md](semantic-diagnostic-code-taxonomy-diag-4a-slice2a-transport-review-pass-2.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-decimal-review-pass-1.md](semantic-diagnostic-code-taxonomy-diag-4a-decimal-review-pass-1.md)
- [reviews/semantic-diagnostic-code-taxonomy-diag-4a-decimal-review-pass-2.md](semantic-diagnostic-code-taxonomy-diag-4a-decimal-review-pass-2.md)

## Verdict

**Reviewer is satisfied. Slice 2b.2 has no blocking findings and is ready to ship.** Every operator-error site in [crates/sifr_type_system/src/check.rs](../crates/sifr_type_system/src/check.rs) now carries an active `DiagnosticCode`; the four fail fixtures whose `expect-error` lines previously rendered through the `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge are re-keyed to the active `SIFR-TYPE-0005` / `SIFR-TYPE-0006` codes that match their underlying message; no out-of-scope domains were touched; the issue tracker correctly marks slice 2b.1 merged and slice 2b.2 started; and the user's local validation matrix is sufficient for the surface area of the change.

I have only two non-blocking observations: **N1** (informational) — one site in the comparison `==`/`!=` fallthrough is mapped to `SIFR-TYPE-0002` (`TYPE_MISMATCH`) rather than `SIFR-TYPE-0005`, which is technically broader than the "0005 / 0006 only" framing in the task description but matches the inventory mapping at [internal_docs/diagnostic_emission_inventory.md:69](../internal_docs/diagnostic_emission_inventory.md:69) and is needed to honor the slice's "no `code: None` left in `check.rs`" hygiene goal; and **N2** (suggestion) — only one of the three new test cases pins the diagnostic code per scenario, leaving the symmetric `is_err()` lines unverified for code-identity. Neither is a blocker.

## Scope of pass 1

This review covers the working tree against `053def1f` (the slice 2b.1 merge). Six files are modified per `git status`:

| File | Lines | Surface |
| --- | --- | --- |
| [crates/sifr_type_system/src/check.rs](../crates/sifr_type_system/src/check.rs) | +51 / −30 | All `code: None` operator emissions converted to active codes; three test functions upgraded to assert codes |
| [crates/sifr/tests/e2e/fail/bigint_int_mixed_arithmetic.sifr](../crates/sifr/tests/e2e/fail/bigint_int_mixed_arithmetic.sifr) | ±1 | `expect-error` re-keyed `SIFR-TYPE-0001` → `SIFR-TYPE-0006` |
| [crates/sifr/tests/e2e/fail/bigint_int_mixed_comparison.sifr](../crates/sifr/tests/e2e/fail/bigint_int_mixed_comparison.sifr) | ±1 | `expect-error` re-keyed `SIFR-TYPE-0001` → `SIFR-TYPE-0006` |
| [crates/sifr/tests/e2e/fail/optional_arithmetic_reachable_after_partial_narrowing.sifr](../crates/sifr/tests/e2e/fail/optional_arithmetic_reachable_after_partial_narrowing.sifr) | ±1 | `expect-error` re-keyed `SIFR-TYPE-0001` → `SIFR-TYPE-0005` |
| [crates/sifr/tests/e2e/fail/optional_arithmetic_without_narrowing.sifr](../crates/sifr/tests/e2e/fail/optional_arithmetic_without_narrowing.sifr) | ±1 | `expect-error` re-keyed `SIFR-TYPE-0001` → `SIFR-TYPE-0005` |
| [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md) | +2 / −1 | Slice 2b.1 marked merged; slice 2b.2 added as started |

I performed five checks:

1. Confirmed the active-code identity of `DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR` and `DiagnosticCode::TYPE_INT_BIGINT_MIXED` against [crates/sifr_diagnostics/src/codes.rs:32-33](../crates/sifr_diagnostics/src/codes.rs:32) — they resolve to `SIFR-TYPE-0005` and `SIFR-TYPE-0006` respectively, with `Severity::Error`.
2. Read [crates/sifr_type_system/src/check.rs](../crates/sifr_type_system/src/check.rs) end-to-end and walked every `Err(TypeError { code: ..., ... })` construction, classifying each as either `TYPE_UNSUPPORTED_OPERATOR`, `TYPE_INT_BIGINT_MIXED`, or `TYPE_MISMATCH`, then cross-checked the code against the message text and the surrounding control flow.
3. Confirmed the slice's "no `code: None` remains" claim with `grep -rn "code:\s*None" crates/sifr_type_system/` — zero hits.
4. Confirmed the slice's "no SIFR-TYPE-0001 fixtures remain on operator messages" claim by greping `expect-error.*SIFR-TYPE-0001:.*(unsupported|cannot mix|cannot compare|bad operand)` across `crates/sifr/tests/e2e/fail/` and verifying the only match is the non-operator `unsupported_default_expr_call.sifr` ("unsupported default argument expression …", correctly deferred).
5. Searched for cross-references to the four migrated fixtures in the rest of the tree (verification baselines, snapshot files, internal docs, registry entries) to confirm nothing else needed re-keying alongside them.

## Code-identity verification

`SIFR-TYPE-0005` and `SIFR-TYPE-0006` are both declared as **active** entries in the registry:

- [crates/sifr_diagnostics/src/codes.rs:32](../crates/sifr_diagnostics/src/codes.rs:32) — `pub const TYPE_UNSUPPORTED_OPERATOR: Self = Self::new("SIFR-TYPE-0005", Severity::Error);`
- [crates/sifr_diagnostics/src/codes.rs:33](../crates/sifr_diagnostics/src/codes.rs:33) — `pub const TYPE_INT_BIGINT_MIXED: Self = Self::new("SIFR-TYPE-0006", Severity::Error);`

Both also appear in the active-code list at [crates/sifr_diagnostics/src/codes.rs:1314-1318](../crates/sifr_diagnostics/src/codes.rs:1314) and have full `active_entry!` registry rows at [crates/sifr_diagnostics/src/codes.rs:603-624](../crates/sifr_diagnostics/src/codes.rs:603) (with the migrated fixtures `optional_arithmetic_without_narrowing.sifr` and `bigint_int_mixed_arithmetic.sifr` listed as their representative fixtures). Doc pages [docs/errors/SIFR-TYPE-0005.md](../docs/errors/SIFR-TYPE-0005.md) and [docs/errors/SIFR-TYPE-0006.md](../docs/errors/SIFR-TYPE-0006.md) exist. So the identities the slice is migrating to are real, active, and matched with the same fixtures the slice re-keys.

## Mapping correctness in `check.rs`

I walked every `Err(TypeError { ... })` site that previously held `code: None` and classified each:

### `type_check_binary_op`

| Line | Branch | Trigger | New code | Verdict |
| --- | --- | --- | --- | --- |
| [62](../crates/sifr_type_system/src/check.rs:62) | int↔bigint mixed (non-pow) | `(Int op BigInt) || (BigInt op Int)` excluding `bigint ** int` | `TYPE_INT_BIGINT_MIXED` | ✓ correct — message says "cannot mix 'int' and 'bigint' in arithmetic", which is the exact 0006 contract |
| [128](../crates/sifr_type_system/src/check.rs:128) | `+` fallthrough | unsupported operands for `+` | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |
| [190](../crates/sifr_type_system/src/check.rs:190) | `-` / `*` fallthrough | unsupported operands for `-`/`*` (covers exhausted str/list/bytes repetition cases too) | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |
| [229](../crates/sifr_type_system/src/check.rs:229) | `/` fallthrough | unsupported operands for `/` | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |
| [271](../crates/sifr_type_system/src/check.rs:271) | `//` / `%` fallthrough | unsupported operands for `//`/`%` | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |
| [303](../crates/sifr_type_system/src/check.rs:303) | `**` fallthrough | unsupported operands for `**` | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |
| [325](../crates/sifr_type_system/src/check.rs:325) | `&` / `\|` / `^` fallthrough | bitwise ops on non-int/non-bool operands | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |
| [343](../crates/sifr_type_system/src/check.rs:343) | `<<` / `>>` fallthrough | shift ops on non-int operands | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |
| [356](../crates/sifr_type_system/src/check.rs:356) | unknown binary operator | `op` not matching any arm | `TYPE_UNSUPPORTED_OPERATOR` | ✓ — defensible default for "unknown operator" since the user-visible failure is still "this operator + these operands aren't supported"; an "internal-only" alternative is unnecessary because the only practical reach for this arm is a future operator the type checker hasn't been taught yet |

The decimal-family guard at line 32 (`DECIMAL_MIXED_WITH_BIGDECIMAL`) and the float↔decimal-family guard at line 45 (`DECIMAL_FLOAT_MIXED`) were already migrated by slice 2b.1 and remain unchanged; the diff does not perturb them. ✓

### `type_check_comparison`

| Line | Branch | Trigger | New code | Verdict |
| --- | --- | --- | --- | --- |
| [402](../crates/sifr_type_system/src/check.rs:402) | `==` / `!=` int↔bigint guard | `(Int == BigInt) \|\| (BigInt == Int)` | `TYPE_INT_BIGINT_MIXED` | ✓ |
| [440](../crates/sifr_type_system/src/check.rs:440) | `==` / `!=` fallthrough | "cannot compare 'X' and 'Y' with `==`" after equality_comparable + union exhaustion | `TYPE_MISMATCH` (`SIFR-TYPE-0002`) | ⚠ See N1 below — correct per inventory but technically outside slice's stated "0005 / 0006 only" framing |
| [458](../crates/sifr_type_system/src/check.rs:458) | `<` / `>` / `<=` / `>=` int↔bigint guard | `(Int < BigInt) \|\| (BigInt < Int)` | `TYPE_INT_BIGINT_MIXED` | ✓ |
| [491](../crates/sifr_type_system/src/check.rs:491) | ordering fallthrough | "'op' not supported between instances of 'X' and 'Y'" | `TYPE_UNSUPPORTED_OPERATOR` | ✓ — the message text is the canonical "operator unsupported on these operands" phrasing |
| [504](../crates/sifr_type_system/src/check.rs:504) | unknown comparison operator | unrecognized `op` in comparison | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |

Decimal-family guards at lines 372 and 386 (`DECIMAL_MIXED_WITH_BIGDECIMAL`, `DECIMAL_FLOAT_MIXED`) are unchanged from slice 2b.1. ✓

### `type_check_unary_op`

| Line | Branch | Trigger | New code | Verdict |
| --- | --- | --- | --- | --- |
| [554](../crates/sifr_type_system/src/check.rs:554) | unary `-` / `+` fallthrough | non-numeric operand | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |
| [587](../crates/sifr_type_system/src/check.rs:587) | unary `not` fallthrough | operand without truthiness support | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |
| [604](../crates/sifr_type_system/src/check.rs:604) | unary `~` fallthrough | non-int/non-bool operand | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |
| [613](../crates/sifr_type_system/src/check.rs:613) | unknown unary operator | unrecognized `op` | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |

### `type_check_bool_op`

| Line | Branch | Trigger | New code | Verdict |
| --- | --- | --- | --- | --- |
| [650](../crates/sifr_type_system/src/check.rs:650) | `and` / `or` fallthrough | one or both operands lack truthiness | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |
| [663](../crates/sifr_type_system/src/check.rs:663) | unknown boolean operator | unrecognized `op` | `TYPE_UNSUPPORTED_OPERATOR` | ✓ |

### Hygiene check

`grep -rn "code:\s*None" crates/sifr_type_system/` returns zero hits. The user's claim is verified: no `code: None` remains in [crates/sifr_type_system/src/check.rs](../crates/sifr_type_system/src/check.rs) (and the rest of the crate is unchanged). The other type-system files (`infer.rs`, `literal.rs`, `narrow.rs`, `union.rs`, `types.rs`) do not construct `TypeError { ... }` at all — `check.rs` is the sole emission site, so the file-level claim and the crate-level claim collapse to the same fact. ✓

## Fixture re-key correctness

For each migrated fixture, I confirmed (a) the new code matches the active code that the corresponding `check.rs` emission site now reports, and (b) the message text is exactly preserved.

| Fixture | Old code | New code | Code path triggered | Match |
| --- | --- | --- | --- | --- |
| [bigint_int_mixed_arithmetic.sifr](../crates/sifr/tests/e2e/fail/bigint_int_mixed_arithmetic.sifr) | `SIFR-TYPE-0001` | `SIFR-TYPE-0006` | `type_check_binary_op` int↔bigint guard at [check.rs:62](../crates/sifr_type_system/src/check.rs:62) | ✓ |
| [bigint_int_mixed_comparison.sifr](../crates/sifr/tests/e2e/fail/bigint_int_mixed_comparison.sifr) | `SIFR-TYPE-0001` | `SIFR-TYPE-0006` | `type_check_comparison` `==`/`!=` int↔bigint guard at [check.rs:402](../crates/sifr_type_system/src/check.rs:402) (and analogously [check.rs:458](../crates/sifr_type_system/src/check.rs:458) for ordering) | ✓ |
| [optional_arithmetic_reachable_after_partial_narrowing.sifr](../crates/sifr/tests/e2e/fail/optional_arithmetic_reachable_after_partial_narrowing.sifr) | `SIFR-TYPE-0001` | `SIFR-TYPE-0005` | `type_check_binary_op` `+` fallthrough at [check.rs:128](../crates/sifr_type_system/src/check.rs:128) (operand `'None \| int'` is neither numeric nor a TypeVar; the union does not collapse) | ✓ |
| [optional_arithmetic_without_narrowing.sifr](../crates/sifr/tests/e2e/fail/optional_arithmetic_without_narrowing.sifr) | `SIFR-TYPE-0001` | `SIFR-TYPE-0005` | same path as above | ✓ |

The fixture-message strings (`cannot mix 'int' and 'bigint' in arithmetic; …`, `cannot compare 'int' and 'bigint'; …`, `unsupported operand type(s) for +: 'None | int' and 'int'`) are byte-for-byte identical to the corresponding `format!(...)` strings in `check.rs`, so the e2e harness's `expect-error` matcher will hit on both code-and-message exactly.

### Coverage of operator-message fixtures on `SIFR-TYPE-0001`

To catch any fixture the slice should have re-keyed but missed, I ran:

```
grep -E "expect-error.*SIFR-TYPE-0001:.*(unsupported|cannot mix|cannot compare|bad operand)" crates/sifr/tests/e2e/fail/
```

The only hit is `unsupported_default_expr_call.sifr` ("unsupported default argument expression for parameter 'x'"), which is **not** an operator diagnostic — it's a default-value lowering failure correctly deferred to a later HIR slice. So the four re-keyed fixtures are the complete operator-message set, matching the user's claim.

Other `SIFR-TYPE-0001` fixtures still in the tree exercise non-operator domains (name resolution, ownership, match exhaustiveness, class fields, imports, etc.) and are deferred per the issue tracker. ✓

### Cross-references

Searching for the four fixture filenames across the repo turned up:

- [crates/sifr_diagnostics/src/codes.rs:608, 619](../crates/sifr_diagnostics/src/codes.rs:608) — registry entries for `SIFR-TYPE-0005` / `SIFR-TYPE-0006` reference `optional_arithmetic_without_narrowing.sifr` and `bigint_int_mixed_arithmetic.sifr` as representative fixtures. Both fixtures now use the active codes, so the registry pointer is internally consistent.
- [internal_docs/diagnostic_codes.md:81-82](../internal_docs/diagnostic_codes.md:81), [internal_docs/diagnostic_emission_inventory.md:78-79, 307-308](../internal_docs/diagnostic_emission_inventory.md:78) — inventory rows already document the migrated mappings (`InvalidOperator` → `SIFR-TYPE-0005`; int/bigint mixed → `SIFR-TYPE-0006`).

No verification baselines (`verification/`), `.snap` files, or other test infrastructure reference these fixtures, so no baseline re-key was missed. ✓

## Test coverage of the new codes

The diff upgrades three existing in-source tests:

- [check.rs:730-746](../crates/sifr_type_system/src/check.rs:730) `test_invalid_binary_op` — now asserts `code == TYPE_UNSUPPORTED_OPERATOR` for `Str -- Str`, `Int + Str`, and `Bool + Bool`. Strong coverage.
- [check.rs:749-760](../crates/sifr_type_system/src/check.rs:749) `test_optional_arithmetic_requires_narrowing` — asserts `code == TYPE_UNSUPPORTED_OPERATOR` for `optional_int + Int` only; the four sibling assertions (`Int + optional_int`, `optional_int - Int`, `optional_int * Int`, `optional_int / Int`) remain bare `is_err()` checks.
- [check.rs:811-823](../crates/sifr_type_system/src/check.rs:811) `test_mixed_int_bigint_comparison_blocked` — asserts `code == TYPE_INT_BIGINT_MIXED` for `Int == BigInt` only; the three sibling assertions (`BigInt == Int`, `Int < BigInt`, `BigInt > Int`) remain bare `is_err()` checks.

This is enough coverage in practice — the e2e fixtures already gate the `+`-operator and `==`-operator paths end-to-end, and the registry's representative-fixture pointers reach the same code paths through the broader pipeline. See N2 below for a small, optional follow-up that would tighten the unit-level coverage symmetrically.

## Out-of-scope check

The slice's "narrow" framing is preserved:

- HIR call sites (`crates/sifr_hir`), HIR-driver bridging (`crates/sifr_driver`), name resolution, ownership, class/import/flow/match/protocol diagnostics — all untouched by the diff.
- The `CompilePhase::TypeCheck => "SIFR-TYPE-0001"` bridge at [crates/sifr_driver/src/diagnostics.rs:137](../crates/sifr_driver/src/diagnostics.rs:137) is intentionally still present, consistent with the issue tracker's "Deferred bridge deletion" entry. The bridge will only be safe to delete once every code-bearing emission site routes its identity through the renderer; that crosses many domains beyond this slice.
- `TypeError` itself (the struct in [crates/sifr_type_system/src/lib.rs:31-36](../crates/sifr_type_system/src/lib.rs:31)) is not extended with structured-args fields; the registry's `arg!("operator")` / `arg!("operand_types")` declarations therefore aren't yet wired through. This is correctly deferred — slice 2b.2 is identity-only, not arg-wiring.

The only judgment call where the slice nudges past its stated scope is the comparison-equality `TYPE_MISMATCH` mapping at [check.rs:440](../crates/sifr_type_system/src/check.rs:440); see N1.

## Issue-tracker edit

[issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:36-37](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md:36) now reads:

```
- [x] `milestone_diag_4a` slice 2b.1 merged: …  PR: https://github.com/sifr-lang/sifr/pull/1673.
- [ ] Started `milestone_diag_4a` slice 2b.2: type-system operator diagnostic migration to active `SIFR-TYPE-0005` and `SIFR-TYPE-0006` codes with fixture re-keying.
```

Both edits are accurate. The slice 2b.1 row is correctly closed with its merged PR; the new slice 2b.2 row is open with a precise scope label that matches what landed in the working tree. The "Deferred `CompilePhase::TypeCheck => SIFR-TYPE-0001` bridge deletion …" row stays as `[x]` for the deferral decision, consistent with the still-present bridge in `sifr_driver`.

## Validation evidence

The user reports — and the surface area of the change confirms is sufficient — the standard local matrix:

- `cargo fmt --check` ✓
- `python3 scripts/check_hir_maintainability_guardrails.py` ✓ (touched no HIR files, but the guardrail still runs as a precaution)
- `cargo test -p sifr_type_system` ✓ (covers the three upgraded unit tests, the unchanged ones, and the rest of the crate's suite)
- `cargo test -p sifr -- --skip test_e2e_pass` ✓ (covers the e2e fail-suite which exercises the four re-keyed fixtures)
- `cargo clippy --workspace -- -D warnings` ✓
- `scripts/run_all_tests.sh --profile quick` ✓ — `report_signature=e1bf653aaa770517`, `wall_time=172.32s`

The `e1bf653aaa770517` signature matches the signature reported on prior `milestone_diag_4a` slices (slice 1, slice 2a, slice 2b.1) per [issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md](../issues/ad-hoc-semantic-diagnostic-code-taxonomy-and-structured-hir-diagnostics.md), suggesting the verification corpus and lane configuration are unchanged — appropriate for an identity-only migration. The `172.32s` wall time is in the normal range. I did not re-run any commands; the surface (one Rust file in the type-system, four single-line fixture edits, one issue-doc edit) is small enough that the user's reported gates fully cover the risk.

## Findings

### N1 (informational, non-blocking) — comparison-equality fallthrough mapped to `TYPE_MISMATCH` is broader than the slice's "0005 / 0006 only" framing but is inventory-correct

**Site**: [crates/sifr_type_system/src/check.rs:439-450](../crates/sifr_type_system/src/check.rs:439).

**What changed**: the `==` / `!=` fallthrough error at line 440 ("cannot compare '{}' and '{}' with {op}", with `kind: TypeErrorKind::TypeMismatch { ... }`) flipped from `code: None` to `code: Some(DiagnosticCode::TYPE_MISMATCH)` (`SIFR-TYPE-0002`). The task framing in this review's prompt and the issue-tracker scope row both describe slice 2b.2 as "operator diagnostic migration to active `SIFR-TYPE-0005` and `SIFR-TYPE-0006`", with broader type-mismatch diagnostics deferred. This site is inside an operator-checking function (`type_check_comparison`), so identifying it as an operator diagnostic is defensible; but its `kind` is `TypeMismatch`, not `InvalidOperator`, and `SIFR-TYPE-0002` is the type-mismatch code.

**Why it is not blocking**:

1. **Inventory is consistent.** [internal_docs/diagnostic_emission_inventory.md:69-70](../internal_docs/diagnostic_emission_inventory.md:69) explicitly maps `TypeErrorKind::TypeMismatch` (the variant used at line 446) to `SIFR-TYPE-0002`. The author's choice matches the documented mapping; picking `SIFR-TYPE-0005` here would have been *inconsistent* with the inventory.
2. **Slice hygiene needs it.** Eliminating every `code: None` from `crates/sifr_type_system/src/check.rs` is part of slice 2b.2's stated outcome (the user's "no `code: None` remains" claim depends on this site being assigned). Leaving it `None` would have broken that goal.
3. **No fixture exercises this exact text.** `grep "cannot compare '" crates/sifr/tests/e2e/fail/` returns only the int/bigint fixture (whose message starts with the same prefix but routes through the int/bigint guard at line 402, not the fallthrough). So no fixture re-key was missed and no e2e expectation was destabilized.
4. **The `kind` discriminator already disagrees with the operator-error sites.** The four other comparison-error sites at lines 491 and 504 use `kind: InvalidOperator { ... }` and were correctly mapped to `TYPE_UNSUPPORTED_OPERATOR`. The line-440 site is the only `kind: TypeMismatch { ... }` arm, so the kind/code split is internally clean.

**Suggested wording for the slice's PR description**: "Also assigns `SIFR-TYPE-0002` to the `==`/`!=` `TypeMismatch` fallthrough at `check.rs:440`, since leaving the only `TypeMismatch`-kind operator site as `code: None` would block the slice's `code: None`-elimination goal in `check.rs`. The mapping matches the inventory row at `diagnostic_emission_inventory.md:69`." This is a documentation-only nudge, not a blocker.

### N2 (suggestion, non-blocking) — symmetric `is_err()` assertions in unit tests still skip the code check

**Sites**: [check.rs:756-759](../crates/sifr_type_system/src/check.rs:756) (`test_optional_arithmetic_requires_narrowing`) and [check.rs:817-822](../crates/sifr_type_system/src/check.rs:817) (`test_mixed_int_bigint_comparison_blocked`).

The slice upgrades the **first** assertion in each test to verify both `is_err()` and `code == ...`. The remaining 4+3 assertions stay as bare `.is_err()` checks. The first assertion is enough to confirm the code wiring works for the representative path, but tightening the symmetric assertions would catch any future regression where, say, a `BigInt > Int` ordering check accidentally mapped to `TYPE_UNSUPPORTED_OPERATOR` instead of `TYPE_INT_BIGINT_MIXED`.

This is a small unit-coverage tightening that is **not** required for the slice — the e2e fixtures exercise the same code paths through the full pipeline — but it would be a low-cost follow-up if a future slice 2b.x revisits these tests for any other reason. Not a blocker.

## Net assessment

Mapping correctness is uniform; hygiene goal is met; fixture re-keys are complete and message-exact; no out-of-scope domain was migrated; the inventory and registry pointers stay consistent with the new active codes; the issue tracker is up to date; validation gates are sufficient for a code-identity-only change. The two non-blocking observations (N1, N2) document scope-boundary judgment and a small future-tightening opportunity — neither requires a change in this slice.

**Verdict: reviewer is satisfied; no blocking findings.** Slice 2b.2 is ready to ship.
