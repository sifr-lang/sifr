## Review of milestone_py_1 fixes

### Verification of fixes

**Fix 1 — `sifr build` now probes Python.** Confirmed at `crates/sifr/src/diagnostic_rendering_and_run.rs:78-111`: `cmd_build` discovers the package session, calls `compile_package_entrypoint_report`, which routes through `package_entrypoint_for_file` → `package_compiler_context` → `package_python_probe_digest`. The digest is fed into `PackageEntrypoint.python_probe_digest`, then `entrypoint.rs:415` stuffs it into `cache_key_fragment`, and `materialize.rs:229-239` includes that fragment in the binary cache key. The new `build_package_project_report` in `api.rs:26-34` and `binary_project_cache_key_includes_package_cache_fragment` test in `materialize.rs:266-283` both check out.

**Fix 2 — Negative env fixtures are concrete, not name-only.** Each entry in `negative_probe_cases.json` carries either a full `probe`+`request` JSON or a `graph` payload, and `env_probe.py:58-94` actually runs `validate_negative_case`/`validate_graph_selection` against them and compares to `expected_rule`. All `REQUIRED_NEGATIVE_RULES` (including `multiple_venv_selection`, `missing_venv_selection`, `probe_execution_failure`) are present.

**Fix 3 — Top-level `python`/`trust` mistypes.** Confirmed: `manifest/sifr.rs:203-220` (`optional_python_table`) emits `PYENV_INVALID_CONFIG` when top-level `python` isn't a table; `optional_table` continues to emit `invalid_sifr_manifest` for `trust`. The new test `mistyped_python_table_reports_pyenv_0001` exercises this path.

**Fix 4 — Declared imports use `import_module`.** `cargo/python_probe.rs:142` calls `import_probe(root, True)` for both `imports` and `native_imports`. `import_probe(root, True)` (line 109-120) goes through `importlib.import_module`, not `find_spec`.

**Fix 5 — Venv isolation enforced.** `probe_validation.rs:29-44` rejects when either `sys_prefix` is outside `venv_root` OR `sys_prefix == sys_base_prefix`; line 46-57 requires ≥1 `site_packages` path inside the venv. Tests `probe_rejects_system_prefix_matching_base_prefix_with_pyenv_0006` and `probe_rejects_site_packages_outside_venv_with_pyenv_0007` cover both new checks. Probe validation now lives in its own `probe_validation.rs` module as stated.

### No blocking correctness issues found.

### Non-blocking observations

1. **Nested `python` array mistypes don't emit PYENV codes.** `sifr_fields.rs:148-169` (`optional_import_root_list`) delegates to `optional_string_list`, which returns `invalid_sifr_manifest` when e.g. `python.allow-imports = "single-string"`. The fix says "nested python fields already emit PYENV diagnostics" — this is true for the relative-path fields (`optional_relative_path` re-wraps as `python_environment_config`), but not for the list-shaped fields. Worth a follow-up to keep diagnostic codes consistent under `[python]`.

2. **`import_module` validates declared roots eagerly.** Loading heavy roots (e.g. `torch`) at probe time will pay full import cost on every cache miss. Intentional given the design contract ("import metadata for declared roots"), but worth a perf comment if torch becomes part of the fast-gate.

3. **`env_probe.py:validate_graph_selection` is a Python re-implementation of the Rust resolver.** If Rust selection logic evolves (e.g., new wildcard or dependency-selection semantics), the Python validator could silently drift. Not a current blocker since the Rust side has its own `tests.rs` coverage, but a cross-language redundancy worth tracking.

4. **`cmd_build` hardcodes `CargoLockMode::Normal`** (no CLI flag honored at the build entry). Pre-existing; not introduced by this milestone.

5. **`probe_validation.rs:120-130` falls back to `normalize_path` when `canonicalize` fails.** Acceptable in unit tests, and live probes resolve through the Python script's `real()` helper first, so the Rust side mostly sees canonicalized paths. Just flagging for awareness on case-insensitive filesystems.

### Verdict: **satisfied**

Milestone_py_1's pass-5 blockers are correctly addressed; the design contract from the issue plan (PYENV-0001 through PYENV-0011, probe-as-source-of-truth, cache invalidation on probe digest, no implicit `uv sync`) is enforced by the code paths and tests reviewed.
