I've reviewed the round 2 changes against the round 1 blockers and the underlying matrix/validator/docs/issue artifacts. The four validation suites pass. Findings below, severity-ordered.

## Findings

### Resolved — Phase contract / scope contradiction (round 1 critical 1)

`plans/phases/39_rust_interop.md:48-62` now explicitly carves out future-owned runtime/ecosystem rows from the self-contained clause, links them to the active follow-up issue, and bars them from stable-release advertising. M39.13 DoD (`:279-282`) is amended to "incomplete fixture families must have positive and negative evidence placeholders, a `future-owned-by-separate-phase` compatibility row, and a concrete active issue or phase owner," and the advanced exit gate (`:396-407`) carries the same constraint. The scope contradiction round 1 flagged is gone — Phase 39 now claims only what passes.

### Resolved — `future_owner` is mechanically constrained to a real owner (round 1 critical 2)

`verification/areas/rust_interop/checks/check_compatibility_matrix.py:22,127-138` now enforces `future_owner` to start with `plans/issues/active/` or `plans/phases/` AND requires the file to exist on disk. All 11 future-owned rows in `rust_interop_compatibility_matrix.json` reference the new `plans/issues/active/rust-interop-runtime-ecosystem-certification.md`, which exists and enumerates every future-owned capability (cross-checked row-by-row: bridge_type_matrix, opaque_resource_matrix, panic_boundary_wrapper_emission, async_runtime_reqwest, callbacks_call_scoped, callback_subscription_matrix, ecosystem_backend_certification, ecosystem_cli_certification, native_build_script, proc_macro_trust, cargo_locked_offline). The validator also blocks any future-owned row whose evidence is already passing in both directions (line 128-129) — so the category cannot be abused in reverse.

### Reframed, not resolved — README-only fixture chain for "supported" rows (round 1 critical 3)

Round 2 did not add executable cargo-probe/runtime artifacts under `verification/areas/rust_interop/fixtures/<id>/`. Every fixture directory is still a single README. The compatibility validator still treats `status: "passing"` in the fixture JSON as ground truth (line 121-126); it does not invoke Cargo, the compiler, or any runner. So a `supported` row with `execution_kind: cargo-probe` (e.g., `direct_crate_crc32`, `direct_crate_matrix`, `opaque_handle_tokenizer`, `local_bridge_blake3`, `async_ecosystem_matrix`) still bottoms out at a README that asserts pass — exactly the chain round 1 called out.

Round 2's defensible answer is "Phase 39 now only claims what is genuinely covered elsewhere (compiler/runtime/codegen tests) and pushes everything else to the active issue." That reframing is sound for the supported-but-contract-shaped rows, but it does mean the `verification/areas/rust_interop` area itself remains an inventory rather than an end-to-end gate. The round 1 reviewer's recommended "partial Option A" (wire the contract-passing fixtures) was not done. This is a residual gap worth surfacing in any closeout note, but not a structural blocker given the amended scope.

### Residual — stale staging language in earlier milestone status notes

The amended Phase 39 contract resolves the high-level contradiction, but two pockets of stale text remain:

- `plans/phases/39_rust_interop.md:227` (M39.10 status): "Runtime-observed crate-backed certification for `arrow`, `datafusion`, `polars`, `ndarray`, and `candle` remains staged for ecosystem closeout." Closeout is now Phase 39 + active issue, not "ecosystem closeout."
- `internal_docs/rust_interop_architecture.md:678-680`: same staging language.
- The M39.6/7/8/11 carry-forward notes (`:162`, `:177`, `:196`, `:240`) still say "tracked by `<fixture_name>`" rather than "tracked by `plans/issues/active/rust-interop-runtime-ecosystem-certification.md`." The redirection works transitively through the future-owned matrix row, but the indirection adds a hop a reviewer must take.

None of these is load-bearing — the validator and amended contract pin the truth — but they should be tightened before phase-level review so the docs don't read inconsistently.

### Residual — `bridge_type_matrix` vs M39.4 DoD

`plans/phases/39_rust_interop.md:141` ("Supported type mappings roundtrip through Rust bridge calls") still reads as DoD, but the positive roundtrip evidence is `planned` and the row is now future-owned. The Phase 39 amended contract absorbs this, but M39.4's per-milestone DoD wording was not adjusted, so M39.4 is technically "done" with one DoD bullet deferred to the follow-up issue. Worth a one-line amendment to M39.4 DoD pointing at the same active issue.

### Carried over from round 1 (not in scope for round 2 changes, still present)

- `same_workspace_crate` and `shared_bridge_crate`: `tier=1` with `execution_kind=contract-only` — the fixture matrix check doesn't cross-validate tier against execution_kind.
- `blocking_diagnostics`: `tier=0`, `execution_kind=compiler-diagnostic`, but `required_crates=["rusqlite", "rayon", "flate2"]` with feature pins. A diagnostic-only fixture can't meaningfully exercise the crate APIs.
- `check_stale_drafts.py` rejection-context detector remains permissive (any "no"/"reject" word in the line prefix counts as rejection context).

These are MEDIUM/LOW, predate round 2, and don't block phase-level review.

---

## Direct answers

**1. Are the round 1 critical blockers resolved by the amended phase contract and concrete active follow-up issue?**

Two of three: yes. (a) Scope contradiction is resolved by the amended `Depends on` / M39.13 DoD / exit gate. (b) `future_owner` is now mechanically constrained to a real owner file, and all 11 rows point at a real, scope-complete active issue. (c) The README-only fixture chain is *reframed*, not resolved — Phase 39's amended contract claims only what is covered, but `verification/areas/rust_interop` itself still doesn't run cargo or the compiler for `supported` rows. That reframing is defensible because the contract surface is covered by tests elsewhere in the workspace, but the round 1 reviewer's "wire the supported fixtures" recommendation was not done.

**2. Is M39.13 acceptable as an honest closeout of the Phase 39 supported surface, or does any blocker remain?**

Acceptable as an honest closeout of the *amended* supported surface. No hard blocker. Before phase-level review I would still want: M39.10 staging note rewritten to reference the active issue (phase doc `:227` and architecture `:678-680`); M39.6/7/8/11 carry-forward notes pointed at the active issue instead of just the fixture name; M39.4 DoD wording amended to acknowledge that roundtrip evidence is in the follow-up issue. These are documentation tightening, not gate-level work.

**3. Can a final phase-level review proceed after this milestone PR, assuming local validation stays green?**

Yes. The structural blockers are resolved and the validator/contract/active issue chain is now coherent. The phase-level reviewer should focus on (a) confirming the residual stale staging text gets cleaned up, (b) confirming the active follow-up issue is durably tracked outside Phase 39 (so it doesn't get lost between phases), and (c) ensuring Phase 40's stable-promotion criteria explicitly gate on the active issue's rows clearing, since the amended Phase 39 contract relies on that constraint.
