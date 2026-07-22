# M13 Closure Review — Pass 7 (PR #2993, `codex/m13-python-readonly-check-doctor`, HEAD `a13a6608c`)

**Bottom line: the frozen M13 implementation satisfies every acceptance item of the
read-only check/doctor contract, the closure ledger's authoritative merge-gate
evidence is independently corroborated (including an arithmetic recomputation of
the E2E signature), and every focused validation re-run at HEAD passes. There are
no blockers and no majors.**

This is the final closure review of the complete `main...HEAD` diff (67 files,
+2470/−353, 11 commits), performed against the M13 contract at
`plans/issues/active/ad-hoc-declaration-first-python-interop.md:1835-1930`, the six
committed review-pass ledgers, `internal_docs/python_interop_architecture.md`,
`docs/python-interop.mdx`, and the diagnostics evidence surfaces. Nothing outside
this review file was modified.

## Scope identity

The two commits after the pass-6-reviewed tree (`81763d521`) — `dee1d6089` and
`a13a6608c` — touch only `plans/`, `internal_docs/architecture.md` (one closure
sentence), and `plans/roadmap.md`. Every production, test, verification,
generated-docs, and demo byte at HEAD is identical to what pass 6 certified, so
this pass re-verifies the frozen implementation and adds independent evidence
checks on the closure claims.

## Closure-ledger evidence — independently corroborated

The ledger records: authoritative merge gate `4056.35s`, Python interop `23/23`
with `deferred=1 resolved=3 parity=5 mutations=0`, E2E `674/674` signature
`1f8b1cadc4f48ec8`, diagnostics `175/175`, `261` hardening variants, zero
failures.

- **Python interop 23/23**: `verification/profiles/merge.json` selects exactly 23
  `python_interop` suites, including the new blocking `readonly-check-doctor`
  (`merge.json:109`); the pre-M13 merge gate recorded `22/22`, and this branch adds
  exactly one suite. The `17/17` create-pr records likewise match that profile's 17
  selected suites.
- **E2E 674/674 signature `1f8b1cadc4f48ec8`**: exactly 674 fixtures exist in
  `crates/sifr/tests/e2e/pass/`, the diff touches none of them, and I recomputed
  the signature from first principles — for a fully passing report,
  `report_signature` (`crates/sifr/tests/e2e_support/batch_execution.rs:718`)
  reduces to the FNV-1a hash of `"pass|674|674|"`, which is exactly
  `1f8b1cadc4f48ec8`. The recorded signature is therefore arithmetically exact for
  a 674/674 all-pass run of the on-disk fixture set, and matches the M10–M12
  closure gates.
- **Diagnostics 175/175**: the `baselines` suite's 147 cases expand to exactly 175
  diagnostic-format variants under the adapter's expansion rule
  (`verification/runner/sifr_verify/area_adapter.py:308`); the 5 `rules` checks are
  counted separately and all pass at HEAD (re-run this pass: `baseline_hygiene`,
  `code_baseline_coverage`, `code_coverage`, `docs_sync`, `schema_sync` — all exit 0).
- **261 hardening variants**: identical to the recorded pre-M13 authoritative gate
  (merge selects `diagnostics, project, fixedbugs, crashes, oss-curated`), and the
  diff adds no hardening fixtures — internally consistent.
- **`m third_party/ruff`**: `git diff main...HEAD -- third_party/ruff` is empty —
  the submodule pointer is unchanged in the PR. The working-tree dirt is a
  semantically neutral one-line join in
  `crates/ruff_python_parser/src/parser/expression.rs`, not part of the diff.

## Acceptance contract — independently re-verified at HEAD

- **Shared typed environment decision.** `resolve_python_environment_for_check`
  (`crates/sifr_package/src/python/environment.rs:117`) is the single
  `NotRequired`/`Resolved`/`DeferredToFinalApplication` authority. Deferral requires
  the caller's opt-in *and* that every trust error is
  `PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED` (environment.rs:184-190) and every
  selection error is `PYENV_MISSING_SELECTION` (environment.rs:192-201). The strict
  wrapper's `unreachable!` (environment.rs:111) is provably dead: with the flag
  false, both error paths return `Err` before a deferred outcome can be built.
  Unit tests cover both flag values, including the discovered-authority case
  (`crates/sifr_package/src/python/tests.rs`,
  `read_only_library_resolution_*`).
- **Parity between ordinary check and `sifr python check`.** Ordinary check gates
  the opt-in on `session.runnable_app_paths().is_empty()`
  (`crates/sifr/src/check_and_package_commands.rs:117`); `python check`/`doctor`
  use the identical `!application` predicate from the same source
  (`crates/sifr/src/python_cli.rs:195-236`,
  `crates/sifr/src/python_runtime_context.rs:34`). Both then execute the same
  driver plan `check_package_python_interop`
  (`crates/sifr_driver/src/build/api.rs:50-60`). Reproduced this pass on a fresh
  bare-library fixture: `python check` → exit 0, environment `deferred`, trust
  `deferred-to-final-application`; `sifr check src/__init__.sifr --frozen` → "no
  errors found". On a fresh untrusted application, all four surfaces — `python
  check`, ordinary `check`, `python doctor`, and `run` — fail with the identical
  `SIFR-PYTRUST-0005`.
- **Strict final-application build/run.** `compile_package_entrypoint_report` and
  `cmd_run_package_file` hardcode `allow_python_deferral=false`
  (`check_and_package_commands.rs:534`, `diagnostic_rendering_and_run.rs:439`), and
  `into_generated_binary_project` passes `false`
  (`crates/sifr_driver/src/build/entrypoint.rs:586-588`). Reproduced: `sifr build`
  on the deferred bare library errors `SIFR-PYTRUST-0005` while both check surfaces
  accept it.
- **Deterministic non-mutating doctor.** `doctor_suggestions`
  (`python_cli.rs:424-459`) derives hunks solely from the typed
  `DeferredPythonEnvironment` — `[python]` only when selection is missing, `[trust]`
  only for actual missing imports — over BTree collections; there is no write path
  in either command and lock mode is hardcoded `CargoLockMode::Frozen`. Reproduced
  on the demo: two `python doctor` runs byte-identical, and a SHA-256 content
  digest of the full demo tree is identical before/after `check` + `doctor`.
- **Standalone explicit and discovered environments; deferred libraries;
  multi-application handling.** Covered by `crates/sifr/tests/python_read_only_cli.rs`
  (explicit-selection, uv-discovered, bare-library deferral, trust-only one-sided
  patch, multi-app `math.ceil`+`math.sqrt`, failure parity via `SIFR-PYIMP-0001`)
  and by the blocking suite
  (`verification/areas/python_interop/runner/readonly_check_doctor.py`), which
  passed here through the official runner with the exact recorded evidence line:
  `python interop read-only check/doctor ok: deferred=1 resolved=3 parity=5
  mutations=0` (`variants=1, failures=0, blocking_failures=0`, 109.2s).
- **Source/graph snapshot identity.** `digest_package_source_snapshot`
  (`crates/sifr_package/src/graph/digest_source_map.rs:15`) digests module bytes
  with package/module identity over `BTreeMap` ordering; I/O failures surface as a
  handled error, not a panic; the suite asserts digest sensitivity to source bytes.
  Demo report shows stable `graph=`/`source=` digests across runs.
- **Diagnostic catalog consistency.** Verified programmatically: for all eight
  codes `SIFR-PYENV-0004`–`0011`, the catalog
  (`verification/areas/diagnostics/data/code_catalog.json`), the registry
  (`crates/sifr_diagnostics/src/codes/registry/registry_entries/python_interop.rs`),
  all eight `docs/errors/SIFR-PYENV-00XX.md` pages, and
  `internal_docs/diagnostic_codes.md` agree on
  `crates/sifr_package/src/python/probe_validation_tests.rs`, every referenced test
  function exists in that module, and no stale `python/tests.rs::probe_rejects`
  reference remains anywhere.
- **Parallel-safe tests.** The pass-5/6 fix is in place (`AtomicU64` fixture
  discriminator plus collision-detecting `create_dir`,
  `python_read_only_cli.rs:9,77-82`). Three fresh default-parallel runs of the
  compiled suite this pass: 6/6, 6/6, 6/6 — on top of pass 6's thirteen
  consecutive 78/78.

## Validation re-run at HEAD (this pass)

- Blocking suite `readonly-check-doctor` via
  `uv run --project verification --locked python -m sifr_verify areas run` — pass,
  exact evidence line. Wired blocking in all four delivery profiles.
- `cargo test --locked -p sifr --test python_read_only_cli` — 6/6 × 3 under
  default parallelism.
- `cargo test --locked -p sifr_package` — 133/133.
- `cargo test --locked -p sifr_driver --lib` — 368 passed, including both new
  probe-policy tests (`read_only_check_defers_library_target_without_an_environment`,
  `read_only_check_marks_embedded_bridge_target_runtime_checked_when_deferred`).
- `cargo clippy --workspace -- -D warnings` — clean; `cargo fmt --check` — clean.
- Guardrails: HIR maintainability PASS, file-size PASS (2768 files, 900-line cap;
  largest touched file `entrypoint.rs` 890, `python_cli.rs` 836), sifr_driver
  maintainability PASS, package-manager 420-line PASS (`environment.rs` 418),
  TypeScript/Go transfer guardrails PASS with line references spot-verified at HEAD
  (`python_cli.rs:612/669`, `environment.rs:283`, `selection.rs:115`).
- Demo `demos/m13_python_read_only`: `python check --json` → `ok`, resolved,
  trust verified, `math.sqrt: verified`; `python doctor` → `suggestions: none`,
  byte-deterministic; ordinary `check --frozen` → no errors; `sifr run` →
  `Python read-only check demo: target verified`; repository `git status` clean
  afterwards (only the pre-existing submodule working-tree dirt).

## Non-blocking notes (carried forward as M14 candidates)

- `PythonInteropCheckReport.environment`/`required_import_roots`
  (`crates/sifr_driver/src/build/report.rs:114`) still have no CLI consumer; the
  CLI derives status from the shared resolver outcome, so drift risk is contained,
  but the dead fields deserve consolidation.
- The defensive `(false, true)` arm in `doctor_suggestions` has no constructible
  input.
- Library inspection clones graph and source map per module
  (`python_cli.rs:289-296`) — a performance observation only.
- Release binaries disable source-tree sysroot discovery (pre-existing
  `debug_assertions` pattern), so the demo/manual reproduction uses the debug
  binary or the verification harness's configured sysroot; not an M13 concern.

Every acceptance item of the M13 contract ("both commands agree with
compiler/build results for the same snapshot"; "neither command writes, installs,
trusts, or runs environment synchronization") reproduces independently at HEAD,
the closure ledger's merge-gate evidence is arithmetically and structurally
consistent with the tree, the unrelated Ruff submodule dirt is absent from the
diff, and all blocking validation lanes re-run clean. There are no blocking and
no major findings.

VERDICT: SATISFIED
