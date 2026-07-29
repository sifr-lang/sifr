# Review round 3 — `certification_8` (crate-backed advanced data runtime)

Re-reviewed the full working diff against `origin/main` (merge-base `94a5fec67`) plus all 13 intended untracked scenario files, the new driver test module, and `_scenario_advanced_data.py`. No files modified.

## Round 2's blocker B4 — resolved, and the new wording is accurate

I verified each clause of the corrected scenario `README.md:17-20` independently:

- **"exact seven-entry"** — `examples/advanced_data_runtime/sifr.toml:30-38` declares exactly seven, and `EXPECTED_NATIVE_LINKS` (`_scenario_advanced_data.py:55-63`) pins the same list under `_require_exact_trust`'s `actual != expected` equality.
- **"covering the locked graph on arm64 and x86_64"** — re-derived from the pinned build script: `blake3-1.8.5/build.rs:367-372` emits `blake3_neon` on little-endian aarch64; `build.rs:344-361` reaches `build_sse2_sse41_avx2_assembly` (`compile()` at 231) and `build_avx512_assembly` (`compile()` at 280) on x86_64 with a C compiler. `NoCompiler`/`NoAVX512` emit strict subsets; `blake3_avx512_intrinsics` needs `should_prefer_intrinsics()` or x86_32. All three blake3 names plus the four arch-independent ones are in the envelope. The wording no longer claims a single host emits all seven — which is the exact defect round 2 flagged.
- **"post-build audit rejects any emitted build-script link output outside that envelope"** — this is precisely `validate_native_link_evidence` (`crates/sifr_driver/src/build/materialize.rs:329-361`): it iterates *observed* `linked_libs` and rejects any name not in the trusted set. Subset enforcement, no reciprocal must-be-observed check, so a cross-arch superset stays fail-closed per host.

Parent `fixture.json` README (`README.md:17-20`, observed→"envelope covering … arm64 and x86_64 validation hosts") and `docs/rust-interop.mdx:189-191` (exact→"the architecture-specific native-link envelope covering the locked default-feature graph on arm64 and x86_64") are both tightened correctly. `internal_docs/rust_interop_architecture.md:951-953` ("accepts only the exact arm64/x86_64 native-output envelope **declared** by this locked graph") is also accurate.

**New verification beyond round 2:** I checked that blake3 is the *only* arch-conditional emitter in the graph, which round 2 had not established. `ring`, `lz4-sys`, `bzip2`, and `flate2` are present in the scenario `Cargo.lock` and would emit link names if built, but all are behind non-default optional features (`ring` is reachable only via `object_store`/`reqwest`/`rustls`); the host's `target/debug/build` contains exactly `blake3`, `liblzma-sys`, `onig_sys`, `psm`, `zstd-sys`. So no Linux-only name can appear outside the envelope. I also confirmed all 491 external `(name, version)` pairs in the scenario lock are a strict subset of the root lock — zero divergence, genuinely offline-safe.

## Independent gate reruns (read-only, not taken on report)

`PYTHONPATH=verification/runner python3 verification/areas/rust_interop/runner.py` → `variants=10, failures=0`, 152 mutation cases, `fixtures=36 diagnostics=10 crates=44 package_examples=60 scenario_examples=18`, `claims=31`. `cargo test -p sifr_driver -- rust_interop` → 167 passed / 0 failed / 24 ignored. `cargo clippy --workspace -- -D warnings` → exit 0 (the `--all-targets` warnings in `rust_interop_contract_tests.rs:729` etc. are all pre-existing; the new test at :416 uses the correct `..TrustPolicy::default()` form). `cargo fmt --check`, `git diff --check`, Python compile, HIR guardrail, file-size guardrail (2960 files, limit 900) all pass.

Recomputed inventory from the data files: 36/36 rows, `18 supported / 12 supported-through-bridge / 1 unsupported-by-design / 5 future-owned`, `13/4/10/9` execution kinds, `62 passing / 10 planned`, 31 claims = 31 doc table rows, `runtime_deferrals: []`. Every number matches the plan (`…certification.md:296-303`). Only `advanced_data_runtime_matrix` was promoted; the three narrower rows stay `contract-only` (`docs/rust-interop.mdx:223-227`).

## Findings

**F1 — `internal_docs/sifr_sysroot_and_stdlib_architecture.md:157-165` still lists `advanced_data_runtime_matrix` as separately owned.** (severity: low — non-blocking)

It states the matrix "currently has 18 supported rows, **8** bridge-supported rows, 1 unsupported-by-design row, and **9** rows owned by separate certification work" and names `advanced_data_runtime_matrix` in that list. Actuals are 12 and 5, and that row is now `supported-through-bridge`.

I checked whether this milestone caused it: at the merge-base the doc already said 8/9 against actual 11/6 — certifications 5, 6, and 7 promoted `opaque_resource_matrix`, `callback_subscription_ecosystem`, and `zero_copy_runtime_matrix` without updating it. This change extends existing drift by a fourth entry. Non-blocking because the error is *conservative* — the doc over-lists separately-owned rows, so it cannot produce a false stable-support claim under its own rule at lines 164-165 — nothing validates the file, and correcting it properly means fixing three prior merged milestones' drift, outside this milestone's scope. Worth a follow-up.

**F2 — plan/review artifact hygiene.** (informational)

`plans/reviews/active/rust-interop-certification-8-review-round-3.md` exists but is 0 bytes. The plan's Review and validation notes (`…certification.md:934-946`) link only Round 1; the round-2 artifact is untracked and unlinked. The final checklist box at line 931 is correctly still `[ ]`.

### Carried-forward observations — all re-confirmed, none violating an acceptance criterion

- `dlpack::Capsule` (`dlpack.rs:7`) and `sifr_arrow_bridge::schema::RecordBatch` (`lib.rs:3-6`) remain never-constructed markers satisfying only the validator's crate-name-prefix check; the public-doc caveat is present (`mdx:187-188`).
- The `"…was already transferred"` branches (`dlpack.rs:58,62`) and `observe_state`'s `ndarray.as_ref()` else-arm (`tensor.rs:120-122`) are unreachable because `own` consumes the handle; the `Option` is still required for `.take()`.
- N4 unchanged: `record_declared_native_links` computes `canonical_target_path` (`trust_validation.rs:16`) while `trusted_native_links` (`materialize.rs:284-303`) flattens to a bare name set. Not a new bypass — the manifest is still the sole authorization — but the recorded path precision is unused.
- `mark_closed(__generated_glue::token())` from a scenario bridge (`record_batch.rs:75`, `tensor.rs:83`, `dlpack.rs:50`): `GeneratedGlueToken` is freely constructible by any crate, so close semantics are bridge-authored, not compiler-enforced. Exact precedent at `zero_copy_runtime_matrix/.../bridges/zero_copy.rs:146`, merged in certification_7.
- `create` resets `RELEASED_*` but not `ACTIVE_*` (`record_batch.rs:57`, `tensor.rs:56`) — harmless in this single-shot scenario.
- `_scenario_advanced_data` import at `_scenario_checks.py:16-20` breaks the alphabetical block. Cap proximity: `_scenario_checks.py` 891, `rust_interop.rs` 898 — both pass, and the milestone's logic was correctly extracted into the 397-line `_scenario_advanced_data.py`.
- `positive/crate_backed_arrow_tensor_roundtrips.sifr` and `src/main.sifr` are byte-identical apart from the entry name with no enforced agreement — matches the `zero_copy_runtime_matrix` convention.
- The x86_64 leg is still derived from build-script source rather than executed here; subset enforcement makes a wrong entry a fail-closed build error, not a false certification.

### Out of scope — preserved unmodified, must not enter the PR

`editor_integrations` submodule bump, dirty `verification/areas/algorithmic_compatibility/corpora/leetcode` submodule, `plans/phases/43_interoperability.md`, `.cert5probe/` (5 files), `.claude/` (2 files), and the two stray `.webp` files. `.DS_Store` and the fixture's `target/` are correctly gitignored; `git status -uall` shows exactly the 13 intended new scenario files plus the new driver test module and checker.

Round 2's B4 is genuinely closed and the replacement wording is accurate in every clause I could check against the manifest, the enforcement code, and the pinned build script. The one remaining doc inaccuracy is conservative, unvalidated, and pre-dates this milestone.

SATISFIED
