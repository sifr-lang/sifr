Review complete. Below is my structured assessment.

## Findings

### Non-blocking, recommend fix before PR

1. **`pydantic-core-native` in `packages/native.toml:14` is not a real package and duplicates `pydantic-core`.** `pydantic-core` already appears in `tier1.toml:7` with `groups = ["imports", "native"]` and `native = true`. `pydantic-core-native` does not exist on PyPI, so any future milestone that turns matrix names into install/import probes (milestone_py_1 onward) will fail on this entry. Either delete it or rename it to a marker that the runner explicitly understands as a non-installable probe. This is the only finding I'd resolve before opening the PR.

2. **Matrix cannot distinguish Tier 1a from Tier 1b** (milestone_py_11 risk). The phase plan splits `Tier 1a: Core Interop Certification Gate` (mandatory full-gate set) from `Tier 1b: Ecosystem Certification Gate`. The matrix only has `tier = "tier1"`. When milestone_py_11 implements the gate, it needs a way to select the 1a-only set. Cheaper to add a `subtier`/`gate` field now (or split into `tier1a.toml`/`tier1b.toml`) than to retrofit later. Not blocking py_0 acceptance, just flagging.

### Cleanup (mild)

3. **`runner/run.py:6` has an unused `from typing import Any`.**

4. **`--self-test` is additive, not exclusive.** `if args.self_test: run_self_tests(...)` falls through and still runs the matrix selection + writes a report. Harmless, but the validation log shows `--self-test --group scaffold` was used, which suggests the author already noticed this and works around it. Consider short-circuiting `--self-test` so it doesn't also write `reports/latest.json`.

5. **`MATRIX_FILES` is hardcoded in `runner/run.py:16-26`.** Adding a future matrix file (e.g., `tensors.toml`) requires editing the tuple. A `sorted(packages_root.glob("*.toml"))` would remove the duplication, since the runner already validates per-entry tiers/groups.

6. **`summary.total_variants` is hardcoded to `1` in `runner/run.py:88`.** The scaffold report claims one variant regardless of how many packages the filters select. `len(selected)` would be more honest evidence even at scaffold stage.

7. **`reports/latest.json` is the default for every run.** Each invocation overwrites it; concurrent group reports clobber each other. Fine for now; later milestones may want a per-group/per-tier filename derived from the filters.

### Correctness/scope verification (all pass)

- 8 diagnostic family reservations (`PYENV`, `PYIMP`, `PYCALL`, `PYCONV`, `PYRES`, `PYZC`, `PYCB`, `PYTRUST`) appear consistently in `registry.rs` (`DIAGNOSTIC_FAMILIES` and entries), `reserved.rs:12-19`, `docs/errors/diagnostic-codes.md` (Families + Reserved Codes tables), and `internal_docs/diagnostic_codes.md`. No drift.
- All 9 matrix files, all 21 fixture directories, and all 9 runner modules from the phase plan's "Verification Area" section are present.
- All 21 Tier 1a packages are represented in `tier1.toml`.
- Phase plan's `[x] milestone_py_0` claims (scaffold landed, families reserved, links from roadmap/index/architecture) are all verifiable on disk.
- `crates/sifr_lowering/src/lower/ipc_payload_calls.rs` is pure rustfmt drift (single multi-line break of `Some("...".to_string())`), unrelated to py_0 scope but needed for `cargo fmt --check`. Acceptable per task description.
- Runner does not invoke `uv sync` or install packages, matching the plan's "Sifr does not run uv implicitly" rule.

### Probe modules

`native_probe.py`, `callback_probe.py`, `zero_copy_probe.py`, `resource_probe.py` are skeletons containing only a `GROUP`/`GROUPS` constant; they are not imported by `run.py`. This is consistent with scaffold scope, but each will need real implementation in milestones py_5/py_6/py_7-9/py_10 respectively. A one-line `# stub: filled in by milestone_py_N` comment in each would document intent.

## Open Questions

1. Is `pydantic-core-native` in `native.toml` intentional (a placeholder ABI probe?), or leftover? If kept, the runner needs to know not to install it. Recommend removing.
2. Tier 1a vs Tier 1b — settle the matrix representation now (subfield vs split file) so milestone_py_11 doesn't churn the entire matrix layout.
3. `host-dependent = true` packages (e.g., `tensorflow`, `gunicorn`, Tier 4) — does the matrix need a structured `skip_when`/`evidence` field per the plan's "host-dependent test policy", or is that deferred to py_11?
4. Should `--self-test` short-circuit (recommended) or remain additive (current)?

## Verdict

**No blocking issues. milestone_py_0 is acceptable as-is.** Family reservations, scaffold layout, runner contract, package matrices, and roadmap/index/architecture links all hang together cleanly and match the phase plan. Local validation passes. The seven findings above are quality/cleanup items; only finding #1 (`pydantic-core-native`) is worth fixing before opening the PR, since it will produce a real failure the moment milestone_py_1 starts treating matrix names as installable. Everything else can ship and be tightened in follow-ups.
