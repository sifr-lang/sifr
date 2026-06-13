# Review: codegen_runtime_build_gap_v3 (Pass 3)

- Phase: `codegen_runtime_build_gap_v3`
- MD source: `issues/codegen-runtime-build-gap-root-cause-breakdown-2026-04-05-v3.md`
- CSV source: `verification/leetcode/codegen_runtime_build_gap_breakdown_20260405_v3.csv`
- Review date: 2026-04-05

---

## 1. Count Integrity

### 1.1 Total count

| Check | Expected | Actual | Status |
|-------|----------|--------|--------|
| MD declared total | 58 | 58 | PASS |
| MD per-case mapping rows | 58 | 58 | PASS |
| CSV data rows (excl. header) | 58 | 58 | PASS |

### 1.2 Family sums

| Family | Declared | Counted | Status |
|--------|----------|---------|--------|
| recursive_field_surface_leaks_to_codegen_without_gate | 21 | 21 | PASS |
| type_contract_emission_gap | 20 | 20 | PASS |
| ownership_and_borrow_emission_gap | 6 | 6 | PASS |
| other_codegen_build_gap | 4 | 4 | PASS |
| binding_scope_and_capture_emission_gap | 3 | 3 | PASS |
| runtime_oracle_canonicalization_needed | 2 | 2 | PASS |
| codegen_production_panic_missing_structured_emission | 1 | 1 | PASS |
| truthiness_bool_lowering_gap | 1 | 1 | PASS |
| **Sum** | **58** | **58** | **PASS** |

### 1.3 Lane sums

| Lane | Declared | Counted | Status |
|------|----------|---------|--------|
| compiler_fix | 35 | 35 | PASS |
| both | 21 | 21 | PASS |
| sifr_adaptation | 2 | 2 | PASS |
| **Sum** | **58** | **58** | **PASS** |

### 1.4 Rust error code presence counts

All 15 error codes verified against per-case mapping:

| Code | Declared | Counted | Status |
|------|----------|---------|--------|
| E0308 | 34 | 34 | PASS |
| E0609 | 21 | 21 | PASS |
| NO_RUST_CODE | 7 | 7 | PASS |
| E0382 | 6 | 6 | PASS |
| E0277 | 4 | 4 | PASS |
| E0282 | 2 | 2 | PASS |
| E0596 | 2 | 2 | PASS |
| E0369 | 1 | 1 | PASS |
| E0424 | 1 | 1 | PASS |
| E0425 | 1 | 1 | PASS |
| E0434 | 1 | 1 | PASS |
| E0502 | 1 | 1 | PASS |
| E0599 | 1 | 1 | PASS |
| E0600 | 1 | 1 | PASS |
| E0631 | 1 | 1 | PASS |

### 1.5 MD-to-CSV row-level consistency

All 58 rows cross-checked field-by-field (fixture_slug, codes, root_cause_family, resolution_lane). **Zero discrepancies found.**

---

## 2. Edge Case Categorization

### 0211 (design_add_and_search_words_data_structure)

- Codes: E0277, E0382
- Family: type_contract_emission_gap | Lane: compiler_fix
- v3 correction: moved to type-contract from prior placement
- Evidence: `if c == None` compares String with Option<_> (E0277); E0382 is consequential move error
- **Verdict: CORRECT.** Primary failure is codegen emitting a type-invalid comparison. The move error is downstream.

### 0729 (my_calendar_i)

- Codes: E0277, E0596
- Family: type_contract_emission_gap | Lane: compiler_fix
- v3 correction: moved from binding/capture family to type-contract
- Evidence: compiler auto-generates Display impl using `{}` on CalendarNode which lacks Display trait; E0596 is downstream borrow consequence
- **Verdict: CORRECT.** Root cause is spurious trait emission, not a binding/capture scope issue.

### 0783 (minimum_distance_between_bst_nodes)

- Codes: E0369
- Family: type_contract_emission_gap | Lane: compiler_fix
- Distinguishing note in MD: not grouped under recursive-field/both because no E0609
- Evidence: `second_val - first_val` on Option<i64> -- compiler wrapped scalars into Option without unwrap
- **Verdict: CORRECT.** This is clearly a type-contract emission issue (incorrect Option wrapping of scalars), not a recursive field surface issue. The distinguishing note is well-placed.

### 0662 (maximum_width_of_binary_tree)

- Codes: NO_RUST_CODE
- Family: codegen_production_panic_missing_structured_emission | Lane: compiler_fix
- Evidence: `thread 'main' panicked at crates/sifr_codegen/src/lib.rs:1492:17`
- **Verdict: CORRECT.** Compiler panic, not generated-code error. Unique family name justified -- different failure mode from all other cases.

### 1968 (array_with_elements_not_equal_to_average_of_neighbors)

- Codes: NO_RUST_CODE
- Family: runtime_oracle_canonicalization_needed | Lane: sifr_adaptation
- Evidence: code compiled and ran successfully (artifact cache hit), failure is oracle shape/order assertion
- **Verdict: CORRECT.** Multiple valid outputs exist for this problem. Pure fixture-side canonicalization needed.

### 2215 (find_the_difference_of_two_arrays)

- Codes: NO_RUST_CODE
- Family: runtime_oracle_canonicalization_needed | Lane: sifr_adaptation
- Evidence: code compiled and ran successfully (artifact cache hit), set-order mismatch in oracle
- **Verdict: CORRECT.** Output set ordering is non-deterministic. Pure fixture-side fix.

**All 6 edge cases: PASS**

---

## 3. Lane Assignment Defensibility

### compiler_fix (35 cases)

All 35 cases exhibit at least one of:
- Rust compiler error (E0308/E0382/E0277/E0282/E0369/E0424/E0425/E0434/E0502/E0596/E0599/E0600/E0631)
- Compiler panic (0662)
- NO_RUST_CODE with build failure indicating codegen produced invalid/empty output (0394, 0513, 0838, 1609)

The fix in every case must happen in the compiler's code generation pipeline. No fixture/adaptation work alone would resolve these.

**Verdict: DEFENSIBLE**

### both (21 cases)

All 21 are in the recursive_field_surface_leaks_to_codegen_without_gate family. Every case involves:
- E0609 (field access on Option<T>) confirming the recursive-field surface leak pattern
- The dual requirement is sound: compiler must gate/fix Option-wrapped field access AND the surface decision about recursive type representation may need adaptation

The 1:1 mapping between this family and the `both` lane is clean and consistent.

**Verdict: DEFENSIBLE**

### sifr_adaptation (2 cases)

Both 1968 and 2215:
- Code compiles and executes successfully (no Rust errors)
- Failure is purely at runtime oracle comparison (deterministic shape/order assertion on non-deterministic output)
- Fix is entirely fixture-side canonicalization

**Verdict: DEFENSIBLE**

---

## 4. Remaining Issues

### No misclassifications found.

All v3 corrections (0211, 0729, 0783) are well-evidenced and the rationale in both MD and CSV supports the placement.

### Minor suggestions (non-blocking):

1. **other_codegen_build_gap rationale is thin.** The 4 cases (0394, 0513, 0838, 1609) all use the identical rationale "Residual generated-Rust build failure requiring compiler-side fix." Consider adding a one-line note per case indicating whether codegen produced empty output, malformed syntax, or truncated code. This would strengthen traceability without changing classification.

2. **0729 secondary E0596.** The rationale correctly identifies E0277 as root cause and E0596 as downstream. Consider adding a brief note (similar to the 0783 distinguishing note) explicitly stating E0596 is consequential here, to preempt any future confusion with the ownership_and_borrow family.

3. **Error code section header.** "presence count" is correct but could be made slightly more explicit: "presence count (each case contributes 1 per unique error code it contains)" to eliminate ambiguity vs. occurrence count.

---

## Verdict

**READY**

Count integrity is perfect (total, families, lanes, error codes all verified). All 6 edge cases are correctly categorized with defensible evidence. Lane assignments are consistent and well-supported across all 58 cases. MD and CSV are in exact agreement. The 3 minor suggestions above are non-blocking improvements for documentation clarity only -- no reclassification or structural changes needed.
