All verification complete. The demo compiles and runs end-to-end, and a broken explicit venv on a library blocks identically on both surfaces rather than deferring. Here is my closure review.

---

# M13 Closure Review — Read-Only Check And Doctor (PR #2993, `codex/m13-python-readonly-check-doctor`, HEAD `6c3a35ef2`)

I reviewed the full `main...HEAD` diff (49 files), the M13 contract in `plans/issues/active/ad-hoc-declaration-first-python-interop.md:1834`, all three commits, both committed review-pass ledgers, and reproduced every prior blocker scenario plus new variants against a freshly built release binary. Per instructions I edited nothing.

## Prior blocker 1 — standalone libraries with explicit or uv-discovered environments must resolve/probe: FIXED

The remediation deletes the CLI-side predicate and makes the package layer the single authority. `resolve_python_environment_for_check` (`crates/sifr_package/src/python/environment.rs:116`) returns a typed `NotRequired` / `Resolved` / `DeferredToFinalApplication` outcome; deferral is permitted only when *every* trust error is `PYTRUST_REQUIRED_IMPORT_UNAUTHORIZED` (environment.rs:184–190) and *every* selection error is `PYENV_MISSING_SELECTION` (environment.rs:192–201), and only when the caller allows it. Selection still goes through `root_environment_selection`, so both explicit `[python]` paths and uv ancestor discovery resolve for a standalone library root.

Reproduced (release binary, scenarios A/E): a library with `[trust] python = ["math"]`, no `[python]`, and pyproject.toml + uv.lock + .venv at the root — the exact pass-2 MAJOR-1 layout — now reports `environment: resolved`, `trust: verified`, `math.sqrt: verified`, and `sifr check src/__init__.sifr --frozen` agrees. With `math.not_a_real_target`, both `sifr python check` and `sifr check` exit 1 with the identical `SIFR-PYIMP-0001`, and `python doctor` also fails instead of emitting wrong suggestions. The explicit-`[python]` variant (pass-1 MAJOR-1) behaves identically. Both variants are covered permanently: `crates/sifr/tests/python_read_only_cli.rs:204,246`, the verification suite's `selected-library`/`discovered-library` blocks (`verification/areas/python_interop/runner/readonly_check_doctor.py:77–172`), and package unit test `read_only_library_resolution_uses_discovered_environment_authority` (`crates/sifr_package/src/python/tests.rs`).

## Prior blocker 2 — shared typed authority for bare/trust-only/discovery-only libraries; strict build/run/apps: FIXED

Both surfaces now consume one decision through `package_python_runtime_for_check` (`crates/sifr/src/python_runtime_context.rs:34`): ordinary check gates deferral on `session.runnable_app_paths().is_empty()` (`crates/sifr/src/check_and_package_commands.rs:117`), and `python check`/`doctor` use the same `!application` predicate from the same `runnable_app_paths` (`crates/sifr/src/python_cli.rs:195–236`). Both then run the identical driver plan `check_package_python_interop` (`crates/sifr_driver/src/build/api.rs:43–60`).

Reproduced (scenarios B/C/D/F/H):
- **Bare library**: `python check` → deferred/deferred-to-final-application; `sifr check` → "no errors found" (exit 0). Agreement, no false diagnostic on either side.
- **Trust-only library, no discoverable project**: both defer; the pass-2 `SIFR-PYENV-0003` mismatch is gone.
- **Application missing trust**: both surfaces exit 1 with identical `SIFR-PYTRUST-0005`; doctor also exits 1 (strict, no suggestions on apps).
- **Build/run stay strict**: `sifr build` on the bare library → `SIFR-PYTRUST-0005`; on the trust-only library → `SIFR-PYENV-0003`. `compile_package_entrypoint_report` and `cmd_run_package_file` hardcode `allow_python_deferral=false` (`check_and_package_commands.rs:534`, `diagnostic_rendering_and_run.rs:439`), and the strict resolver wrapper's `unreachable!` (environment.rs:111) is a genuine invariant — strict mode returns `Err` before either deferral branch.
- **Non-deferrable failures stay blocking for libraries**: broken explicit venv → identical `SIFR-PYENV-0004` failure on both surfaces, not a deferral.

## Doctor: genuinely missing hunks, deterministic, read-only — VERIFIED

`doctor_suggestions` (`python_cli.rs:424`) derives hunks solely from the typed `DeferredPythonEnvironment`: `[python]` only when `environment_selection_missing`, `[trust]` only for the actual `missing_trusted_imports`. Reproduced: bare library → both hunks; trust-only library → `[python]` hunk only; untrusted discovery/selection library → `[trust]` hunk only; resolved library and demo → `suggestions: none`. Byte-determinism and byte-level non-mutation (file contents + symlink targets, success and failure paths) verified by my own snapshot harness, the Rust test, and the suite. The only mutation paths in `python_cli.rs` belong to `certify`, an unchanged pre-existing command; check/doctor use `CargoLockMode::Frozen` and `uv lock --check --offline` only.

## Other contract points

- **Multi-app coverage**: `runnable_app_paths` (`crates/sifr_package/src/ops/workspace_session.rs:46`) enumerates all app targets; `src/main.sifr` + `src/bin/secondary.sifr` both probed and `verified` (test + suite output `deferred=1 resolved=3 parity=5 mutations=0`).
- **Snapshot identity**: `digest_package_source_snapshot` digests module bytes with package/module identity over BTree ordering (`crates/sifr_package/src/graph/digest_source_map.rs:15`); suite asserts digest sensitivity to source bytes; I/O errors surface as diagnostics, not panics.
- **No user-triggerable panics**: sweep of all changed production files found none; JSON serialization failure is handled (`python_cli.rs:467`).
- **File-size guardrail**: largest touched first-party files are 892/890/875 lines — all under 900; the remediation split `python_runtime_context.rs` out for this.
- **Guardrails/lints**: `check_hir_maintainability_guardrails.py` PASS, `check_typescript_go_transfer_guardrails.py` PASS with corrected line refs (spot-verified `environment.rs:283`, `selection.rs:115`), `cargo clippy -p sifr -p sifr_driver -p sifr_package -- -D warnings` clean, `cargo fmt --check` clean.
- **Tests**: `python_read_only_cli` 6/6 (uv and `verification/.venv` present — nothing skipped, and skips now `eprintln`), `sifr_package` python 57/57, `sifr_driver --lib` 368 pass including both new probe-policy tests, blocking suite wired in all four delivery profiles.
- **Demo**: `demos/m13_python_read_only` — `python check` resolved/verified, `doctor` suggestions none, `sifr run` prints `Python read-only check demo: target verified`.
- **Docs**: `docs/python-interop.mdx` and `internal_docs/python_interop_architecture.md:49–78` now state the single shared resolution outcome, standalone-root resolution via explicit selection *or* uv discovery, deferral only for selection-less/untrusted library sessions, ordinary-check probe execution, and strict build/run — all claims I verified empirically. Plan tracking records passes 1–2 and leaves Wave 4 open, correctly.
- **third_party/ruff**: absent from the committed diff (`git diff main...HEAD --name-only` has no ruff entry; submodule pointer identical). The `-dirty` marker is uncommitted local working-tree state in the submodule (`crates/ruff_python_parser/src/parser/expression.rs`), plus untracked `.agent-m13-pass3.log` — neither is part of the PR.

## Non-blocking observations

1. **Carried from passes 1–2**: `PythonInteropCheckReport.environment`/`required_import_roots` (`crates/sifr_driver/src/build/report.rs:114`) still have no CLI consumer — the CLI re-derives status from `context.runtime`. The drift risk that produced the original blockers is now largely gone because both derive from the single resolver outcome, but the dead fields remain worth consolidating in M14.
2. **Carried perf note**: library inspection still clones graph + source map per module (`python_cli.rs:289–296`).
3. **Pre-existing corner, unchanged by this branch**: an application with only bridge-target declarations and zero external imports resolves `NotRequired`, so both check surfaces pass while `build` errors `SIFR-PYENV-0003`; on main, check was frontend-only and passed too, so this is not an M13 regression.
4. `doctor_suggestions`' `(false, true)` arm can emit a suggestion with an empty patch; I could not construct a reachable input, and it is defensive only.
5. Wave 4's authoritative full-profile gate must still be run before merge, as the plan itself records; the pre-review create-pr gate (`1110.82s`, all blocking lanes) predates the final remediation commit, whose focused validations I re-ran and confirmed here.

Both prior blockers are demonstrably closed at the root (one typed package-layer authority consumed by every surface), the closure evidence is permanent and blocking in all four profiles, and every acceptance criterion of the M13 contract held up under independent reproduction.

VERDICT: SATISFIED
