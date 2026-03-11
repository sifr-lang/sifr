# Phase 14 Gap 4: Structural Passes Hard Gate (No Raw-Text Fallback in Production)

Date: 2026-02-25  
Status: Done  
Parent: `issues/216-phase14-codegen-architecture-closeout-epic.md`
Merged PR: `#787`

---

## Problem

Structural passes still contain fallback behavior for `RawCode` by text parsing/scanning, which weakens the IR-only architecture.

Evidence:
- `crates/sifr_codegen/src/ir_imports.rs:34`
- `crates/sifr_codegen/src/ir_imports.rs:98`
- `crates/sifr_codegen/src/ir_imports.rs:165`
- `crates/sifr_codegen/src/ir_imports.rs:309`
- `crates/sifr_codegen/Cargo.toml:12` (`syn` currently in main dependencies)
- `crates/sifr_codegen/src/intrinsics/mod.rs:309` (test helper uses `RustExpr::RawCode`)

Strict checklist link:
- `internal_docs/phases/14_codegen_architecture_finish_checklist.md:163`

---

## Root Cause

Because `RawCode` remained in production paths, `ir_imports` preserved raw parsing collectors (`collect_from_raw_*`) to maintain behavior.  
This creates a mixed structural/text mode that is incompatible with a strict IR pass model.

---

## Desired End State

1. Structural passes operate only on typed IR in production.
2. Production pass pipeline fails fast if raw nodes are present.
3. No raw text parsing fallback is needed for production outputs.

---

## Scope

### In scope
- `crates/sifr_codegen/src/ir_imports.rs`
- `crates/sifr_codegen/src/ir_validate.rs`
- dependency cleanup in `crates/sifr_codegen/Cargo.toml` (where feasible)
- production assembly callsites in:
  - `crates/sifr_codegen/src/lib.rs`
  - `crates/sifr_codegen/src/entrypoints.rs`

### Out of scope
- Upstream elimination of raw nodes in generation paths (issue 219). This issue assumes that work is already in place.

---

## Implementation Plan

1. Add production pre-pass assertion:
   - detect any `RawCode` in `RustFile.items` tree
   - fail with explicit diagnostic instead of silently parsing raw text

2. Remove production dependency on raw collectors in `ir_imports`:
   - either delete `collect_from_raw_item_code`, `collect_from_raw_stmt_code`, `collect_from_raw_expr_code`
   - or gate them behind test-only/migration-only mode not used by production entrypoints

3. Ensure `collect_import_needs_from_items` remains deterministic and structural-only for production flows.

4. Add regression tests:
   - pass case: structural-only IR import collection works
   - fail case: raw node in production assembly triggers clear failure

5. Dependency cleanup:
   - after removing production raw parsing in `ir_imports`, drop any no-longer-needed parser dependencies from the main code path.
   - note: `syn` may still be required by `stdlib_filter`; if so, keep dependency but remove `ir_imports` runtime reliance on raw parsing.

6. Explicit test-only carveout:
   - preserve allowed test-helper `RawCode` usage (for example intrinsic registry tests),
   - enforce hard gates only on production assembly/entrypoints.

---

## Acceptance Criteria

1. Production entrypoints do not rely on raw-code parsing during structural passes.
2. If a raw node reaches production pass pipeline, generation fails explicitly.
3. `ir_imports` no longer includes production raw fallback parsing behavior.
4. Test-only `RawCode` helper usage remains allowed and documented (does not trip production hard gate).
5. Parser dependency footprint is reduced where possible; if not fully removable, rationale is documented (`stdlib_filter` dependency).
6. All structural pass tests and E2E parity tests pass.

---

## Validation

1. `cargo test -p sifr_codegen`
2. `cargo clippy -p sifr_codegen -- -D warnings`
3. `scripts/run_e2e_pass.sh`
4. `cargo test --workspace`
5. `cargo clippy --workspace -- -D warnings`

---

## Suggested PR Slices

1. Slice A: Add hard raw-node detection + production failure wiring.
2. Slice B: Remove/gate raw collector fallbacks from `ir_imports`.
3. Slice C: Add regression tests and update docs/checklists.
