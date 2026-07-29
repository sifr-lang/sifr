# Review — Rust interop runtime ecosystem certification, `certification_7` (round 9)

**Base:** `53cc9c4bf36762d39a0b372402d202589f920c2e` (`origin/main`, = merge-base)
**Exact head:** `a487ca004ae5d6b736a3536043a20b0b78c94723` (committed 2026-07-29 03:59:24 +0300)
**Diff:** 51 files, +1982 / −206. No files modified — review only.

## Validations run at exact head

| Check | Result |
|---|---|
| `cargo test -p sifr_driver --lib zero_copy -- --ignored --test-threads=1` | **pass** — 3/3 (`test_build_zero_copy_crate_backed_view_lifecycle`, `test_check_zero_copy_borrow_escape_and_invalid_mutability_rejected`, `test_check_zero_copy_view_send_sync_obligations`), 26.0s |
| `uv run --project verification --locked python -m sifr_verify areas run --area rust_interop` | **pass** — variants=10, failures=0; matrix rows=36, fixture_rows=36, tiers=5, claims=30, self-tests 139+33+20+6+5 cases |
| `cargo clippy --workspace -- -D warnings` | **pass** |
| `cargo fmt --check` | **pass** |
| `python3 scripts/check_hir_maintainability_guardrails.py` | **pass** |
| `python3 scripts/check_file_size_guardrails.py` | **pass** (2955 files, limit 900) |
| `git diff --check` | **pass** |
| `areas run --area coverage_matrix` (integration-regression probe) | **pass** — variants=6, incl. new `generated_code_release_divergence` self-test (15 mutations) |

Independent inventory recount matches the issue's post-item block **exactly**: 36 compat rows / 36 fixture rows / 36 schema-v2 manifests; 60 passing + 12 planned evidence directions; 18 `supported`, 11 `supported-through-bridge`, 1 `unsupported-by-design`, 6 `future-owned`; 13 cargo-probe / 4 compiler-diagnostic / 10 contract-only / 9 runtime-observed; 44 required crate aliases; 60 package + 17 scenario examples; 30 stable claims.

## Findings, by severity

**1. Blocking — the authoritative merge gate is red and no lane report exists at the exact head.**
`target/validation_lane_reports/merge.latest.*` (03:58:58) failed at `performance_budget_checks`: `check-single-file-001-arithmetic (perf.check.single.arithmetic) median_ms regression: measured=1374.613 threshold=1334.139 waiver_status=no_waiver` (+3.03%, `waiver_status=no_waiver`). Every earlier blocking step was green. Separately, **both** retained reports predate `HEAD`: create-PR finished 03:21 and merge 03:58:58, while `a487ca004` was committed 03:59:24 — so the fully-green create-PR report (all 24 steps `pass`) was produced on `88b1eb31d`, not the integrated head. I re-validated the head delta directly (see below), but neither lane has an authoritative run at `a487ca004`.

*Attribution:* not plausibly the diff. `check-single-file-001-arithmetic` is `sifr check` on a single arithmetic file with no Rust interop declarations; every path this diff adds is gated on `RustInteropDecoratorKind::ZeroCopy`/`View` declarations reaching `apply_package_rust_interop_metadata`, and the two `sifr_codegen` additions are `pub` re-exports of existing logic (no extra work on any path). Corroborating environmental signal: `sample_count=5` (p95 suppressed for all 39 benchmarks as under-sampled), the retained earlier merge run of the same lineage *passed* this same benchmark at the same threshold, the two failing attempts hit *different* unrelated cases, and that earlier run then aborted in e2e with `failed to create batch crate dir: No space left on device` against a 50 GB generated-artifact cache. I am not waiving the gate — it must be rerun green at `a487ca004`.

**2. Low — Python import-block style regression in a touched file.**
`verification/areas/rust_interop/checks/_scenario_checks.py:29-35`: the new `_scenario_zero_copy` import is inserted out of alphabetical order (before `_scenario_source_checks`), and the blank line separating the import block from `REQUIRED_SCENARIO_EXAMPLES` was deleted. No gate covers this (the repo has no Python linter over `verification/`), so it is cosmetic only.

**3. Note — `view=` identity is unenforced for generated-record returns.**
`zero_copy_validation.rs:238` accepts any `view=` when the Ok slot is a generated bridge type (`is_rust_generated_bridge_type_path`). This is the intended round-3/4 scoping (opaque crate-backed handles get exact identity; contract-only records keep advanced-data metadata validation) and is documented in `internal_docs/rust_interop_architecture.md`, with `package_rust_interop_preserves_generated_record_view_contract` pinning it. Recording it as a known scope limit, not a defect.

No findings on panic safety, provenance, hermeticity, safe-Rust behavior, or doc accuracy — details below.

## Substantive verification

- **Panic safety.** Every added `unwrap`/`expect`/index in Rust source is in `#[cfg(test)]` code. `declarations[0]` (`zero_copy_validation.rs:122`) is provably non-empty — the slice is a `BTreeMap` entry built by pushing declarations, and its canonical path equals the map key, so the obligation-map key is correct. Ordering is sound: `validate_zero_copy_contracts` (`rust_interop.rs:164`) populates `zero_copy_probe_obligations` before `resolve_declaration` → probe planning reads it. `returned_ok_type` is fully fallible (depth-aware top-level comma scan, `saturating_add/sub`). The broadened `requires_send/requires_sync` on same-path Function/View probes feeds only the probe-plan digest (`rust_interop_plan.rs:566`), so it coarsens cache keys and changes no semantics.
- **Safe-Rust zero-copy behavior.** The bridge asserts rather than assumes: `Bytes::from(data)` pointer identity vs the captured `Vec` pointer, `mmap` address identity across `make_read_only()`, `bytes_view` pointer identity plus the pre-move `b'S'` mutation surviving `drop(owner)`, `bytemuck::try_cast_slice` == `[9, 8]` at the sealed address, and `Packet::ref_from_bytes` at the same address. Failures become `ViewError`, not panics; the only `copy_from_slice`/index calls use compile-time-constant lengths, so `trusted_no_panic` is honest. No `unsafe`, enforced by `reject_unsafe_rust` with a self-test that mutates `pub fn create` → `pub unsafe fn create`.
- **Hermeticity.** All five example lock entries match the root `Cargo.lock` exactly (`bytes 1.11.1`, `bytemuck 1.25.2`, `memmap2 0.9.11`, `zerocopy`/`zerocopy-derive 0.8.48`, plus `proc-macro2 1.0.106`, `quote 1.0.45`, `syn 2.0.117`, `unicode-ident 1.0.24`). `zerocopy[derive]` is frozen in the catalog, `_matrix_inventory.EXPECTED_FEATURE_POLICIES`, both zero-copy fixture manifests, and both matrices, with a `check_fixture_matrix` self-test for the missing-policy case.
- **Provenance.** All four evidence directions `include_str!` the exact checked-in fixtures (`ZERO_COPY_EVIDENCE`/`ZERO_COPY_NEGATIVE`, and `ZERO_COPY_BYTES_*`/`ZERO_COPY_VIEW_MATRIX_*` in the contract tests). The scenario self-test covers 12 mutations spanning pin drift, derive drift, trust drift, and six behavioral-token drifts.
- **Integration regression from the Phase 40 main merge.** The `88b1eb31d..a487ca004` delta is docs/plans plus `generated_code_quality`/`coverage_matrix`/release-profile assets only — zero Rust source, zero rust-interop files. The new `generated_code_quality:release-full` suite is release-profile-only (`verification/profiles/release.json`); `create-pr` and `merge` still map to `smoke`/`representative`. `coverage_matrix` passes at head with the two new surfaces (`generated_rust_toolchain_survival`, `codegen_snapshots`) green. **No integration regression affects certification 7 or its gates.**
- **Unrelated working-tree paths preserved.** `editor_integrations` (submodule pointer → `a980835e6`, `heads/codex/bump-vscode-beta-14-1-ga980835`) and untracked `.cert5probe/` are both intact and absent from the branch diff (`in-diff-count=0`). `plans/reviews/active/rust-interop-certification-7-review-round-9.md` exists as an empty untracked placeholder; I did not write to it.
- **Docs accuracy.** `docs/rust-interop.mdx` narrows the deferral list to `advanced_data_runtime_matrix` alone, adds a scope-bounded paragraph, and the generated claims table gains the row — all validated by the area's stable-candidate check (claims=30). The transfer-guardrail anchors `rust_interop_probe.rs:53/73/141` resolve to the three real reads (`cargo_manifest_path.is_file()`, `cache_file.is_file()`, `fs::read_to_string(... Cargo.toml)`), correctly shifted by the one added struct field.

## Prior-finding closure

Rounds 1–6 all confirmed closed at this head: exact Ok-slot opaque-handle identity with prefix/container regressions (`rejects_view_type_prefix_alias`, `rejects_view_nested_in_list_return`); view-trait classification with `include_stderr=false` and a test asserting `children.is_empty()`; all four Send/Sync probe forms unit-tested (`rust_interop_probe_tests.rs`); `zerocopy[derive]` frozen; sealed mmap reinterpreted through both `bytemuck` and `zerocopy`; file sizes under 900; bridge-type diagnostic preserved for propagated unsupported Result slots (`signature_has_unsupported_type` now also honors `unsupported_reason`, asserted by `preserves_unsupported_return_diagnostic` including "no ZC diagnostic"); token inventories relocated to scenario-owned modules; identity scoped to opaque handles with `rust_opaque_handle_type` reused from codegen; generated-path recognition moved into codegen with malformed-path unit tests; both positive and both negative directions bound to their exact checked-in fixtures; `copy_fallback=True` present with the paired opaque view; rejected keys named verbatim in diagnostics (`unsupported \`@rust.view(...)\` key \`mutable\``, `` key `copy_fallback` ``). Rounds 7–8 reported no findings; nothing has reopened.

## Verdict

**NOT SATISFIED**

The implementation, evidence, provenance, hermeticity, safe-Rust runtime behavior, and documentation are sound — no actionable defect in the diff, and all seven required focused checks plus the integration probe pass at `a487ca004`. The sole blocker is gate evidence: the authoritative merge lane is red at `performance_budget_checks` (+3.03% on an unrelated benchmark, `no_waiver`), and neither the create-PR nor the merge report was produced at the exact head. Note also that only the 3% failure survives in the retained reports; the 2.6% attempt has been rotated out, so I can confirm one of the two described failures, not both. Rerun both lanes at `a487ca004` — I expect them green given the attribution analysis, but that is a prediction, not evidence.
