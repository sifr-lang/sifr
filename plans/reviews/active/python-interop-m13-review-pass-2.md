The review is complete. I reproduced the remediated pass-1 scenario (fixed) and then broke the remediation with a variant it doesn't cover. Findings below; all evidence is from the rebuilt release binary at HEAD (`d6696342e`) plus the full `main...HEAD` diff (42 files). Per instructions I modified nothing (the empty `plans/reviews/active/python-interop-m13-review-pass-2.md` was left untouched).

## Findings

### MAJOR-1 — The parity gap survives in a new form: a library root whose environment the compiler resolves by *uv-project discovery* still gets `python check: ok` while `sifr check` fails; doctor's suggestion is wrong for it

The remediation gates read-only runtime resolution on `application || package.manifest.python.selects_environment()` (`crates/sifr/src/python_cli.rs:227`, backed by `crates/sifr_package/src/manifest/sifr.rs:60`). But that predicate is a re-derivation, not the actual authority. The resolver (`resolve_python_environment_with_requirements`, `crates/sifr_package/src/python/environment.rs:139-153`) requires Python whenever required imports **or root trust** exist, and `root_environment_selection` grants selection to any session root via ancestor discovery of `pyproject.toml` + `uv.lock` (`configured_or_discovered_project_root` / `discover_project_root`, `crates/sifr_package/src/python/selection.rs:89-116`) — no explicit `[python]` section needed.

Reproduced: standalone library with `[trust] python = ["math"]`, **no** `[python]` section, and pyproject.toml + uv.lock + .venv at the package root — the exact layout the compiler's own help text tells users to create ("create pyproject.toml, uv.lock, and .venv in the project ancestry"):

- Valid target: `sifr check src/__init__.sifr --frozen` → "no errors found", probe executed against the discovered interpreter. `sifr python check` → "ok" but `environment: deferred`, `trust: deferred-to-final-application`, `math.sqrt: deferred` — contradicting the compiler's resolved/verified state on the identical snapshot.
- Invalid target `math.not_a_real_target`: `sifr check` → exit 1 `SIFR-PYIMP-0001`; `sifr python check` → exit 0 "ok". False negative.
- `sifr python doctor` → exit 0 and suggests adding `[python]`/`[trust] python = ["math"]` to `final-application/sifr.toml` — trust this root already declares, for an environment the compiler already resolves.

This is the same contract violation as pass-1 MAJOR-1 ("nothing to defer when the environment is resolvable locally"), reachable through the most conventional uv layout instead of explicit selection. It also falsifies the new doc claims — `internal_docs/python_interop_architecture.md:67` ("the shared authority that keeps normal check and Python inspection diagnostics aligned") and `docs/python-interop.mdx:78` ("`deferred` when a library has no authority to select an environment") — and it is a duplicate-authority instance: the fix copied a fragment of the resolver's gate onto the CLI instead of consuming the resolver's decision. No test covers a discovery-resolved library. The remediation direction should be to ask the single authority (attempt resolution for the session root and defer only on the resolver outcomes that genuinely mean "final-application authority missing"), not to widen the CLI-side predicate again.

### MEDIUM-2 — In the sanctioned deferral scenarios, ordinary check *errors* rather than defers, so the acceptance "both commands agree with compiler results" is never met for a standalone library with declarations

Reproduced on two snapshots not covered by any test:
- Bare library (same shape as the shipped deferred fixture, `verification/areas/python_interop/runner/readonly_check_doctor.py:24`): `python check` → exit 0 "ok"; `sifr check src/__init__.sifr --frozen` → exit 1 `SIFR-PYTRUST-0005`.
- Trust-only library, no discoverable project: `python check` → exit 0 "ok"/deferred; `sifr check` → exit 1 `SIFR-PYENV-0003`.

The milestone mandates the deferred-library policy and pass-1 accepted it, so I rate this medium rather than major — but the compiler path has no deferral concept for standalone library roots at all (`requires_python` is true from the library's own declarations, then trust/selection hard-fail), and the internal doc's claim that a selection-less library root "still runs its full compiler/protocol plan" with deferred probes (`internal_docs/python_interop_architecture.md:60-64`) omits that ordinary `sifr check` rejects the same snapshot outright. This needs either an explicit, documented and tested policy decision (deferral is a deliberate divergence of the read-only surface), or resolver-level deferral so both surfaces genuinely agree.

### MINOR-3 (carried from pass-1 MINOR-2) — CLI re-derives environment/trust status while the driver report's answer goes unused

`run_python_read_only_plan` computes `environment`/`trust` from `context.runtime` (`crates/sifr/src/python_cli.rs:309-330`) while `PythonInteropCheckReport.environment`/`required_import_roots` (`crates/sifr_driver/src/build/report.rs:114-146`, populated in `python_check.rs:67`) have no consumer. This duplication is exactly the mechanism that produced MAJOR-1; consolidating on one authority would fix both.

### MINOR-4 (carried from pass-1 MINOR-3) — Library inspection remains O(modules × import-closure)

`python_inspection_entrypoints` treats every module as an entrypoint and each iteration clones the full graph and source map (`crates/sifr/src/python_cli.rs:315-345`). Correctness unaffected.

### Observation — the shipped CLI test only passes in debug profile

`cargo test --release -p sifr --test python_read_only_cli` fails 3/4 with `SIFR-STDLIB-0003` because release binaries disable source-tree sysroot discovery (`crates/sifr_sysroot/src/resolve.rs:59`, `debug_assertions`); the debug run (what the harness executes) passes 4/4. Pre-existing pattern, not a defect of this PR — noting so "4/4" evidence is understood as debug-profile evidence.

## Contract verification (what held up)

- **Pass-1 MAJOR-1 as literally stated is fixed**: explicit `[python]`-selecting library now resolves, probes, and fails/succeeds identically to `sifr check` (suite `parity=3`, Rust test `library_with_root_environment_selection_resolves_and_matches_normal_check`, both re-run and passing here; suite output `deferred=1 resolved=2 parity=3 mutations=0` in 76s).
- **Shared plan**: `check_package_python_interop` is one plan for both surfaces; build keeps the strict policy (`SIFR-PYENV-0003` preserved, unit-tested at `python_interop.rs:499`); bridge targets are `RuntimeChecked` under both policies (unit-tested); skipping `apply_package_runtime_metadata` on the check path drops only an internal-panic bootstrap-injection path, no user diagnostic — no binary startup is injected into libraries.
- **Read-only**: `uv lock --check --offline` only (`python_probe.rs:25`), no materialization on check, frozen `CargoLockMode` hardcoded, certifications loaded read-only; byte-level non-mutation including symlink targets asserted on success and failure in both the Rust test and the suite.
- **Determinism/snapshot identity**: BTree collections throughout; doctor byte-determinism asserted twice; `digest_package_source_snapshot` digests file bytes with package/module identity, I/O errors handled, suite asserts digest sensitivity.
- **Multiple apps**: `runnable_app_paths` covers all targets; suite verifies both `math.ceil` and `math.sqrt`.
- **Gates**: `cargo clippy --workspace -- -D warnings` clean (the `--all-targets` failures I hit are pre-existing test files untouched by this branch), `cargo fmt --check` clean, HIR guardrails PASS, transfer guardrails PASS with accurate updated line refs (`python_cli.rs:578/635`, `selection.rs:115` verified), all touched first-party files ≤ 898 lines, Ruff submodule pointer unchanged in the diff, profiles wired in all four delivery profiles as blocking.

## Speculative improvements (non-blocking)

- Expose a first-class resolver outcome (e.g. `PythonAuthorityDecision::{Resolved, DeferredToFinalApplication, Error}`) from `sifr_package` so the CLI, driver, and doctor all consume one decision; doctor suggestions could then be derived from the concrete missing pieces (trust vs selection) instead of one static patch.
- Have doctor omit the `[trust]` hunk when the root already declares the required roots, and target the current package's `sifr.toml` when the root itself is the resolvable authority.
- Emit a `skipped` marker exit path (or `#[ignore]`-with-reason) for the uv/venv-gated Rust tests so profile dashboards can distinguish skip from pass.

MAJOR-1 is a demonstrated false-negative `ok` plus wrong doctor guidance on a snapshot the compiler diagnoses — the same contract violation class pass-1 blocked on, surviving the remediation via the discovery path. The milestone cannot be certified in this state.

VERDICT: NOT SATISFIED
