# M2 Opus Review — Pass 1

Review complete. I verified the M2 diff against the plan's tasks/acceptance/validation and inspected the runtime, lowering, digest, docs, and verification surfaces.

## Findings

1. **Registry references a nonexistent test name** — `SIFR-PYTRUST-0003`'s registry entry, `internal_docs/diagnostic_codes.md`, and `docs/errors/SIFR-PYTRUST-0003.md` all cite `native_python_trust_requires_a_canonical_requirement`, but the test is actually `native_trust_requires_a_canonical_requirement`. The generated `code_catalog.json` records the correct name, so the registry is out of sync with reality.
2. **Cache-identity blindspot for derived contributions** — `resolve_python_environment_with_requirements` accepts derived inputs, but cache identity does not cover them. When declarations and bridges contribute roots, cache identity would silently miss them.
3. **Missing uv misclassified as stale metadata** — if `uv` is not on `PATH`, `run_uv_lock_check` maps `NotFound` to `PYENV_LOCK_OR_PROJECT_STALE` with help suggesting `uv sync`. The real cause is a missing tool, not stale lock data.
4. **No integration coverage for the derived-contribution API** — only `canonical_python_requirements` has a unit test; nothing exercises resolution with a `PythonRequirementContribution` and asserts that provenance flows into the `PYTRUST-0005` help.
5. **Unit test panics without uv** — the lock-consistency test shells out unconditionally, so `cargo test -p sifr_package` hard-fails on a machine without uv instead of skipping.

The core M2 cutover is otherwise implemented consistently across parser, resolver, lowering, runtime, docs, and verification surfaces.
