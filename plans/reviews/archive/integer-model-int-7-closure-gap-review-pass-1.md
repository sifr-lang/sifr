# INT-7: Diagnostics, Documentation, and Migration Cleanup — Gap/Coverage Review

## Review Metadata
- **Review pass**: 1 (gap/closure audit)
- **Branch state**: current main after PR #1897
- **Reviewer**: agent (automated gap audit)
- **Date**: 2026-05-08

---

## Verdict: **BLOCKED**

INT-7 is not ready to close. PR #1897 completed the diagnostic family reservation wave (SIFR-INT-0001..0011 all accounted for), but the remaining INT-7 acceptance criteria — migration cleanup, docs updates, and fixture hygiene — are not satisfied.

---

## What PR #1897 Accomplished

- Reserved non-emittable entries for `SIFR-INT-0002`, `SIFR-INT-0008`, `SIFR-INT-0009`, `SIFR-INT-0010` using `DiagnosticState::Reserved`.
- Regenerated `docs/errors/diagnostic-codes.md` and `internal_docs/diagnostic_codes.md` with consistent reserved summaries.
- Refactored `reserved_code` helper to accept custom summary text, which is a non-breaking simplification.
- All `SIFR-INT-0001..0011` codes are now accounted for — no orphaned slots.

This satisfies one INT-7 wave but not the full milestone.

---

## INT-7 Acceptance Criteria (from `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md`)

From the INT-7 scope:

> - Add stable diagnostic codes for integer range, narrowing, unsafe division, float precision, bool comparison, JSON policy, and dtype overflow-policy errors. ✓ (PR #1897)
> - Reserve and document the `SIFR-INT-0001..0011` diagnostic families listed in `internal_docs/integer_model.md`. ✓ (PR #1897)
> - **Update public docs, internal docs, demos, and issue references to use exact `int` and explicit fixed-width types. ✗**
> - **Remove or quarantine transition fixtures that mention public `bigint`. ✗**
> - **Add examples for web APIs, dataframes/tensors, bytes, FFI, and common domain values. ✗**
> - **Ensure architecture, roadmap, and relevant phase docs point at the canonical design. ✗**

The three unchecked items map to concrete blockers below.

---

## Blocking Findings

### B1: `demos/integer_safety/main.sifr` still uses `bigint` as a primary type

**File**: `demos/integer_safety/main.sifr`

**What**: The demo file treats `bigint` as the primary arbitrary-precision type throughout: function signatures (`-> bigint`), local declarations (`x: bigint`), arithmetic expressions (`bigint(n) * factorial(n - 1)`), and comparisons.

**Why this is a blocker**: INT-7 requires updating demos to use exact `int` and explicit fixed-width types. `internal_docs/integer_model.md` explicitly states `bigint` should not remain a separate user-facing type, and `demos/` is in-scope for demo hygiene. Even if `bigint` remains as a temporary transition alias in the compiler, the demos should not showcase it as the recommended type for arbitrary-precision work.

**Also affected**:
- `demos/generic_stdlib/main.sifr` — `value: bigint = bigint(42)`
- `demos/project_build/formatter.sifr` — `value: bigint = bigint(42)`
- `demos/cargo_manifest/helper.sifr` — uses `bigint` in comments
- Generated `demos/integer_safety/idiomatic.rs` — contains `use num_bigint::BigInt;` and `num_bigint::BigInt` references

**Remediation**: Replace `bigint` with `int` in demo signatures and declarations. `int` is now the exact arbitrary-precision type. Update the demo's narrative text (comments, header) to reflect the final model. Regenerate emitted Rust.

---

### B2: `bigint_arithmetic` remains in `quick_e2e_manifest.json` and `pr_e2e_manifest.json`

**Files**:
- `verification/validation_lanes/quick_e2e_manifest.json` (line 28)
- `verification/validation_lanes/pr_e2e_manifest.json` (line 28)

**What**: Both manifests include `"bigint_arithmetic"` as a pass fixture.

**Why this is a blocker**: `verification/integer_model_implementation_inventory.md` (lines 85–95) explicitly lists these manifest entries as "Known Legacy References to Retire or Quarantine". The implementation inventory is part of the INT-0 audit artifact and is authoritative for INT-7 migration scope.

**Remediation**: Remove `"bigint_arithmetic"` from both manifests. The fixture file itself can remain on disk as a quarantined regression artifact (see NB1 below), but it should not be in the active quick/pr validation lanes.

---

### B3: 11 `bigint_*` pass fixtures in `crates/sifr/tests/e2e/pass/` are not accounted for

**Files** (11 total):
- `bigint_arithmetic.sifr`
- `bigint_as_dict_key.sifr`
- `bigint_basic.sifr`
- `bigint_comparison.sifr`
- `bigint_factorial.sifr`
- `bigint_large_value.sifr`
- `bigint_overflow_conversion.sifr`
- `bigint_to_int.sifr`
- `generic_accumulate_bigint.sifr`
- `generic_counter_bigint.sifr`
- `int_to_bigint.sifr`

**Why this is a blocker**: `verification/integer_model_implementation_inventory.md` explicitly flags these fixtures as "transition or rewrite fixtures" to be retired or quarantined. INT-7 owns this cleanup.

**Current state**:
- `bigint_arithmetic.sifr` currently has a `# expect-error: SIFR-TYPE-0006` marker — it actually asserts a compile error, so the filename is misleading (the fixture name in manifests is `bigint_arithmetic`, which implies a pass fixture, but it has a fail assertion). This needs resolution.
- The 11 pass fixtures rely on the `bigint` transition alias. Once the public alias is removed, these fixtures will either compile with warnings (if the alias persists) or fail (if removed entirely). They are not forward-compatible with the final model.

**Remediation**: See Non-blocking notes NB1–NB3 for fixture disposition strategy.

---

### B4: Phase docs still describe `bigint` as the canonical arbitrary-precision type

**Files**:
- `internal_docs/phases/13_type_system_completion.md` (lines 440–490) — `bigint` is documented as the primary arbitrary-precision type with `bigint` literals, `Type::BigInt`, `num_bigint::BigInt` codegen, and `int(b)` / `bigint(n)` explicit conversion rules.
- `internal_docs/phases/14_codegen_architecture.md` (lines 484, 777, 830, 875) — `needs_bigint` boolean flag documented in codegen preamble generation.
- `internal_docs/phases/28_decimal_type_and_exact_numeric_semantics.md` (lines 46, 50, 56, 76, 78, 139–140, 165, 168, 172, 208–209) — `Decimal(bigint)`, `BigDecimal(bigint)`, `bigint(decimal)`, `bigint(bigdecimal)` conversion rules.
- `internal_docs/phases/13_type_system_completion.md` (lines 489–490) — lists expected e2e fixtures including `bigint_basic`, `bigint_large_value`, `bigint_arithmetic`, `bigint_comparison`, `bigint_to_int`, `int_to_bigint`, `bigint_as_dict_key`, `bigint_factorial`.

**Why this is a blocker**: INT-7 requires "architecture, roadmap, and relevant phase docs point at the canonical design." The phase docs currently describe the bootstrap model (before the integer model rewrite) as if it were the current design.

**Remediation**: Update phase docs to reflect the final model. The canonical design in `internal_docs/integer_model.md` says `int` is the arbitrary-precision type, `bigint` is a temporary transition alias, and there is no separate long-term user-facing bigint. Phase doc updates should either:
1. Point at `internal_docs/integer_model.md` as the canonical source for integer semantics, or
2. Be updated to reflect the final model (int for arbitrary precision, no separate bigint type).

---

### B5: `internal_docs/diagnostic_emission_inventory.md` still lists `TYPE_INT_BIGINT_MIXED` as requiring retirement/migration

**File**: `internal_docs/diagnostic_emission_inventory.md` (line 80)

**What**: The inventory entry for `SIFR-TYPE-0006` is marked as needing retirement or migration.

**Why this is a blocker**: `verification/integer_model_implementation_inventory.md` (line 46) explicitly says "retire or migrate `TYPE_INT_BIGINT_MIXED`". The diagnostic emission inventory is an INT-7 handoff artifact that should be synchronized when migration decisions are made.

**Current state**: `SIFR-TYPE-0006` is active (Severity::Error, `TYPE_INT_BIGINT_MIXED` constant in `codes.rs`, present in `ACTIVE_DIAGNOSTIC_CODES`). The `bigint_int_mixed_arithmetic.sifr` and `bigint_int_mixed_comparison.sifr` fail fixtures assert `SIFR-TYPE-0006`.

**Remediation**: See question 3 answer below. `SIFR-TYPE-0006` should remain active until the public `bigint` alias is removed, at which point it should be deprecated (not deleted, as existing fixtures assert it). The inventory entry should be updated to document the decision.

---

## Non-Blocking Notes (for fixture disposition strategy)

### NB1: `bigint_*` pass fixtures disposition

The 11 `bigint_*` pass fixtures use the `bigint` transition alias. Once the alias is removed, these fixtures will fail to compile. The correct disposition is:

- **Recommended**: Remove from active manifests (done in B2), add a comment header `# TEMPORARY TRANSITION FIXTURE — bigint alias only` at the top of each file to mark them as quarantined, and leave them on disk. This preserves regression coverage during the transition period and makes it clear these are not canonical.
- **Alternative**: Delete the pass fixtures entirely. Less valuable than quarantine since it removes regression coverage.
- **Not recommended**: Keep them as-is in the manifest, as the implementation inventory explicitly flags them for retirement.

### NB2: `bigint_arithmetic.sifr` is misnamed

`crates/sifr/tests/e2e/pass/bigint_arithmetic.sifr` currently has `# expect-error: SIFR-TYPE-0006` — it asserts a compile failure, making it a fail fixture in disguise. The fixture name in manifests implies it should pass. This needs resolution regardless of the disposition decision:

- Either rename the file to `bigint_int_mixed_arithmetic.sifr` and move to `fail/` (if the test should stay), or
- Remove the `# expect-error` marker and update the fixture to actually pass (if `bigint` is still supported as a pass-through), or
- Delete the fixture if it's no longer relevant.

The manifest entries for `bigint_arithmetic` reference this file, so the misnamed file needs resolution before removing the manifest entry.

### NB3: `int_to_bigint.sifr` and `bigint_to_int.sifr` fixtures

These fixtures test `int`/`bigint` conversion constructors. Once the `bigint` alias is removed, both constructors (`int_to_bigint` — calling `bigint()` from `int`, `bigint_to_int` — calling `int()` from `bigint`) will no longer make sense in the same way. Update or delete these fixtures when the alias removal PR lands.

---

## Answers to Review Questions

### Q1: What are the minimal concrete blockers to closing INT-7?

Five concrete blockers:

1. **`demos/integer_safety/main.sifr` (and 3 related files)**: `bigint` used as the primary type in demo code — violates INT-7 demo hygiene requirement.
2. **`bigint_arithmetic` in `quick_e2e_manifest.json` and `pr_e2e_manifest.json`**: explicitly listed in the implementation inventory as a legacy reference to retire.
3. **11 `bigint_*` pass fixtures not quarantined**: implementation inventory explicitly marks these as transition/rewrite fixtures.
4. **Phase docs (13, 14, 28) describe `bigint` as canonical**: INT-7 requires phase docs to point at the canonical design.
5. **`diagnostic_emission_inventory.md` has stale `TYPE_INT_BIGINT_MIXED` entry**: should be synchronized with the migration decision.

All five are addressable in a single PR wave (items 1–4 are directly related to `bigint` hygiene; item 5 is a doc sync).

---

### Q2: bigint transition fixtures: rename, move, remove, document as quarantined, or leave until public alias removed?

**Recommended strategy** (not leaving until the last moment, as the inventory explicitly calls these out):

1. **For pass fixtures**: Remove from manifests immediately. Add a comment header `# TEMPORARY TRANSITION FIXTURE — subject to removal when public bigint alias is removed` to each file. Leave files on disk during the transition period to preserve regression coverage. When the public `bigint` alias is removed in a future PR, delete these fixtures.

2. **For `bigint_arithmetic.sifr`**: Resolve the misnamed fixture first (it has `# expect-error` and should be in `fail/` or renamed), then remove from manifests and quarantine.

3. **For fail fixtures** (`bigint_int_mixed_arithmetic.sifr`, `bigint_int_mixed_comparison.sifr`): These test `SIFR-TYPE-0006` which should remain active as long as the `bigint` alias exists. No action needed on these fixtures now; when the alias is removed, the fail assertion and the diagnostic can both be deprecated together.

4. **Do not leave everything until the public alias is removed** — the implementation inventory is explicit that these fixtures are "Known Legacy References to Retire or Quarantine" and that list is an INT-0 audit artifact.

---

### Q3: Should `TYPE_INT_BIGINT_MIXED` / `SIFR-TYPE-0006` be retired/migrated now, or remain as transition-only until public bigint is removed?

**Remain active as transition-only until the public `bigint` alias is removed.**

Reasoning:
- `SIFR-TYPE-0006` serves a real purpose: it enforces that mixing exact `int` and the transition `bigint` alias requires explicit conversion. This is a valid type-check rule that users of the transition alias encounter.
- Two fail fixtures explicitly assert `SIFR-TYPE-0006`: `bigint_int_mixed_arithmetic.sifr` and `bigint_int_mixed_comparison.sifr`. These fixtures remain valid as long as the `bigint` alias exists.
- The transition alias (and this diagnostic) should be removed in the same PR that removes the `bigint` alias from the type system entirely. At that point, the diagnostic and its fixtures are both deprecated together.
- Deleting `SIFR-TYPE-0006` now would break the fail fixtures and require rewriting them — a churn that should wait for the alias removal PR.

**Action required now**: Update `internal_docs/diagnostic_emission_inventory.md` entry for `SIFR-TYPE-0006` to document that it remains active during the transition period and will be deprecated when the public `bigint` alias is removed.

---

### Q4: Are any public docs currently stale in a way that blocks INT-7?

**Yes — `docs/errors/SIFR-TYPE-0006.md` and `docs/errors/diagnostic-codes.md` describe `bigint` as an active type.**

Specifically:
- `docs/errors/SIFR-TYPE-0006.md` summary: "Int and bigint are mixed without an explicit conversion." — describes `bigint` as an active type.
- `docs/errors/SIFR-INT-0011.md` summary: "Temporary bigint transition alias used." — correctly describes `bigint` as temporary.

The `SIFR-TYPE-0006` description is accurate for the current transition state (users can still write `bigint` annotations), but the diagnostic code table in `docs/errors/diagnostic-codes.md` lists `SIFR-TYPE-0006` without context that `bigint` is a temporary alias. This is a minor clarity gap, not a blocking staleness.

The more significant public doc staleness is in the phase docs (B4 above) which describe `bigint` as the primary arbitrary-precision type.

---

### Q5: What exact PR sequence would satisfy INT-7 closure with minimal risk?

**PR 1: Bigint demo hygiene and phase doc updates**
- Replace `bigint` with `int` in `demos/integer_safety/main.sifr`, `demos/generic_stdlib/main.sifr`, `demos/project_build/formatter.sifr`.
- Update header comments/narrative text in affected demo files.
- Regenerate `demos/integer_safety/idiomatic.rs` and other generated demo files.
- Update `demos/cargo_manifest/helper.sifr` comment.
- Update `internal_docs/phases/13_type_system_completion.md` to reference `internal_docs/integer_model.md` as the canonical source for integer semantics, or update to reflect the final model.
- Update `internal_docs/phases/14_codegen_architecture.md` to remove `needs_bigint` references or mark them as legacy.
- Update `internal_docs/phases/28_decimal_type_and_exact_numeric_semantics.md` conversion rules to reference `int` instead of `bigint`.
- Sync `internal_docs/diagnostic_emission_inventory.md` `SIFR-TYPE-0006` entry to document transition-only status.

**PR 2: Manifest cleanup and fixture quarantine**
- Remove `"bigint_arithmetic"` from `verification/validation_lanes/quick_e2e_manifest.json`.
- Remove `"bigint_arithmetic"` from `verification/validation_lanes/pr_e2e_manifest.json`.
- Resolve `bigint_arithmetic.sifr` misnamed fixture (has `# expect-error`, should be in `fail/` or renamed).
- Add quarantine header comment to remaining 10 `bigint_*` pass fixtures.

**PR 3: Final docs regeneration and INT-7 checklist update**
- Run `cargo run -p sifr_diagnostics --bin gen-error-docs` to regenerate `docs/errors/diagnostic-codes.md` and `internal_docs/diagnostic_codes.md` with any updated diagnostic entries.
- Update `issues/ad-hoc-integer-model-and-fixed-width-numeric-contract.md` INT-7 checklist to mark PR 1 and PR 2 items complete.
- Add INT-7 closure review artifact.

**Subsequent PR (not INT-7 scope): Remove public `bigint` alias**
- When the `bigint` transition alias is removed from the type system, deprecate `SIFR-TYPE-0006` simultaneously.
- Delete the quarantined `bigint_*` pass fixtures.
- Delete the `bigint_int_mixed_*` fail fixtures (or update to test `int`/`int32` mixing if that scenario is still relevant).

---

## Summary Table

| Item | Status | Files affected | Action |
|------|--------|---------------|--------|
| Diagnostic code reservation (SIFR-INT-0002/0008/0009/0010) | ✓ Done PR #1897 | `codes.rs`, `diagnostic-codes.md`, `diagnostic_codes.md` | None |
| Demo hygiene (bigint as primary type) | ✗ Blocker | `demos/integer_safety/main.sifr`, `demos/generic_stdlib/main.sifr`, `demos/project_build/formatter.sifr`, `demos/cargo_manifest/helper.sifr` | Replace bigint with int, update narrative |
| Manifest cleanup | ✗ Blocker | `quick_e2e_manifest.json`, `pr_e2e_manifest.json` | Remove bigint_arithmetic |
| Phase doc updates | ✗ Blocker | `internal_docs/phases/13_type_system_completion.md`, `14_codegen_architecture.md`, `28_decimal_type_and_exact_numeric_semantics.md` | Point at canonical design or update |
| Fixture quarantine | ✗ Blocker | 11 `bigint_*` pass fixtures, `bigint_arithmetic.sifr` | Remove from manifests, add quarantine header |
| Diagnostic emission inventory sync | ✗ Blocker | `internal_docs/diagnostic_emission_inventory.md` | Document TYPE_INT_BIGINT_MIXED transition-only status |
| TYPE_INT_BIGINT_MIXED retirement | → Defer to alias removal PR | `codes.rs`, `SIFR-TYPE-0006.md`, fail fixtures | Keep active; deprecate together with alias |
| Public docs staleness (SIFR-TYPE-0006 description) | NB only | `docs/errors/SIFR-TYPE-0006.md` | Acceptable during transition; will be resolved with alias removal |

---

## Review History Entry

**INT-7 gap/closure review pass 1**: Found 5 concrete blockers before INT-7 can close. PR #1897 completed the diagnostic reservation wave but did not address migration cleanup (demo hygiene, manifest cleanup, fixture quarantine, phase doc updates, and diagnostic inventory sync). `TYPE_INT_BIGINT_MIXED` should remain active as transition-only; retire simultaneously with public `bigint` alias removal. Recommended 3-PR sequence for closure.