# Review Pass 2: Codegen Runtime Build Gap Root-Cause Breakdown (v2)

**Reviewed**: 2026-04-05
**Source artifact**: `issues/codegen-runtime-build-gap-root-cause-breakdown-2026-04-05-v2.md`
**Supporting data**: CSV breakdown (v2), pass-1 review
**Scope**: Verify pass-1 reconciliations, audit lane assignments, flag remaining misclassifications

---

## 1. Pass-1 Reconciliation Audit

The v2 header claims: "Reviewer pass-1 reconciliations applied: `0211`, `0729`, `0783` with emit-level evidence."

### 1.1 Case 0211 — PARTIALLY ADDRESSED

| Dimension | Pass-1 Recommendation | v2 Applied | Verdict |
|-----------|----------------------|------------|---------|
| Family | `ownership_move_reuse` → `type_contract_mismatch` | `type_contract_emission_gap` | **APPLIED** |
| Lane | `compiler_fix` → `both` | `compiler_fix` (unchanged) | **REJECTED** |

**Assessment**: The family reclassification is correct — E0277 (`can't compare 'String' with 'Option<_>'`) is the root-cause error, not E0382.

The lane rejection is **defensible**. The v2 rationale ("Emit shows invalid comparison `if c == None` for String token") demonstrates the codegen is emitting a logically invalid comparison. This is a pure compiler logic bug: no runtime type definition change makes `String == Option<_>` valid. The compiler must stop emitting this pattern. `compiler_fix` holds.

**Status**: PASS — reconciliation adequate.

### 1.2 Case 0729 — INADEQUATELY ADDRESSED

| Dimension | Pass-1 Recommendation | v2 Applied | Verdict |
|-----------|----------------------|------------|---------|
| Family | Keep `misc` (or investigate) | `binding_scope_and_capture_emission_gap` | **WRONG FAMILY** |
| Lane | `compiler_fix` → `both` | `compiler_fix` (unchanged) | **STILL UNJUSTIFIED** |

**Family problem**: The error codes are E0277 (`CalendarNode` doesn't implement `std::fmt::Display`) and E0596 (`cannot borrow as mutable`). Neither is a binding-scope or capture issue. Compare with the other cases in this family:

| Case | Codes | Genuine binding/capture issue? |
|------|-------|-------------------------------|
| 0051 | E0425 (cannot find value `row` in scope) | Yes |
| 0304 | E0424 (`self` in non-method), E0277, E0282 | Yes (E0424 is binding) |
| 0417 | E0434 (can't capture dynamic env in fn item), E0596 | Yes (E0434 is capture) |
| **0729** | **E0277 (missing Display), E0596 (mutability)** | **No** |

E0277 is a trait-bound failure. E0596 is a mutability constraint. Neither involves undefined variables, misused `self`, or invalid closure captures. This is a **misclassification** — 0729 does not belong in `binding_scope_and_capture_emission_gap`.

**Lane problem**: The v2 rationale itself says "auto-Display use of CalendarNode without Display impl." If `CalendarNode` is a sifr runtime type, adding `Display` is adaptation work. Pass-1 flagged this and asked to investigate the type's origin. The v2 provides no answer. Without that investigation, `compiler_fix` alone is not justified — the fix may require `impl Display for CalendarNode` in the runtime type definitions.

**Recommended correction**:
- Family: Create a new single-case family `trait_and_mutability_emission_gap` or place in `type_contract_emission_gap` (E0277 is a type/trait constraint failure).
- Lane: `both` until CalendarNode's origin is confirmed. If CalendarNode is compiler-synthesized, `compiler_fix` is fine.

### 1.3 Case 0783 — PARTIALLY ADDRESSED

| Dimension | Pass-1 Recommendation | v2 Applied | Verdict |
|-----------|----------------------|------------|---------|
| Family | `misc` → `recursive_field_access` (or new `option_wrapped_value`) | `type_contract_emission_gap` | **DIFFERENT PATH, DEFENSIBLE** |
| Lane | `compiler_fix` → `both` | `compiler_fix` (unchanged) | **DEFENSIBLE WITH CAVEAT** |

**Family**: Pass-1 recommended folding 0783 into the recursive-field/Option-unwrap family because the root cause (operating on `Option<T>` without unwrapping) is the same mechanism. The v2 placed it in `type_contract_emission_gap` instead. Both are defensible:
- **For type_contract**: E0369 is literally a type error (operator not defined on `Option<i64>`). Correct categorization by error type.
- **For recursive_field/option_wrapped**: The root *mechanism* (Option leaking to expression position without unwrap) is identical to the 21 E0609 cases. Correct categorization by root cause.

The v2's choice is internally consistent — it classifies by error code pattern. Acceptable.

**Lane**: The v2 rationale says "compiler wrapped scalars into Option and failed to unwrap." If the compiler is the agent that introduced the Option wrapping (not the runtime type definition), then `compiler_fix` is correct. This is a meaningful distinction from the recursive_field cases where the Option wrapping originates from the recursive type definition (ListNode/TreeNode). However, the claim "compiler wrapped scalars" needs verification — if the Option wrapping comes from TreeNode field definitions, the same `both` logic should apply.

**Status**: CONDITIONAL PASS — acceptable if the "compiler wrapped scalars" claim is verified. Flag for workstream lead to confirm.

---

## 2. Lane Assignment Audit

### 2.1 Aggregate Lane Shift

| Lane | v1 (original) | Pass-1 recommended | v2 |
|------|--------------|-------------------|-----|
| `both` | 38 | 41 | 21 |
| `compiler_fix` | 20 | 17 | 35 |
| `sifr_adaptation` | 0 | 0 | 2 |

The v2 moved **sharply** in the opposite direction from pass-1's recommendation. The main driver: all 19 `type_contract_emission_gap` cases moved from `both` (v1) to `compiler_fix` (v2). This is the single largest lane reassignment.

### 2.2 Type Contract → compiler_fix: Is It Defensible?

The v2 judgment definition: *"`compiler_fix`: generated Rust is invalid, panics in codegen, or violates type/ownership/binding contracts; these are compiler bugs even when fixture syntax is imperfect."*

Under this definition, every E0308 case is a compiler bug. This is **technically correct** — the compiler should never emit code that fails `cargo check`. But the question for execution planning is: **can the compiler fix alone resolve the failure, or does the runtime type definition also need changes?**

Spot-checking tree-related type_contract cases:

| Case | Problem domain | Likely type surface |
|------|---------------|-------------------|
| 0105 (preorder + inorder → tree) | Binary tree construction | `Option<TreeNode>` / `TreeNode` mismatch likely |
| 0108 (sorted array → BST) | BST construction | Same |
| 0450 (delete BST node) | BST mutation | Same |
| 0572 (subtree of another tree) | Tree comparison | Same |
| 0617 (merge two binary trees) | Tree merge | Same |
| 0701 (insert into BST) | BST insertion | Same |

These 6+ cases involve recursive tree types where E0308 mismatches likely occur between `TreeNode` and `Option<TreeNode>`. The same Option-wrapping dynamic that makes recursive_field cases `both` may apply here — the only difference is the error manifests as E0308 (type mismatch) rather than E0609 (field access on Option).

**Risk**: If these cases need runtime type definition changes alongside the compiler fix, marking them `compiler_fix` only will lead to incomplete fixes that still fail at build time.

**Verdict**: The lane assignment is **consistent and rule-based** (all type_contract → compiler_fix), which is good for reproducibility. But it may **under-count adaptation work** for tree/list type_contract cases. This should be flagged to the workstream lead as a known risk.

### 2.3 Recursive Field Surface → both: Correct

All 21 cases have E0609 (field access on `Option<ListNode>` or `Option<TreeNode>`). The `both` lane is well-justified: the compiler must emit unwrap/match patterns, and the runtime type definitions define the Option wrapping. No issues.

### 2.4 sifr_adaptation Lane (NEW): Correct

Cases 1968 and 2215 are correctly identified as adaptation-only:
- Both show cache hits and no build errors — the Rust code compiled and ran
- Failure is at the oracle assertion level (non-deterministic output ordering)
- Family `runtime_oracle_canonicalization_needed` is accurate
- `sifr_adaptation` lane is the right call

This is a good addition that was missing from v1.

### 2.5 Other Lanes: No Issues

| Family | Lane | Cases | Verdict |
|--------|------|-------|---------|
| `ownership_and_borrow_emission_gap` | `compiler_fix` | 6 | Correct — borrow/move violations are pure codegen logic errors |
| `binding_scope_and_capture_emission_gap` | `compiler_fix` | 3 (excluding 0729) | Correct — undefined bindings are codegen errors |
| `other_codegen_build_gap` | `compiler_fix` | 4 | Correct — codegen failed to emit Rust code at all |
| `codegen_production_panic_missing_structured_emission` | `compiler_fix` | 1 | Correct — compiler panic is a compiler bug |
| `truthiness_bool_lowering_gap` | `compiler_fix` | 1 | Correct — Python→Rust lowering is compiler responsibility |

---

## 3. Remaining Misclassifications and Weak Rationale

### 3.1 MISCLASSIFICATION: 0729 in `binding_scope_and_capture_emission_gap`

**Severity**: Concrete error.
**Detail**: See Section 1.2. E0277 + E0596 are not binding/scope/capture issues.
**Impact**: Inflates `binding_scope_and_capture_emission_gap` count from 3 to 4; family description no longer accurately characterizes all its members.

### 3.2 WEAK RATIONALE: `other_codegen_build_gap` catch-all (4 cases)

Cases 0394, 0513, 0838, 1609 all have `NO_RUST_CODE` and identical rationale: "Residual generated-Rust build failure requiring compiler-side fix." Pass-1 Section 5.2 asked for investigation into what Python patterns trigger empty emission. The v2 provides no additional evidence.

These 4 cases are essentially unanalyzed — they're correctly bucketed as compiler failures (no code emitted), but there's no root-cause insight. For execution toward zero failures, the workstream needs to know what triggers the empty emission.

**Recommendation**: Not a misclassification, but rationale should be strengthened before executive use. At minimum, note whether these share a common Python pattern (e.g., specific stdlib usage, string manipulation, BFS/DFS patterns).

### 3.3 INCONSISTENCY: 0783 vs recursive_field family

As discussed in Section 1.3, the Option-unwrap mechanism is shared between 0783 (`Option<i64>` arithmetic) and the 21 recursive_field cases (`Option<ListNode/TreeNode>` field access). Placing them in different families with different lanes creates an inconsistency:
- recursive_field (Option field access) → `both`
- 0783 (Option arithmetic) → `compiler_fix`

If the v2 position is that 0783 is `compiler_fix` because "compiler wrapped scalars into Option" (i.e., the Option wrapping is compiler-introduced, not from type definitions), this should be stated explicitly as the distinguishing criterion. Currently it's implicit.

### 3.4 MINOR: Family count in header vs per-case listing

Quick arithmetic verification of v2 counts:

| Family | Declared | Counted from per-case | Status |
|--------|----------|-----------------------|--------|
| `recursive_field_surface_leaks_to_codegen_without_gate` | 21 | 21 | PASS |
| `type_contract_emission_gap` | 19 | 19 | PASS |
| `ownership_and_borrow_emission_gap` | 6 | 6 | PASS |
| `binding_scope_and_capture_emission_gap` | 4 | 4 | PASS |
| `other_codegen_build_gap` | 4 | 4 | PASS |
| `runtime_oracle_canonicalization_needed` | 2 | 2 | PASS |
| `codegen_production_panic_missing_structured_emission` | 1 | 1 | PASS |
| `truthiness_bool_lowering_gap` | 1 | 1 | PASS |
| **Total** | **58** | **58** | **PASS** |

Lane counts:

| Lane | Declared | Counted | Status |
|------|----------|---------|--------|
| `compiler_fix` | 35 | 35 | PASS |
| `both` | 21 | 21 | PASS |
| `sifr_adaptation` | 2 | 2 | PASS |
| **Total** | **58** | **58** | **PASS** |

All counts are arithmetically consistent.

---

## 4. Summary of Required Corrections

| # | Case | Issue | Severity | Action |
|---|------|-------|----------|--------|
| 1 | 0729 | Wrong family (`binding_scope_and_capture` — errors are E0277/E0596, not binding/capture) | **High** | Reclassify to `type_contract_emission_gap` or new family |
| 2 | 0729 | Lane unjustified (`compiler_fix` — Display impl may be adaptation work) | **Medium** | Change to `both` or provide CalendarNode origin evidence |
| 3 | 0783 | Implicit distinction from recursive_field family not documented | **Low** | Add explicit note: "compiler-introduced Option wrapping" as distinguishing criterion |
| 4 | Tree type_contract cases | Risk of under-counting adaptation work | **Low** | Flag to workstream lead; no reclassification needed if compiler team accepts full ownership |
| 5 | 0394/0513/0838/1609 | No root-cause insight beyond "no Rust emitted" | **Low** | Acceptable for v2 but should be investigated before workstream execution |

---

## 5. What v2 Got Right

- **0211 reconciliation**: Family correctly reclassified; `compiler_fix` lane justified with emit-level evidence showing a pure codegen logic error (`String == None` comparison).
- **0783 reconciliation**: Moved out of `misc` catch-all into a substantive family with emit-level rationale.
- **New `sifr_adaptation` lane**: Correctly identified 2 runtime oracle cases that were previously lumped into compiler-side failures.
- **New `codegen_production_panic` family**: Correctly isolated the 0662 panic as a distinct failure mode with a specific crash location.
- **1203 absorbed into `ownership_and_borrow_emission_gap`**: Pass-1 suggested this if the family were broadened. v2 did it. E0502 (conflicting borrows) fits alongside E0382 (use after move).
- **Internal consistency**: All counts verified. No arithmetic errors, no duplicates, no omissions.
- **Lane determinism**: Every case's lane is determined by its family, making the classification reproducible and auditable.

---

## 6. Verdict

### `not_ready`

**Blocking issue**: 0729 is concretely misclassified into `binding_scope_and_capture_emission_gap` (E0277/E0596 are not binding/capture errors), and its lane lacks justification for rejecting `both`.

**Required before executive use**:
1. Fix 0729 family assignment and either justify `compiler_fix` with CalendarNode origin evidence or change lane to `both`.
2. Add a one-line note on the 0783 lane distinguishing it from the recursive_field `both` pattern.

**Acceptable risks (flag but don't block)**:
- Tree-related type_contract cases may need adaptation work despite `compiler_fix` lane — workstream lead should be aware.
- The 4 `other_codegen_build_gap` cases have no root-cause insight — acceptable for triage but will need investigation during execution.

After correcting items 1-2, the artifact is ready for executive use with the flagged risks documented.
