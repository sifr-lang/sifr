# M13 Review — Read-Only Check And Doctor (PR #2993, `codex/m13-python-readonly-check-doctor`)

I inspected the full `main...HEAD` diff (37 files), read all affected production paths (`python_cli.rs`, driver `api/entrypoint/python_interop/python_check/report`, `sifr_package` digest/session), the new CLI test, verification suite, profile wiring, demo, public/internal docs, and the plan tracking. I also reproduced behavior empirically with the built CLI: the new verification suite passes locally, the demo verifies end-to-end, and I constructed three extra scenarios beyond the shipped tests. The Ruff submodule pointer is unchanged in the branch (the `m` in git status is local working-tree content only). All touched first-party files are under 900 lines.

## Findings

### MAJOR-1 — A library that selects its own `[python]` environment gets `python check: ok` while `sifr check` on the same snapshot fails; doctor's suggestion is wrong for that package
`crates/sifr/src/python_cli.rs:203` (in the diff; `python_read_only_context`) gates runtime resolution on `application` — a package with no runnable app targets never resolves an environment, even one it explicitly selected. But the compiler authority (`package_compiler_context` → `package_python_runtime`, `crates/sifr/src/check_and_package_commands.rs:187`) resolves environment authority by *session root*, not by app-target presence, and `root_environment_selection`/`non_root_environment_configuration` (`crates/sifr_package/src/python/selection.rs:15,54`) only forbid selection by non-root packages in a graph — a standalone library is its own root and its selection is honored.

Reproduced: library-only package with `[python] venv/pyproject/lock` + `[trust] python = ["math"]` (valid venv) and `@python(math.not_a_real_target)`:
- `sifr python check` → exit 0, `Python check: ok`, target `deferred`, trust `deferred-to-final-application`
- `sifr check src/__init__.sifr --frozen` → exit 1, `SIFR-PYIMP-0001` (the probe ran against the library's own selected interpreter)
- `sifr python doctor` → exit 0 and suggests adding `[python]`/`[trust]` to `final-application/sifr.toml` — sections this package already declares.

This violates the milestone contract "Commands agree with compiler/build diagnostics for the same source snapshot" with a false-negative `ok`, and it makes the new claim in `internal_docs/python_interop_architecture.md` ("Library-only packages have no authority to select an environment") factually wrong — the resolver demonstrably grants that authority to any session root and normal `check` exercises it. The library-deferral carve-out doesn't cover this case: there is nothing to defer when the environment is selected and resolvable locally. Either `python check` should resolve/probe when the library root selects an environment, or the resolver/`sifr check` should reject library-root selection — but the two surfaces must not disagree. No test covers a library with `[python]` selection.

### MINOR-1 — `sifr check` for package projects now runs full binary-project generation and live interpreter probes
`crates/sifr_driver/src/build/api.rs:44` reroutes `check_package_project` through `check_package_python_interop`, which calls `into_generated_binary_project_with_probe_policy(true)` after frontend diagnostics. Plain `sifr check` on any package now performs codegen and, when a runtime resolves, spawns the Python interpreter per target probe. This is the intended parity mechanism (and is what makes the parity tests pass), all validation budgets pass, and the probe is read-only — but it is a behavioral/performance expansion of `check` that the docs describe only from the `python check` side. Worth an explicit note in the PR/architecture docs that ordinary `check` now executes probes; observation, not a defect.

### MINOR-2 — Duplicated, partially dead report derivation between driver and CLI
`PythonInteropCheckReport.environment` and `.required_import_roots` (`crates/sifr_driver/src/build/report.rs:114`, populated in `python_check.rs:667`) have no consumer: `run_python_read_only_plan` (`python_cli.rs`) recomputes environment status and trust from `context.runtime`/`context.required_imports` instead of using the driver's per-entrypoint answer, and `check_package_project` discards the report entirely. Two authorities now derive "deferred vs resolved"; they agree today (runtime is passed into the plan), but this is drift-prone and `PythonEnvironmentCheck::Resolved { interpreter }` is dead weight. Suggest the CLI consume the driver's environment report or the driver drop the unused fields.

### MINOR-3 — Library inspection cost is quadratic in module count
For library-only packages, `python_inspection_entrypoints` treats every module as an entrypoint and `run_python_read_only_plan` runs the full frontend+codegen plan once per module, cloning the whole graph and source map each iteration (`python_cli.rs`). For a large library this makes `python check` O(modules × import-closure) versus one pass for `sifr check`. Correctness is unaffected (declarations/targets are BTree-merged deterministically); performance observation.

### MINOR-4 — Rust CLI application-parity test can pass vacuously
`crates/sifr/tests/python_read_only_cli.rs:30` — `TestPackage::application()` silently returns `None` (test passes with no assertions) when `uv` is absent or `verification/.venv` is missing, and the whole test is `#[cfg(unix)]`. Mitigated because the blocking `readonly-check-doctor` verification suite fails hard in all four delivery profiles, but a `skipped` signal (e.g. eprintln) would prevent silent erosion.

## Contract verification (evidence)

- **Read-only + shared plan**: `python check`/`doctor` use `CargoLockMode::Frozen`, the same `load_package_graph_context`, `resolve_python_bridge_graph`, `package_python_runtime` (env/lock/trust/certification/probe), and the same driver plan (`check_package_python_interop`) as normal check. Byte-level non-mutation (files + symlink targets) is asserted in both the Rust test and the verification runner, on success and failure. ✓
- **Doctor**: suggestions are static/deterministic, output-only; byte-determinism asserted twice. No mutation paths exist in the command. ✓
- **Library deferral / final-app resolution / multi-target**: verified by the suite (deferred library; app resolves env, trust `verified`, both `math.ceil` and `math.sqrt` app targets `verified` via `runnable_app_paths`). ✓
- **Check/build agreement for applications**: reproduced — app missing `[python]`/trust fails both `python check` and `sifr check` with identical `SIFR-PYTRUST-0005`; invalid target fails both with `SIFR-PYIMP-0001` (suite + my scenario 3). ✓ — except the library-with-selection case in MAJOR-1. ✗
- **Determinism**: `BTreeMap`/`BTreeSet` throughout, `PackageSourceMap.modules` is a `BTreeMap`, so the source-content snapshot digest and report ordering are deterministic. ✓
- **Evidence coverage**: success/failure parity, deterministic doctor, non-mutation, source-digest sensitivity, deferral, final-app, multi-target, all four delivery profiles, demo, and docs are all present; suite passes locally (`python interop read-only check/doctor ok: deferred=1 resolved=1 parity=2 mutations=0`), demo verifies (`math.sqrt: verified`, `suggestions: none`, normal check clean). ✓
- **Guarantees**: no unsafe, no user-triggerable panics (serialize fallback + handled I/O), file sizes ≤ 884 lines (largest: `entrypoint.rs`), guardrail-doc line references (`python_cli.rs:578/635`) confirmed accurate, Ruff submodule untouched. ✓
- **Tracking**: issue plan updated (Waves 1–3 checked, Wave 4 open pending this review); roadmap/architecture closure edits follow the M12 precedent of landing at Wave 4 closure. ✓

MAJOR-1 is a demonstrated contract violation (false `ok` plus misleading doctor guidance on a configuration the compiler path honors), so the milestone cannot be certified as-is; everything else is in good shape.

VERDICT: NOT SATISFIED
