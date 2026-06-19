# Embedded Python Interop Exit Evidence

Status: documentation, evidence, Opus sign-offs, and local validation are complete. PR #2677 merged on 2026-06-19.

## Scope Covered

- Root-owned uv CPython environment selection, probing, and cache metadata.
- Embedded CPython runtime lifecycle, GIL/refcount discipline, object handles, and startup wiring.
- Opaque object import/attribute/item/call helpers, kwargs, Python traceback capture, and typed `Result` errors.
- Primitive, list/tuple/dict, record, bytes, and fixed-width conversion contracts.
- Async blocking classification, offload constraints, and Python coroutine blocking.
- Context-manager cleanup, explicit close/release, resource diagnostics, and double-release coverage.
- `Py_buffer`, Arrow PyCapsule, DLPack, and array-interface zero-copy contracts with explicit copy-vs-view evidence.
- Local and threadsafe Python-to-Sifr callbacks.
- Tier 1 through Tier 4 package certification matrices with deterministic matrix reports and explicit host-dependent skips.
- Public docs, internal architecture docs, diagnostics documentation, and issue tracking updates.

## Diagnostics

Active compiler diagnostic families:

| Family | Codes | Evidence |
| --- | --- | --- |
| `SIFR-PYENV` | `0001..0011` | `crates/sifr_package/src/python/tests.rs`; generated docs under `docs/errors/`; registry rows in `internal_docs/diagnostic_codes.md`. |
| `SIFR-PYTRUST` | `0001..0004` | `crates/sifr_package/src/python/trust_policy_tests.rs`, `crates/sifr_lowering/src/lower/python_trust_tests.rs`; generated docs under `docs/errors/`; registry rows in `internal_docs/diagnostic_codes.md`. |

Reserved runtime-adjacent families:

- `SIFR-PYIMP`
- `SIFR-PYCALL`
- `SIFR-PYCONV`
- `SIFR-PYRES`
- `SIFR-PYZC`
- `SIFR-PYCB`

Runtime failures in those areas currently return structured `py.PythonError` family values. They are not compiler diagnostics unless a future compiler-emitted failure mode requires a stable diagnostic code.

## Verification Commands

Focused Python interop commands:

```bash
python3 -m py_compile verification/python_interop/runner/*.py
verification/python_interop/run.sh --self-test
verification/python_interop/run.sh --group scaffold
verification/python_interop/run.sh --group env
verification/python_interop/run.sh --tier tier1 --report reports/tier1.latest.json
verification/python_interop/run.sh --tier tier2 --report reports/tier2.latest.json
verification/python_interop/run.sh --tier tier3 --report reports/tier3.latest.json
verification/python_interop/run.sh --tier tier4 --report reports/tier4.latest.json
verification/python_interop/run.sh --group callbacks --report reports/callbacks.latest.json
verification/python_interop/run.sh --group dataframes --report reports/dataframes.latest.json
verification/python_interop/run.sh --group cloud --package boto3 --report reports/package.latest.json
```

Repository gates:

```bash
git diff --check
python3 scripts/check_file_size_guardrails.py
python3 scripts/check_hir_maintainability_guardrails.py
python3 verification/areas/coverage_matrix/checks/verification_taxonomy.py
scripts/run_all_tests.sh --profile create-pr
scripts/run_all_tests.sh
```

Latest local validation evidence:

- py11 `create-pr` validation passed on 2026-06-19 with zero failures and advisory `warm wall-time budget exceeded`; this included Python interop package certification code as merged in PR #2676.
- py12 focused validation on 2026-06-19:
  - `python3 -m py_compile verification/python_interop/runner/*.py`
  - `verification/python_interop/run.sh --self-test`
  - `verification/python_interop/run.sh --group scaffold`: `scaffold`
  - `verification/python_interop/run.sh --group env`: `passed`
  - `verification/python_interop/run.sh --tier tier1`: `matrix-passed`, 149 selected, 148 certified, 1 host-dependent skip.
  - `verification/python_interop/run.sh --tier tier4`: `matrix-passed`, 30 selected, 0 certified, 30 host-dependent skips.
  - `verification/python_interop/run.sh --group callbacks`: `matrix-passed`, 5 certified packages.
  - `verification/python_interop/run.sh --group dataframes`: `matrix-passed`, 4 certified packages.
  - `verification/python_interop/run.sh --group cloud --package boto3`: `matrix-passed`, 1 certified package.
- py12 `scripts/run_all_tests.sh --profile create-pr` passed on 2026-06-19:
  - `wall_time=362.24s`, `cpu=222.93s`, `max_rss=390.4MiB`, `swaps=0`.
  - e2e: 132 passed, 0 failed; cache hits `44/44`.
  - hardening summary: 6 variants, 0 failures, 0 blocking failures.
  - advisory: `warm wall-time budget exceeded`.
- py12 `scripts/run_all_tests.sh` default merge gate passed on 2026-06-19:
  - `wall_time=1360.56s`, `cpu=1268.50s`, `max_rss=542.7MiB`, `swaps=0`.
  - e2e: 651 passed, 0 failed; cache hits `182/182`.
  - hardening summary: 260 variants, 0 failures, 0 blocking failures.
  - advisories: `warm wall-time budget exceeded`; `group skew is high; investigate batching balance or fixture clustering`.
  - project-workspace hardening baselines passed after the manifest-level quiet-run support was added for run baselines with deterministic stdout and intentionally empty stderr.
  - note: earlier local attempts were invalidated by stale overlapping validation work; the recorded gates above are the clean authoritative passes.
- Opus sign-offs are recorded in the issue tracker review artifacts with no remaining blockers after documented fixes.

## PR Record

- py0: [#2665](https://github.com/sifr-lang/sifr/pull/2665)
- py1: [#2666](https://github.com/sifr-lang/sifr/pull/2666)
- py2: [#2667](https://github.com/sifr-lang/sifr/pull/2667)
- py3: [#2668](https://github.com/sifr-lang/sifr/pull/2668)
- py4: [#2669](https://github.com/sifr-lang/sifr/pull/2669)
- py5: [#2670](https://github.com/sifr-lang/sifr/pull/2670)
- py6: [#2671](https://github.com/sifr-lang/sifr/pull/2671)
- py7: [#2672](https://github.com/sifr-lang/sifr/pull/2672)
- py8: [#2673](https://github.com/sifr-lang/sifr/pull/2673)
- py9: [#2674](https://github.com/sifr-lang/sifr/pull/2674)
- py10: [#2675](https://github.com/sifr-lang/sifr/pull/2675)
- py11: [#2676](https://github.com/sifr-lang/sifr/pull/2676)
- py12: [#2677](https://github.com/sifr-lang/sifr/pull/2677)
