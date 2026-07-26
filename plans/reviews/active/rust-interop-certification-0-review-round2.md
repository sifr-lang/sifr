## Re-audit of `certification_0` (working tree vs `7554f89b5`)

**Verified clean.** Area run: 10 variants / 0 failures (5 suites, 5 registered in all four profiles). Runner self-tests pass. Recorded inventory reproduces exactly: 36 compat rows / 36 fixture rows / 36 schema-v2 manifests, 47 passing + 25 planned (both matrices), categories 17/5/1/13, execution kinds 13/4/10/9, 44 aliases, 23 claims (= 36 − 13 future-owned), 5 suites / 10 cases. The five contract-only rows changed in `notes` only. Both new rows are honest (`future-owned-by-separate-phase`, `future_owner`, both directions `planned`, tier 2/4 + `runtime-observed` legal, `future-owned`/`future-owned-diagnostic` matching the `ecosystem_backend_certification` convention). `cargo fmt --check`, file-size guardrail (2833 files), HIR guardrail, `git diff --check` all pass.

### Round-1 findings: all 14 resolved

1. **Resolved.** `profile_runner.py:255-258,272-285` now runs `cargo fetch --locked` (env-stripped of `CARGO_NET_OFFLINE`, `self.env` is a full `os.environ` copy so the pop is effective) before `enable_offline_cargo()`, via `cargo_setup.py:19-23`. That the fetch really populates optional-only deps is corroborated: `candle-core-0.11.0.crate`, `polars-0.54.4.crate`, `datafusion-54.1.0.crate` are in the registry cache although nothing compiles them, and `cargo fetch --locked --offline` succeeds.
2. **Resolved (metric).** README no longer substitutes per-case sums. See finding 8 for the residual.
3. **Resolved.** Phase 40 `:56-62,399-402,417-423` is confirm-only; issue `:221-232` now says this item registers. No contradiction remains.
4. **Resolved.** "all **five** registered suites" (`README.md:128-130`).
5. **Resolved.** `main()` sweeps every `docs/**/*.md(x)` (`check_stable_support_claims.py:490-494`); I confirmed live that a secondary doc triggers all three rules (`cannot be presented as runtime evidence`, `advertisement must be in the canonical table`, `must be described as future-owned and planned`). The overclaim rule is now independently reachable via prose. Residual: findings 3, 4.
6. **Resolved.** 16 cases including omission, capability drift, duplicate ids, schema/source/role drift.
7. **Resolved.** `default-features` is now compared for every crate (`_crate_catalog.py:120-126`) and all 44 entries state it explicitly; a `[features]` table is rejected (`:73-76`) with a mutation case. Residual: findings 5, 6.
8. **Resolved.** Metadata expectations derive from `feature_policies` (`_crate_catalog.py:145-151`).
9. **Resolved.** `README.md:62-66` scopes the claim and disclaims sysroot-vendor coverage.
10. **Resolved.** Duplicate paragraph gone.
11. **Resolved.** "is validated against" (`docs/rust-interop.mdx:76`).
12. **Resolved.** Round-1 artifact populated (9,840 bytes). See finding 12 for round 2.
13. **Resolved.** `docs/rust-interop.mdx:79-82` names both rows; the checker now *requires* it (`:191-195`).
14. **Resolved.** `check_fixture_matrix.py` 880 → 757 lines via `_matrix_inventory.py`.

### New findings

**1. MEDIUM — `Cargo.lock` carries a broad, unrelated dependency update outside this item's scope.** The diff is not "catalog additions only": 471 package entries added, 104 removed, 124 names with changed version sets, and 12 package families dropped (`wit-bindgen 0.51.0`, `wasmparser`, `wit-component`, `wit-parser`, `wasm-encoder`, `id-arena`, …). At least **30 names got new versions that are unreachable from `sifr_rust_interop_catalog`** in the lock graph — e.g. `insta 1.47.2 → 1.48.0` (a dev-dependency of `sifr_codegen`/`sifr_lsp` only, `cargo tree -i insta` confirms), plus `ignore 0.4.25→0.4.31`, `globset`, `bstr`, `time 0.3.47→0.3.54`, `toml`/`toml_edit`/`toml_writer`, `winnow`, `camino`, `serde_with`, `ref-cast`, `thin-vec`, `console`, `rust_decimal`. Adding optional deps cannot cause these; a full `cargo update` was run. AGENTS.md treats lockfile diffs as *intentional* dependency-graph changes, and the item's Post-item inventory only claims the 44 aliases are pinned and present. This silently re-bases the compiler's own snapshot/manifest/walker dependencies inside a matrix-modelling PR. Regenerate the lock from base with only the catalog-required resolution, or record and justify the update explicitly.

**2. MEDIUM-LOW — nothing regression-tests that the runner executes the setup.** `prepare_cargo_cache` / `enable_offline_cargo` appear nowhere outside `profile_runner.py` (grep is empty). The self-tests (`selftest.py:96-101,141-142,309-321`) only assert what `cargo_setup_command()` returns. Deleting the `prepare_cargo_cache()` call at `profile_runner.py:256` leaves all self-tests green and recreates round-1 finding 1 exactly — the one failure mode this PR exists to close.

**3. LOW-MEDIUM — the docs-wide sweep is untested.** Every one of the 16 self-test cases calls `_validate(..., public_documents=None)` (`check_stable_support_claims.py:467-468`); the `public_documents` merge path (`:172-182`) is exercised only by production data. Removing the `docs/` rglob in `main()` (`:490-494`) would keep the suite green while collapsing the gate back to the single marker block.

**4. LOW — prose gating keys off hardcoded id sets with no binding to the matrix.** `RUNTIME_DEFERRAL_IDS` / `COMPILE_SCOPE_IDS` (`check_stable_support_claims.py:21-31`) duplicate data that is derivable (`category == future-owned-by-separate-phase && execution_kind == runtime-observed`; `execution_kind == contract-only`). No check asserts the sets match the matrix, so a future-owned runtime row added by `certification_1`+ or a new contract-only row silently escapes both prose rules.

**5. LOW — `candle` default-features is pinned by a hardcoded special case, not by policy data.** `_crate_catalog.py:121-122` (`if crate == "candle": expected_default = False`) overrides `EXPECTED_FEATURE_POLICIES["candle"] = {"backend": "cpu-only"}` (`_matrix_inventory.py:124`), which carries no `default_features` key. The canonical policy table and the enforced pin can therefore disagree with all gates green — the same self-referential shape as round-1 finding 8.

**6. LOW — untested branches in the catalog mutation suite.** `_crate_catalog.py` cases cover 8 mutations but never: unexpected extra dependency (`:69-72`), a non-`optional` dependency (`:104-105`), a wrong `package` alias (`:106-111`), or missing `[package.metadata.sifr-rust-interop]` (`:142-144`).

**7. LOW — offline/network governance drift between the policy doc and the profile data.** `profile_policy.md:21-25` was updated to sanction a registry fetch inside the profile run, but `create-pr.json:38-51` still declares `network_policy.live_network_allowed: false`, `execution_sandbox.external_network: "forbidden"`, and `reference_host.notes` "must not depend on external network during profile execution" (same in merge/nightly/release), and `profile_policy.md:35-36` still names `doctor` as *the* setup boundary. On a cold cache the create-PR profile now does depend on the network from inside its own process.

**8. LOW — the budget measurement is still not recorded.** `README.md:142-146` promises that "`certification_0` records the profile-enforced step wall time" but records no number, deleting the previous concrete 3,244/3,479 ms without replacement; the checklist item is unchecked (`certification.md:181-182`). Measured warm equivalent here: **3.60 s / 3.62 s** for the five suites against the 5,000 ms blocking budget, of which the new `cargo fetch --locked --offline` subprocess is ~0.42 s. The number must land in the README before merge, and the ~1.4 s headroom is worth stating explicitly since it now varies with cargo package-cache lock contention.

**9. LOW — the setup fetch is invisible to lane accounting.** `prepare_cargo_cache` emits `[sifr-profile-setup]` only (`profile_runner.py:283`), not `[sifr-lane-step]`, so `reports.py:39` never attributes its time to any step, while `write_time_file` still counts it in lane wall time. Cold-cache cost is now ~471 extra packages / ~100 MB of `.crate` downloads (measured from the cache) charged silently against the 15-minute cold create-PR budget, on every profile including `python-interop-live`.

**10. LOW — `CERTIFICATION_CRATE_COUNT: usize = 44` (`crates/sifr_rust_interop_catalog/src/lib.rs:9`) is unreferenced and unvalidated.** Nothing ties it to the dependency table or `REQUIRED_CRATES`; it will silently go stale the first time a row PR adds a crate.

**11. LOW — style regression from the extraction.** `check_fixture_matrix.py:33-34` has a single blank line before `def main`, unlike every sibling module in the directory.

**12. LOW — `plans/reviews/active/rust-interop-certification-0-review-round2.md` is 0 bytes.** The canonical evidence rule (`certification.md:87`) requires a review artifact per round; round 1's is now populated, round 2's is not.

*Notes, not findings:* the `stable-candidate` sweep covers `docs/**` only, so the Stable Release Constraint's "release checklist" surface (`internal_docs/distribution_pipeline.md`, `scripts/distribution/`) remains ungated — reasonably Phase 40's ownership, but worth stating in the item. The hardcoded `/tmp/sifr-rust-interop.bin` in `zero_copy_runtime_matrix/examples/memmap2.sifr:12` stays inert while the row is planned but must adopt the temp-dir hermeticity rules at `certification_7`. I did not rerun the authoritative profiles; per your instruction the create-PR readonly-check-doctor timeout is excluded.

NOT SATISFIED
