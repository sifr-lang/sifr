Based on my review of the fixes against the changed files, the planning doc DoD, and the pass-1 blockers, here is the assessment.

No blocking issues.

## Non-blocking notes

- **`sifr build` cache key skips probe digest.** `cmd_build` (diagnostic_rendering_and_run.rs:79) → `compile_entrypoint_report` → `build_project_report` → `RootedEntrypoint::Project`, which has no `python_probe_digest`. Only `cmd_check_package_file` and `cmd_run_package_file` go through `package_compiler_context`. Issue doc explicitly scopes the wiring to "check/cached run" (issue line 18), so this matches stated scope — but DoD line 730 ("probe output is part of build cache keys") reads broader. No runtime effect at milestone_py_1 (no Python codegen yet); becomes a real cache-correctness concern at milestone_py_2 when codegen consumes the probe. Worth tracking explicitly before py_2 lands. Either add `cmd_build_package_file` or extend `Commands::Build` to accept package selection.

- **`json_array` silent fallback still present.** environment.rs:442 keeps `unwrap_or_else(|_| "[]".to_string())`. Pass-1 finding 10 unchanged. `Vec<String>` serialization is infallible — use `.expect("...")`, since silent fallbacks violate AGENTS.md "no fallback paths" rule.

- **Test gap: same venv path shared by root and dep.** New tests cover dep-only venv (PYENV-0001) and root-only venv resolves cleanly. No test for "root has `.venv` AND a dep also declares the same `.venv` path" — current code at environment.rs:105 would reject the dep selection even though the venv is identical. Either dedupe by `(venv_root, interpreter)` before the non-root filter, or add a test asserting the current strict behavior is intentional.

- **Pass-1 non-blocking findings 4, 7, 12 still unaddressed.** Validator does not check `pointer_width == usize::BITS`, minimum CPython version, or `soabi`; probe subprocess does not `.env_clear()` (env hygiene); `selections.into_iter().next()` (environment.rs:121) is still BTreeMap-order, not root-preferring — though Fix 3 now narrows this to "all selections are root" so it's effectively single-valued. Acceptable to defer, but worth filing as a milestone_py_2 punch-list item.

- **`PYENV_MULTIPLE_SELECTIONS` precedence.** Confirmed correct: distinct venvs (environment.rs:91–103) fires before non-root filter (105–119) — multiple deps each declaring different non-root venvs report PYENV-0002 first, which matches the stated invariant.

- **Symlink fix.** `canonical_or_normalized` (environment.rs:423) canonicalizes both sides with safe fallback. Fix is correct for the `/var → /private/var` case and uv worktree symlinks.

- **File-size guardrail.** All touched files under 900 (max: entrypoint.rs at 778). Clean.

- **Registry.** SIFR-PYENV-0001..0011 are all wired in `python_interop.rs` (142 lines).
