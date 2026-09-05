# Review — `hardening_3`, round 8 (immutable post-PR final audit)

**Scope:** commit `535b4a45943a0178b17c24b06ed5a3a20948d06a`, PR #3022, diff `origin/main...HEAD`. No files modified. All commands run synchronously to completion. Pass 7 treated as void.

## Packaging verification

The commit is exactly the reviewed implementation, and nothing else.

- Single commit on `fa288b02c` (`git merge-base origin/main HEAD` = `fa288b02c2828e...`), no merge commits.
- PR head `535b4a45943a...` == local `HEAD`. `gh pr view 3022`: 1 commit, **101 changed files, +3431/−349** — byte-identical to `git diff --stat origin/main...HEAD`. `mergeable: MERGEABLE`, `mergeStateStatus: CLEAN`, `isDraft: true`.
- `git diff --name-only origin/main...HEAD | grep -Ei 'ad-hoc-class|phase-40|receiver-place'` → no matches. The 8 excluded working-tree files (1 modified plan, 7 untracked reviews) are all uncommitted and absent from the diff. No omitted file: every `include_str!` target and every bound `test_file` resolves.
- **`internal_docs/typescript_go_architecture_transfer_guardrails.md` is correctly in scope**, contrary to rounds 5–6 excluding it. Its only change resyncs three `rust_interop_probe.rs` line anchors (49→50, 68→70, 211→168) that this PR's own edit to that file moved. I verified all three at the committed blob: `:50` = `probe.backend.cargo_manifest_path.is_file()`, `:70` = `cache_file.is_file()`, `:168` = `crate_feature_exists` / `fs::read_to_string`. All three are genuine filesystem probes; the inventory row stays accurate.

## Independently re-derived evidence data (my own traversal, not the shipped checker)

- **34/34** fixture manifests at `schema_version: 2`.
- **47** passing evidence sides, **all 47** carry `validation`; **0** passing sides unbound. **21** non-passing sides, **0** carry `validation` — no planned evidence falsely bound.
- **0** shared test bindings (`(test_file, test_name)` unique across all 47).
- All 47 `test_name`s resolve to **exactly one** `fn` definition in their declared `test_file`; all `step` values are `crate_tests`.
- Suites: `sifr_driver_lib` 31, `sifr_driver_generated_builds` 7, `sifr_lowering` 5, `sifr_runtime` 4. Profiles: `create-pr` 40, `merge` 7.
- **Weakest-profile derivation is correct.** From `verification/profiles/*.json`: `sifr_driver_generated_builds` is `modes:["full"]`/blocking and `create-pr` sets `legacy_facade.crate_tests: "smoke"`, so `merge` is its weakest executing profile — exactly the 7 `merge` bindings. The other three suites are `["smoke","full"]`/blocking → `create-pr`.
- **`--ignored` coupling holds on real data:** every `sifr_driver_generated_builds` binding is an `#[ignore]` test and every other binding is not; the generated-builds command is `["test","-p","sifr_driver","--lib","--","--ignored","--test-threads=1"]`. Zero mismatches.
- **Two-sided provenance on all claimed rows:** 17 `supported` + 5 `supported-through-bridge` + 1 `unsupported-by-design` = 23 rows, all with both sides `passing` + valid `validation`. 11 `future-owned-by-separate-phase`. `CLAIMED_SUPPORT_CATEGORIES` (`check_compatibility_matrix.py:27`) covers all three claimed categories, not just the self-tested one.
- **All 47 READMEs repeat the canonical `test_name`/`test_file`/`suite_id`/`profile`** — 0 mismatches. README is **not** validator input: the only reference is an `is_file()` existence check (`check_fixture_matrix.py:198`), and `_provenance_checks.py:40` documents the manifest load as README-free.

## Outcome-assertion binding — mutation-tested by me

I imported `_rust_test_outcomes.validate_bound_test_outcome` directly and ran every outcome-bearing binding against the real repo:

- **25** bindings with `expected_result` in `{diagnostic, runtime-error-state}`.
- **Controls that failed: 0** — every declared code/state is genuinely asserted by its bound test.
- **Mutations that passed: 0** — swapping `expected_diagnostic` to a bogus code, and flipping `closed`↔`poisoned`, is rejected in all 25 cases.

This independently confirms round 6's core claim and closes the round-3/round-4 mis-binding class mechanically.

## Execution-kind source class — round-5 finding 2 fix confirmed

`_provenance_checks.py:385-416` is now a symmetric guard, not the old `elif` chain: `runtime-observed` *must* be in `crates/sifr_runtime/`; **every other kind is rejected from it** (closing the negative-`cargo-probe` hole); positive `cargo-probe` additionally requires a generated-build suite or the `#[doc = "sifr-evidence: executes-cargo-probe"]` marker. The marker appears on exactly 3 tests (`rust_interop_contract_tests.rs:99,261`, `rust_interop_async_contract_tests.rs:104`).

Round-5 finding 1 (taxonomy collision) is fully resolved: identifiers are now `nonruntime_kind_failures` / `cargo_negative_runtime_failures` with hyphenated fixture ids `"contract-only-source"` / `"cargo-negative-source"`, and I ran the `coverage_matrix` area — **5 variants, 0 failures**, `verification_taxonomy` pass.

## Claim narrowing

Across the entire diff, `execution_kind` changes are **exactly 2 downgrades and 0 upgrades** (`git diff | grep execution_kind` → `+2 contract-only / −2 runtime-observed`), and **0 tier changes**. `async_runtime_core` and `callback_subscription_core` go `runtime-observed → contract-only` with capability text, notes, `internal_docs/rust_interop_architecture.md`, `internal_docs/sifr_sysroot_and_stdlib_architecture.md`, and the certification phase doc all updated consistently to disclaim runtime execution.

## Package-local bridge Cargo probing

`plan_package_bridge_probe` (`probe_planning.rs:52-98`) now pushes a real `PendingRustBridgeProbe` for `root == "bridge"`, and fails closed: no resolved sysroot runtime crate → `SIFR-RUST-CARGO-*` diagnostic + `return None` → the caller's `else { return; }` skips resolution. `probe_cargo_toml` moved to the new `rust_interop_probe_manifest.rs` and gained `cargo_package_name`, emitting `package = "..."` when the dependency alias differs. The new `source_prefix` injects `use <dep>::bridges as bridge;` after the inner attribute, covered by `prefixed_probe_keeps_inner_attributes_before_bridge_imports`. **Offline-safe:** the scenario's `blake3` is `{ path = "rust/blake3_backend" }`, a checked-in local crate — no registry or network dependency.

## Validation I ran

| Gate | Result |
|---|---|
| `rust_interop` area | **7 variants, 0 failures**; fixtures=34, rows=34 |
| fixture matrix self-test | **68 cases** |
| compatibility self-test | 4 cases (temp-tree only, incl. "claimed row missing provenance") |
| tiers / stale-drafts self-test | 6 cases / pass |
| `coverage_matrix` area (taxonomy) | 5 variants, 0 failures |
| `cargo test -p sifr_driver --lib` | **387 passed, 0 failed, 40 ignored** |
| ignored tier-1 cargo probes (`--ignored`, 6 tests) | **6 passed, 0 failed** (68s) — all three tier-1 rows build & run for real |
| `cargo test -p sifr_lowering` | **831 passed, 0 failed** |
| `cargo test -p sifr_runtime` | **55 passed, 0 failed** |
| `cargo clippy --workspace -- -D warnings` | clean |
| `cargo fmt --check` | pass |
| file-size guardrail | PASS (2828 files) |
| HIR guardrails | PASS |
| `git diff --check origin/main...HEAD` | clean |

Every numeric claim in the PR body that I could check independently reproduced exactly (387/40, 831, 55, 7 variants, 68 cases). Per instruction I did not restart a full `run_all_tests` lane; the 22/22 + 2944ms/5000ms + E2E 131/131 figures rest on round 6's own execution, and the diff touches no E2E fixture or runner file that could move the 131 count.

## Exit conditions

All met: schema v2 with the exact `validation` object ✔; distinct positive/negative tests added where a README pointed at a broad module (new `rust_interop_evidence_contract_tests.rs`, the two new local-bridge generated-build tests) ✔; all passing evidence migrated across 34 rows ✔; `check_fixture_matrix.py` validates suite/profile/file/test ownership ✔; `check_compatibility_matrix.py` requires two-sided provenance for all claimed categories ✔; README removed as validator input ✔; self-tests present for every enumerated case — missing suite `:490`, wrong profile mode `:494`, non-blocking suite `:497`, wrong step `:522`, missing test `:535`, duplicate test `:544`, ignored mismatch `:547`, path escape `:672`, status/provenance mismatch `:675`, README-only claim `:681`, commented pseudo-test `:693`, shared evidence test `:710-724`, plus the two execution-kind cases `:743,:760` ✔.

## Non-blocking observations

- `fixtures/opaque_resource_core/README.md:6` — the lead sentence still says "the executable evidence is the named runtime test filter below," above bullets citing the broad `cargo test -p sifr_runtime interop`. Its own Canonical section 13 lines later declares `fixture.json` authoritative and names the two exact tests. This is the only remaining instance of that prose in any fixture README; it is not validator input and cannot cause a false gate pass, but it reads against the milestone's own objective. Worth a one-line edit in `hardening_5`.
- `fixtures/local_bridge_blake3/examples/blake3.sifr:6-7` declares `@rust(blake3.hash, ...) def blake3_hash(input: bytes) -> bytes`, while the local backend's `hash` returns `u64`. Package examples are metadata-only and never compiled, so this is illustrative-only, but it is type-incorrect as written.
- The local `blake3` backend is an FNV-1a stub crate *named* `blake3` (pre-existing, unchanged by this PR). `required_crates: ["blake3"]` could be misread as upstream blake3 coverage; the row's capability ("package-local bridge binding") and evidence are about the bridge mechanism, so no claim is overstated.
- This PR replaces the local-bridge fixture's `panic=map_error(bridge.blake3.map_panic)` shape with `panic=trusted_no_panic` plus a `rust-no-panic` trust entry. Necessary — the old form had a dangling `map_panic` reference and was never compiled — but the row no longer illustrates panic mapping. `panic_boundary` still owns that.
- `expected_result: "pass"` positives (21 rows) have no mechanical outcome binding; the tests do assert acceptance, but there is no canonical shape to check. Unchanged from round 6.
- Round-6 code-shape nits still open: `let … else { return }` embedded in a struct-literal field (`rust_interop.rs:303-313`); ~25 lines of probe-planning duplicated between `probe_planning.rs:52-98` and `rust_interop.rs:401-429`; two independent Rust maskers (`_rust_test_evidence.py:323`, `_rust_test_outcomes.py:174`); `_validate_command_filters` handles only `--skip`; `dependency_features`' `segments[1]`-as-feature heuristic now also reaches package bridges. Zero current instances trip any of these, and all fail closed.
- Near-cap hand-maintained files: `rust_interop.rs` 896, `rust_interop_contract_tests.rs` 872, `_provenance_checks.py` 865, `check_fixture_matrix.py` 861, `rust_interop_probe.rs` 857 (cap 900). `hardening_4` should not add to these.
- The plan's progress row reads `hardening_3 | review approved; PR pending`. Correct at commit time; update to `merged | PR #3022` per the documented step-5 flow.

No commit/PR packaging issue, omitted file, false evidence, or regression found. The claims are mechanically no stronger than executed evidence, both historical mis-binding classes are now impossible rather than merely corrected in data, and every gate I ran is green.

Actionable findings: 0. SATISFIED.
