I've mapped the whole change and confirmed the moved files are byte-identical to originals except for `use` renames and one `expect` message. Report follows.

# Code Review — M0b: extract `sifr_ipc` from `sifr_stdlib_manifest`

## Blocking findings

**None.** The boundary split is complete and self-consistent: every consumer of the moved IPC types, every verification-profile entry, every classification/sanitizer/fingerprint list, the dependency-direction guard, and both internal architecture docs have been updated in lockstep. No stale `sifr_stdlib_manifest::Ipc…` or `sifr-stdlib-ipc-pipe-fixture-worker` references remain outside archived plans/reviews (which are historical records and should stay untouched).

## Non-blocking observations

### 1. Stale summary paragraph in typed-IPC design report — same doc got other rows fixed
- `verification/areas/stdlib_parity/reports/concurrency_runtime_typed_ipc_design.md:5` still reads *"internal **`sifr_stdlib`** helpers encode/decode/read/write length-prefixed Postcard envelopes…"*.
- Lines 30–35 of the same document were correctly rewritten in this PR from `sifr_stdlib::ipc_*` → `sifr_ipc::ipc_*`, so the summary paragraph is now inconsistent with the very table it introduces. The `sifr_stdlib` naming here predates M0a (it was already wrong on `HEAD`) — but M0b is the natural moment to reconcile it since it's the same words being changed elsewhere in the same file.
- **Suggestion:** change "internal `sifr_stdlib` helpers" → "internal `sifr_ipc` helpers".

### 2. `sifr_ipc` lib.rs module doc understates the audience
- `crates/sifr_ipc/src/lib.rs:1-5` says the crate is *"reused by compiler tests and runtime-facing verification fixtures"*.
- In fact `crates/sifr_lowering/src/lower/ipc_schema_extraction.rs:2` uses `sifr_ipc::{IpcSchemaField, IpcSchemaType, IpcSchemaVariant}` at compiler runtime (not just under `#[cfg(test)]`). The classification file also marks `sifr_ipc` lib as `first_party_compiler`. The docstring should say something closer to "used by the compiler (lowering-owned schema extraction) and by verification fixtures" so future readers don't infer this is a test-only crate.

### 3. Review artifact placeholder is empty
- `plans/reviews/active/stdlib-native-boundary-m0b-ipc-crate-review-pass1.md` was added as a 0-byte file. If the review artifact is meant to land with the M0b PR (as M0a's `…-m0a-manifest-crate-review-pass1.md` did), it needs to be populated before merge; otherwise the empty placeholder should be dropped from the diff.

### 4. Phase tracker doesn't record M0b's merge yet
- `plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md:232` still reads *"M0b: create `sifr_ipc` and move shared IPC protocol code."* with no PR/commit reference — reasonable while the PR is unmerged, but note that the M0a bullet (line 229) was updated with its PR/commit link in this same PR, so consistency at merge time will require adding the M0b PR link before landing.

## Verified clean

- **Dependency direction and isolation.** `scripts/check_source_crate_dependency_direction.py` gives `sifr_ipc` `allowed_normal_dependencies={"postcard","serde"}` and `forbidden_source_references=ALL_SIFR_CRATES - {"sifr_ipc"}` (script lines 131–134). New self-test cases exercise both violations (lines 369–384). `sifr_ipc` is also added to `IR_FORBIDDEN_DEPENDENCIES`, `STDLIB_FORBIDDEN_DEPENDENCIES`, and `GENERATED_STDLIB_FORBIDDEN_DEPENDENCIES`, so downstream generated/stdlib/ir crates cannot silently regress by importing it.
- **Content-parity.** `crates/sifr_ipc/src/{ipc_connection,ipc_frame,ipc_payload,ipc_request_tracker,ipc_schema,ipc_transport}.rs` are byte-identical to their pre-move `sifr_stdlib_manifest/src/*.rs` counterparts (verified via `diff -q` against `HEAD:`). No behavior can regress from the code motion itself. Public re-exports in `crates/sifr_ipc/src/lib.rs:14-29` match the removed `pub use ipc_*` block from `sifr_stdlib_manifest/src/lib.rs` exactly.
- **Test/fixture moves.** `tests/ipc_process_pipe_fixture.rs` and `tests/fixtures/ipc_pipe_fixture_worker.rs` differ from their originals only in (a) `use sifr_stdlib_manifest::` → `use sifr_ipc::`, (b) two `sifr_stdlib_manifest::IpcConnectionPhase::Closed` → `sifr_ipc::IpcConnectionPhase::Closed` at fixture lines 233/264/295, (c) the `.expect("stdlib model crate lives under crates/sifr_stdlib_manifest")` → `.expect("ipc crate lives under crates/sifr_ipc")` at fixture line 42, and (d) the `--bin sifr-stdlib-ipc-pipe-fixture-worker` → `sifr-ipc-pipe-fixture-worker` cargo invocation. `crates/sifr_lowering/src/lower/ipc_schema_extraction.rs` mirrors the same rename in its Unix bootstrap test (lines 190–207).
- **Manifest cleanup.** `crates/sifr_stdlib_manifest/Cargo.toml` correctly drops `postcard`, `serde`, the `__test_fixture` feature, and the fixture bin section. `postcard`/`serde` still appear in `features.rs` and `features_tests.rs` as string literals used to render generated-project dependency specs — those are data, not linkage, so removing the crate-level `[dependencies]` entries is correct. The `sifr_stdlib_manifest` root doc-comment (`src/lib.rs:1-9`) is trimmed accordingly.
- **Workspace wiring.** `Cargo.toml` adds `sifr_ipc` to `members` (line 9) and `workspace.dependencies` (line 58). `sifr_lowering/Cargo.toml` picks it up via `sifr_ipc = { workspace = true }`. `Cargo.lock` shows the correct new `sifr_ipc v0.0.0 → postcard + serde` node and correctly removes `postcard`, `serde` from `sifr_stdlib_manifest`'s dep list.
- **File-size and boundary invariants.** Largest new file is `ipc_connection.rs` at 705 lines (well under the 900-line cap). `[lints] workspace = true` is set in `crates/sifr_ipc/Cargo.toml:23-24`, so pedantic clippy applies uniformly. No `sifr_stdlib_manifest` string appears anywhere under `crates/sifr_ipc/`.
- **Verification/coverage metadata.** `create-pr.json`, `merge.json`, `nightly.json`, `release.json` each add exactly one blocking `sifr_ipc` entry immediately before `sifr_stdlib_manifest`. `cargo_metadata_classification.json` moves the `ipc_process_pipe_fixture` test target and `sifr-ipc-pipe-fixture-worker` bin from the `sifr_stdlib_manifest` block into the new `sifr_ipc` block, and clears `sifr_stdlib_manifest`'s `features` array. `sanitizer_manifest.json` retargets both structured-skip entries from `-p sifr_stdlib_manifest` to `-p sifr_ipc`. `generated_code_quality.py` folds `sifr_ipc` into the `PRODUCER_FINGERPRINT_CRATES` list, keeping the fingerprint inputs sorted. `supported_host_matrix.md` retargets six IPC rows to `-p sifr_ipc`. Actual `cargo metadata` output matches the classification file (lib + bin + test target, deps limited to `postcard`, `serde`).

## Summary

M0b is mechanically clean and semantically consistent with the M0 boundary intent as stated in `plans/issues/active/ad-hoc-stdlib-native-boundary-completion.md`. All four non-blocking items are low-priority polish that can be applied before merge without touching code paths: three are one-line doc/comment tweaks, and the fourth is either populating or removing the empty review-artifact placeholder.
