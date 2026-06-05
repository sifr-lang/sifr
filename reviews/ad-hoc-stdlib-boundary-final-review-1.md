Verdict: **CHANGES_REQUESTED**

I verified all seven exit gates and found a docs gap that contradicts the M6 contract clause "Repository search confirms no stale references to `sifr_hir`, `crates/sifr_hir`... in user-facing documentation." The validation evidence rationalizes leftover hits as "active migration contract and execution history only," but several stale references sit in **active** architecture sections, not historical ones.

## Exit gate verification

1. **sifr_stdlib + sifr_ir + sifr_lowering exist with intended ownership** — PASS. `crates/sifr_stdlib/src/{contract,dependencies,intrinsic_registry,source_inventory}.rs`, `crates/sifr_ir/src/lib.rs` (HIR types only), `crates/sifr_lowering/src/lower/*` (split files, no monolithic `lower.rs` or `stdlib.rs`).
2. **sifr_hir removed** — PASS. `crates/sifr_hir` absent; `cargo metadata` returns only `sifr_stdlib`, `sifr_ir`, `sifr_lowering`.
3. **Generated Cargo dep decisions centralized in sifr_stdlib** — PASS. `crates/sifr_stdlib/src/dependencies.rs` holds the registry; `sifr_codegen` consumes via `use sifr_stdlib::dependencies::*` (see `crates/sifr_codegen/src/cargo_manifest.rs`).
4. **codegen/lint consume sifr_ir without depending on lowering** — PASS. `crates/sifr_codegen/Cargo.toml` has `sifr_ir` + `sifr_stdlib` (no `sifr_lowering`); `crates/sifr_lints/Cargo.toml` has only `sifr_ir`.
5. **Dep-direction guardrails in create-pr validation** — PASS. `scripts/check_source_crate_dependency_direction.py` is invoked from both `create-pr` and `merge` profiles in `scripts/run_all_tests.sh`; CI workflow `local-first-validation.yml` mirrors it.
6. **Docs + execution checklist updated** — **FAIL (partial)**. See below.
7. **Full `run_all_tests.sh` ran** — PASS. 574.22s, 31 hardening variants, 0 failures; report at `target/validation_lane_reports/merge.latest.json`.

## Blocking fixes

- `internal_docs/architecture.md:92` — Active "HIR Decomposition Contract" → "What Are HIR's Concerns?" still links `crates/sifr_hir/src/lower.rs`. Update to `crates/sifr_lowering/src/lower/`.
- `internal_docs/architecture.md` "Audit & Re-Decomposition Plan" subsection (≈ lines 104–250) — referenced ≈ 20+ times: `crates/sifr_hir/src/lower.rs`, `crates/sifr_hir/src/hir_nodes.rs`, `crates/sifr_hir/tests/...`, plus the section "Layout under `crates/sifr_hir`" at line 156. The M6 contract requires removing stale references. Either:
  - retarget paths to `sifr_ir` / `sifr_lowering`, or
  - explicitly mark the section as historical/superseded by the new "Crate Boundary Refactor (Sifr_Stdlib + Sifr_Ir + Sifr_Lowering)" section.
- `internal_docs/compiler_pipeline.html:1147,1149` — inline comments `// HIR (sifr_lowering)` and `(sifr_lowering)` annotations on HIR data conflict with the prose update at line 519 ("HIR data (`sifr_ir`) … produced by `sifr_lowering`"). Reconcile so HIR data is attributed to `sifr_ir`.

## Verification command for the fix

After updating, the grep claimed-PASS in the execution doc should also pass when narrowed to live architecture sections, e.g.:

```bash
rg -n "crates/sifr_hir|sifr_hir/src" internal_docs docs
```

Should return no hits, or only matches inside explicitly historical/migration-contract blocks.

## Non-blocking residual risks (after the fixes land)

- `issues/ad-hoc-stdlib-ir-lowering-boundary-refactor-execution.md:118` still lists the M6 PR link as "pending" — expected until the docs PR is opened; remember to close out before merging.
- M6 merge-profile run reports an advisory "e2e group skew is high"; not a blocker for closure but worth tracking in phase 35 perf work.
- The new "Crate Boundary Refactor" section in architecture.md uses `[x]` checkmarks for completed milestones; consider folding it into the canonical pipeline description once docs settle so future readers don't have two competing layout descriptions.
