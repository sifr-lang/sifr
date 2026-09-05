I've now verified the highest-impact claims directly. Here's the synthesized review.

---

# Review: milestone_py_1 (ad-hoc embedded Python interop)

## Blocking findings

**1. `sifr build` does not invoke the Python probe.**
`crates/sifr/src/diagnostic_rendering_and_run.rs:71-96` (`cmd_build`) → `compile_entrypoint_report` at `crates/sifr/src/check_and_package_commands.rs:380-391` → `build_project_report` / `build_single_file_report`. Neither path constructs a `PackageEntrypoint` or calls `package_compiler_context`, so `python_probe_digest` is permanently `None` for `sifr build`. A user running `sifr build` inside a workspace with `[python]` declared silently bypasses every probe assertion (interpreter, free-threaded, venv prefix, site-packages, declared imports, native imports, digest invalidation). Only `sifr check` (via `cmd_check_package_file`) and the package-run path are wired. This contradicts the milestone contract (`plans/issues/active/ad-hoc-embedded-python-interop.md:125,166`).

**2. Verification "fixtures" do not exercise the rejection rules they claim to cover.**
`verification/python_interop/fixtures/env_probe/` contains only `positive_probe.json` and `negative_rules.json`. The "negative" fixture is a flat list of rule-name strings (`env_probe.py:13-24`); `validate_fixture_contract` (`env_probe.py:48-59`) only asserts that the names form a superset of `REQUIRED_NEGATIVE_RULES`. No probe input is ever fed through a rejection path — no PyPy fixture, no free-threaded fixture, no prefix-mismatch fixture, no broken-declared-import fixture. The plan's DoD ("Positive and negative fixtures cover every validation rule") is not met; the runner produces a green status by validating JSON shape, not by exercising probe failure modes.

**3. Manifest `table()` helper silently swallows mistyped `[python]` (and `[trust]`) entries.**
`crates/sifr_package/src/manifest/sifr.rs:180-182`:
```rust
fn table<'a>(value: &'a toml::Table, key: &str) -> Option<&'a toml::Table> {
    value.get(key).and_then(toml::Value::as_table)
}
```
Combined with `unwrap_or_default()` at `sifr.rs:150-153`, a user who writes `python = "venv"`, `python = ["allow-imports"]`, or `[python.allow-imports] = "numpy"` (string instead of array of strings) gets the same outcome as omitting `[python]` entirely — no diagnostic, no probe, generated binary missing the interop bridge. This is the same shape as the previously-fixed silent JSON-array fallback. `table()` should distinguish "absent" from "present-but-wrong-type" and emit `PYENV_INVALID_CONFIG` for the latter.

**4. Declared-import validation only confirms specs are *findable*, not *importable*.**
`crates/sifr_package/src/cargo/python_probe.rs:109-118`: declared imports go through the `do_import=False` branch (`import_probe(root, False)` at line 142). `importlib.util.find_spec` locates a top-level module without executing `__init__.py`, so a broken `__init__.py` (`SyntaxError`, `ImportError` from a missing C extension, side-effect failure) on a declared root will still report `ok: True` because `find_spec` returns a valid spec. Native imports correctly use `do_import=True`. Either flip declared imports to `do_import=True`, or downgrade the contract in the plan from "actually import" to "spec-resolvable."

**5. Site-packages and base-prefix checks do not actually verify venv isolation.**
- `environment.rs:192-199` checks only `probe.site_packages.is_empty()`. A misconfigured environment whose site-packages list points entirely at system directories (no `.venv/lib/.../site-packages` entry) passes. The plan's "Validate site-packages exists in the venv" is not enforced — add: at least one canonicalized site-packages entry must be `path_is_within(venv_root)`.
- `validate_python_environment_probe` never asserts `sys.prefix != sys.base_prefix`. A bare system interpreter pointed at a path that happens to contain a `.venv`-named subdirectory would pass the prefix-within check. CPython's venv contract is `sys.prefix != sys.base_prefix` inside a venv; without it, `[python].interpreter = /usr/bin/python3` silently bypasses isolation.

## Non-blocking observations

- **Diagnostic selection ordering** (`environment.rs:86-118`): the multi-selection check runs before the non-root rejection. If the root declares no venv and two non-root members each declare different venvs, the user gets `PYENV_MULTIPLE_SELECTIONS` ("select exactly one") first, then `PYENV_INVALID_CONFIG` ("only root may select") after they remove one. Both diagnostics are valid; the order is just confusing.
- **Registry template vs runtime drift** (`environment.rs:166,176,185` vs `registry_entries/python_interop.rs` templates): the registry's `{implementation}` / `{import_root}` / `{reason}` placeholders are not what `format!` strings emit at runtime (e.g. `'PyPy'` quoted, `…for embedded Python interop` suffix, `…outside selected venv…`). The hand-authored baseline aligns with the registry, but downstream tools that key off `{placeholder}` extraction will mis-parse real diagnostics. Reconcile by making `format!` strings match the templates verbatim.
- **`features` parsing**: `manifest/sifr.rs:130` uses `value.to_string()` which produces TOML serialization (`"foo"` → `"\"foo\""`, non-strings silently accepted). Out of `[python]` scope but the same class of bug.
- **Dead `unwrap_or` fallbacks** in `sifr.rs:190` and `sifr_fields.rs:116,177` (`rsplit('.').next().unwrap_or(...)` is unreachable for non-empty `&str`).
- **`selections.into_iter().next().ok_or_else(...)` at `environment.rs:120-124`** is unreachable after the prior guards; if it ever fired it would emit a misleading `cargo_metadata_parse` diagnostic.
- **Probe inherits parent env**: `python_probe.rs:23-31` uses `-I` but does not `env_clear()` — `VIRTUAL_ENV`, `PYTHONHOME`, `LD_LIBRARY_PATH`, `DYLD_LIBRARY_PATH` from the user shell can affect the probe. Worth a `milestone_py_2` ticket.
- **Probe data is computed then discarded**: only `digest_python_environment_probe(...).hex` is propagated (`check_and_package_commands.rs:181-192`). Codegen never sees the structured `PythonEnvironmentProbe`. Acceptable for py_1, but py_4+ will need the data plumbed, not re-fetched.
- **Stale plan claims**: `plans/issues/active/ad-hoc-embedded-python-interop.md:12-19`, `plans/phases/index.md:53`, and `plans/roadmap.md:124` mark milestone_py_1 complete with "live environment evidence with positive and negative fixture coverage" — overclaim given finding #2.
- **Diagnostics catalog / registry / docs / baseline alignment for SIFR-PYENV-0001..0011 is clean.** All 11 codes are registered (`registry.rs:64-74,625-635`), present in `code_catalog.json` (lines 2243-2418) and `code_baseline_coverage.json` (lines 1433-1542), each has a docs page, and the package library baseline fixture lists all 11. No duplicates, no orphans.

## Conclusion

**Not satisfied.** Five blocking gaps remain — the most consequential being (1) `sifr build` never invokes the probe, (2) verification fixtures don't exercise rejection rules they claim to cover, (3) manifest silently drops mistyped `[python]` config, (4) declared-import validation is `find_spec`-only, and (5) venv isolation is under-checked (site-packages-within-venv, `sys.prefix != sys.base_prefix`). Diagnostics registry/catalog and graph-resolution semantics are solid; the gaps cluster in production wiring, probe semantics, manifest robustness, and verification coverage.
