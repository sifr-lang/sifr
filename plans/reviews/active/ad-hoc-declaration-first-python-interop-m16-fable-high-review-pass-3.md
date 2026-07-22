# M16 Frozen Whole-Diff Review — Raw API Ergonomics On Shared Ownership (Pass 3)

**Reviewer:** Independent milestone-closure review (Fable High, fresh pass)
**Range:** `3f974f33b` → `e66659baa` (4 commits: `feat(python): add typed raw API ergonomics`, `fix(python): address M16 review findings`, `docs(python): record satisfied M16 review`, `fix(lsp): skip unused Python environment resolution`), PR #2996
**Scope reviewed:** the full 54-file diff (+1330/−93); complete read of the new LSP environment-resolution gate (`crates/sifr_lsp/src/python_declarations.rs`) and its package-side dependencies (`sifr_package` selection, environment resolution, canonical requirements, bridge resolution/inventory, trust policy, manifest model); the M16 feature surfaces re-inspected at head (`direct_validation.rs`, `python_raw_api_codegen.rs`, fail fixtures, positive fixture, guardrail inventories, `internal_docs/architecture.md`, plan invariants and the M15/M16 sections of `plans/issues/active/ad-hoc-declaration-first-python-interop.md`); both earlier M16 review artifacts read and their findings re-verified against source and behavior, not taken on trust.

## Verdict summary

The M16 feature body remains sound — everything pass-2 verified at `e3377da62` still holds at head, and I re-executed the representative evidence. The new head commit's LSP performance fix, however, is **not the "exact-input gate" the plan doc claims**: I constructed two concrete packages where `sifr check` at head emits a hard error while the LSP at head returns zero diagnostics, and demonstrated that only the new gate suppresses them (opening the gate with an unrelated trust entry restores the identical diagnostic through the unchanged resolution path). These are confirmed false negatives against the closure criteria "workspace/member behavior must remain conservative" and "no Python-capable package may be skipped."

## Prior-findings re-verification (pass-1 → pass-2 closure holds at head)

- **Pass-1 MINOR-1 (polluted async fixtures):** still closed. Both fixtures use canonical `await task.sleep(0.0)` with no bogus import; I compiled all three fail fixtures at head — each emits exactly one diagnostic (`SIFR-PYCONV-0001` at 6:12; `SIFR-ASYNC-0003` at 7:12 and 7:18) at the annotated spans.
- **Pass-1 MINOR-2 (stale architecture status):** still closed. `internal_docs/architecture.md:54` carries the M16 contract sentences (typed `from_value`/`to_value`/`kwarg` through the declaration conversion authority, compiler-known checked `Object` methods, automatic release, owned-loop raw coroutines).
- **Feature invariants spot-re-verified at head:** raw conversion validation goes through the exact declaration predicate `is_direct_type` (`crates/sifr_lowering/src/lower/python_interop/direct_validation.rs:28`); codegen generates exclusively through `input_conversion`/`output_value_expr` and the existing declaration runtime helpers (`crates/sifr_codegen/src/python_raw_api_codegen.rs:3-99`) — no parallel converter or second ownership model. The two commits after pass-2 touch only `sifr_lsp`, one test file, and docs (verified via per-commit stats), so pass-2's package/runtime/demo/e2e evidence remains valid for those crates.
- **Independent execution at head:** all 26 `sifr_lsp` python-declaration tests pass (including the new `locked_package_without_python_inputs_skips_environment_resolution`, the configured-environment-without-declarations test, and the fingerprint unit tests); `raw_typed_ergonomics.sifr` compiles clean; `cargo fmt --check -p sifr_lsp` and `cargo clippy -p sifr_lsp -- -D warnings` clean; HIR maintainability guardrails PASS; all touched hand-maintained files under 900 lines (largest touched: 871).

## Gate analysis (what holds up)

Before the findings, the parts of the gate I challenged and found **correct**:

- **Configured environments with zero declarations still validate.** `manifest.python != PythonConfig::default()` is strictly broader than `selects_environment()` (it also catches `requires-imports`-only configs), and the existing test passes.
- **Invalid manifests keep canonical diagnostics.** `package_has_static_python_inputs` returns `true` on `SifrManifest::load` failure (`python_declarations.rs:651`), so the resolution path runs and `PackageSession::discover` renders the canonical package diagnostic. The synthetic `CargoPackageId("lsp-python-input-probe")` is used only for diagnostic provenance; `load` performs no id/name cross-check.
- **Fingerprint interplay is sound for the selection skip.** `package_input_fingerprint` now hashes pyproject/uv.lock/interpreter only when static inputs exist (`python_declarations.rs:597-604`). I verified from `resolve_python_environment_for_check` (`crates/sifr_package/src/python/environment.rs:168-211`) that when static inputs are absent, resolution can only end in `NotRequired`, `DeferredToFinalApplication`, or a trust error — a `Resolved` outcome (the only one that reads pyproject/lock/interpreter) requires root trust or explicit `[python]` selection, both of which make static inputs true. So no environment output can depend on unhashed selection paths.
- **Gate inputs are cache-coherent.** Every input the gate reads (root `sifr.toml`, bindings/certifications files, canonical bridge dir, the compiler plan) is covered by the fingerprint or the graph/source revisions in the cache entry key; watcher events additionally clear all caches (`session.rs:147-149`).
- **The benchmark win is legitimate in its target scenario.** For a genuinely pure, single-package locked project, pre-fix resolution loaded the full Cargo package graph only to conclude `NotRequired` with zero diagnostics; skipping it is behavior-preserving there, and the 103.5 → 69.7 MiB RSS drop comes from not launching cargo metadata/graph loading. The weakening is confined to the two multi-input shapes below.

## Findings

### MAJOR-1 — The gate skips graph-contributed Python requirements: a pure-root workspace with a Python-requiring member loses its trust diagnostics in the editor

**Files:** `crates/sifr_lsp/src/python_declarations.rs:626-653` (`package_has_python_inputs` / `package_has_static_python_inputs`), versus `crates/sifr_package/src/python/environment.rs:122` and `crates/sifr_package/src/python/requirements.rs:38-58`.

**Evidence (all reproduced at head `e66659baa`):** The compile path derives `requires_python` from `canonical_python_requirements(graph, …)`, which iterates **every package in the Cargo graph** — a workspace member's or path dependency's `[python] requires-imports` (and dependency bridge inventories via `resolve_python_bridge_graph`) make Python required regardless of the root manifest. The new LSP gate consults only the current plan and the **current package root's** manifest/artifacts; it never sees the graph — not loading the graph is the entire point of the fix. I built a scratch workspace (pure root `sifr.toml` with no `[python]`, no `[trust]`, no artifacts; member `sifr.toml` with `[python] requires-imports = ["numpy"]`, frozen `Cargo.lock`):

- `sifr check` at head: `error[SIFR-PYTRUST-0005]: required Python import root 'numpy' is not authorized by the root application`.
- The real `sifr lsp --stdio` at head, same package, `textDocument/diagnostic`: `{"kind": "full", "items": []}`.

The resolution path itself still surfaces this diagnostic whenever the gate passes — the existing test `workspace_member_python_requirements_are_validated` (`python_declaration_tests.rs:558`) passes at head only because its fixture root incidentally carries `[python]` + `[trust]` config; strip the root config and the scenario regresses. Pre-fix the LSP resolved unconditionally, so this is a behavioral regression introduced by `e66659baa`, and no watcher event or cache invalidation can recover it (the gate never consults member manifests). It also falsifies the standing architecture claim that "LSP environment status goes through the same Cargo package graph, root trust, … decisions as package checking" (`internal_docs/architecture.md:54`) and the plan's "cheap exact-input gate" wording (`plans/issues/active/ad-hoc-declaration-first-python-interop.md`, M15 status paragraph).

**Rationale:** Directly violates two stated closure criteria — "workspace/member behavior must remain conservative" and "no Python-capable package may be skipped." The suppressed diagnostic is precisely the one that tells the root author what to fix (add the trust entry); the build still fails, but the editor is now silent on a build-failing configuration, breaking LSP/compiler diagnostics parity that M15 established and the architecture doc guarantees.

**Remediation:** Make the gate conservative for non-trivial package graphs. Cheapest exact option: after the static checks fail, decide purity from inputs already read — e.g., parse the root `Cargo.lock` (already read for the fingerprint) and treat any lockfile listing more than the root package as "has potential Python inputs," falling back to full resolution; or enumerate workspace members/path dependencies from the root `Cargo.toml` and apply the same static-input probe to each member's `sifr.toml` (in that case the fingerprint must also hash those member manifests so gate flips invalidate the cache). Add a regression test mirroring `workspace_member_python_requirements_are_validated` but with a **pure** root manifest, asserting `SIFR-PYTRUST-0005` still appears. Verify the representative benchmark package remains on the fast path, and correct the "exact-input gate" plan wording to match the shipped semantics.

### MAJOR-2 — The gate checks only the canonical bridge directory, silencing the misplaced-bridge-root diagnostic

**Files:** `crates/sifr_lsp/src/python_declarations.rs:634-639` versus `crates/sifr_package/src/python/bridge_inventory/filesystem.rs:9-34` (`misplaced_root_diagnostics`).

**Evidence (reproduced at head):** `package_has_static_python_inputs` tests only `root.join(PYTHON_BRIDGE_ROOT).is_dir()` — the fixed canonical `src/python_bridges`. Bridge inventory discovery, however, also diagnoses bridge sources found at `package_root/python_bridges` or `<configured-source-root>/python_bridges` with `SIFR-PYIMP-0002` ("bridge source root must be 'src/python_bridges', not …"). Scratch package: pure manifest, `python_bridges/util.py` at package root, frozen lockfile:

- `sifr check` at head: `error[SIFR-PYIMP-0002]: invalid package-local Python bridge: bridge source root must be '…/src/python_bridges', not '…/python_bridges'`.
- `sifr lsp --stdio` at head: `{"kind": "full", "items": []}`.
- Same package with an unrelated `[trust] python = ["numpy"]` added (opening the gate): the LSP immediately renders the full `SIFR-PYIMP-0002` diagnostic with its help text — proving the resolution machinery is unchanged and only the gate suppresses it.

There is also a cache-coherence wrinkle: the fingerprint's `hash_python_bridge_inputs` hashes only the canonical root, so a misplaced bridge dir appearing/disappearing changes neither the gate result nor the fingerprint (watcher-driven `invalidate_external` masks this in editors that send file events, but the fingerprint should cover what discovery reads).

**Rationale:** Same invariant violation as MAJOR-1 in a second shape: a package that plainly intends Python bridging is skipped, and the guidance diagnostic that names the exact fix is hidden in the editor while `sifr check` fails. The gate under-approximates the very input class ("bridge directories") the remediation summary claims to cover.

**Remediation:** Extend the static check to the same candidate set discovery uses: `package_root/python_bridges` plus `<source_root>/python_bridges` for each configured source root (the manifest is already loaded in this function, so `source_roots` is available). Include those candidate paths in `package_input_fingerprint`. Add a regression test: pure manifest + misplaced `python_bridges/` → `SIFR-PYIMP-0002` appears in document diagnostics.

## Non-findings (observations, no action required)

- The `to_value`/`from_value`/`kwarg` validation, sealed `Object` identity, checked `PythonError` contract, all-borrow/automatic-release ownership, owned-loop coroutines, blocking-effect rejections, guardrail inventories, docs, and demo were verified end-to-end in passes 1–2 and remain code-identical at head; I re-executed the fixture, LSP, formatting, Clippy, and maintainability evidence at head rather than re-running the heavy native package/runtime suites, whose recorded pass-2 results still bind.
- The `lockfile_less_*` conservative behaviors and `stale_existing_lockfile_remains_a_package_error` are unaffected by the gate (missing-lockfile handling sits inside the resolution path and the stale-lockfile fixture carries the configured manifest).
- `dd7c1f45b` correctly marks M15 merged with its PR link and records the pass-2 artifact; appropriate bookkeeping.
- The plan states the full authoritative merge profile must be rerun after this review/remediation cycle — correct, and remediation of the findings above will require it regardless.

## Verdict

The M16 feature milestone remains solid, but the head remediation commit ships a Python-environment input gate with two independently confirmed editor false negatives — workspace/member-contributed requirements and misplaced bridge roots — each demonstrated with a live `sifr check` error against a silent `sifr lsp` at the same head, in direct conflict with the closure criteria that no Python-capable package be skipped and that workspace/member behavior remain conservative.

VERDICT: NOT SATISFIED
