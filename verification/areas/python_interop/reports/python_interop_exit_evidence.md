# Embedded Python Interop Exit Evidence

Status: documentation, evidence, Opus sign-offs, and local validation are complete. PR #2677 merged on 2026-06-19.

Additional verification productionization is complete through PR #2683: PR
#2680 moved the runner into `verification/areas/python_interop`, PR #2681 added
explicit container-runtime/live-profile policy, PR #2682 added the opt-in
testcontainers-backed live examples for Redis, Postgres, Kafka-compatible
Redpanda, and LocalStack SNS/SQS, and PR #2683 recorded final status evidence.
Message callback verification now includes first-class Kafka, Pub/Sub-style,
SNS, and SQS examples where Python clients consume messages, while checked Sifr
fixtures pass the consumed Python object to `threadsafe_callback` handlers.

## Scope Covered

- Root-owned uv CPython environment selection, probing, and cache metadata.
- Embedded CPython runtime lifecycle, GIL/refcount discipline, object handles, and startup wiring.
- Opaque object import/attribute/item/call helpers, kwargs, Python traceback capture, and typed `Result` errors.
- Primitive, list/tuple/dict, record, bytes, and fixed-width conversion contracts.
- Async blocking classification, offload constraints, and Python coroutine blocking.
- Application-owned asyncio execution, typed coroutine declarations, native
  cancellation-cause mapping, and consuming async close.
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
| `SIFR-PYTRUST` | `0001`, `0003..0005` | `crates/sifr_package/src/python/trust_policy_tests.rs`, `crates/sifr_lowering/src/lower/python_trust_tests.rs`; generated docs under `docs/errors/`; registry rows in `internal_docs/diagnostic_codes.md`. |
| `SIFR-PYIMP` | `0001..0003` | declaration target validation, package bridge inventory, and reserved runtime collision tests. |
| `SIFR-PYCALL` | `0001` | declaration call-shape lowering and driver target-probe tests. |
| `SIFR-PYCONV` | `0001` | recursive declaration conversion lowering/codegen/runtime tests. |
| `SIFR-PYRES` | `0002` | sequenced declaration activation tests. |
| `SIFR-PYCTX` | `0001` | synchronous context declaration, ownership, and cleanup tests. |

Reserved later-protocol families:

- `SIFR-PYASYNC`
- `SIFR-PYZC`
- `SIFR-PYCB`

Runtime Python exceptions continue to return structured `PythonError` values;
compiler diagnostics own invalid declarations, trust, bridge setup, and protocol
contracts.

## Verification Commands

Focused Python interop commands:

```bash
python3 -m py_compile verification/areas/python_interop/runner/*.py
verification/areas/python_interop/run.sh --self-test
verification/areas/python_interop/run.sh --group scaffold
verification/areas/python_interop/run.sh --group env
verification/areas/python_interop/run.sh --tier tier1 --report ../../../target/verification/areas/python_interop/tier1.latest.json
verification/areas/python_interop/run.sh --tier tier2 --report ../../../target/verification/areas/python_interop/tier2.latest.json
verification/areas/python_interop/run.sh --tier tier3 --report ../../../target/verification/areas/python_interop/tier3.latest.json
verification/areas/python_interop/run.sh --tier tier4 --report ../../../target/verification/areas/python_interop/tier4.latest.json
verification/areas/python_interop/run.sh --group callbacks --report ../../../target/verification/areas/python_interop/callbacks.latest.json
verification/areas/python_interop/run.sh --group dataframes --report ../../../target/verification/areas/python_interop/dataframes.latest.json
verification/areas/python_interop/run.sh --group cloud --package boto3 --report ../../../target/verification/areas/python_interop/package.latest.json
uv run --project verification/areas/python_interop --locked python verification/areas/python_interop/runner/run.py --async-declaration-examples
scripts/run_all_tests.sh --profile python-interop-live
```

## Synchronous Context Activation

The declaration-first `@python.context.enter` / `@python.context.exit` surface
is active. Its normative outcome and cleanup matrix is checked in at
`fixtures/sqlite_context/sync_context_evidence.json`. Focused runtime tests prove
type-sensitive original-exception replay, truthy suppression, nested replay
lifetime, exact-once manager exit, ignored non-Python suppression evidence, and
Python-primary cleanup-failure precedence. Lowering tests cover context-only
obligations, never-entered managers, entered-borrow escape/move/close rejection,
and incompatible distinct opaque entered results.

The registered `sqlite-context` library example is the runnable transaction evidence. It
compiles and runs normal, no-return, early-return, narrowing, break, continue,
and Python-error paths and requires the marker
`sifr-python-interop:sqlite-context:total=71`.

## Hermetic Package Bridge Activation

The `bridge.*` package target namespace is active. Its evidence ledger is
`fixtures/package_bridge_archive/package_bridge_evidence.json`. Focused tests
cover loader-before-main ordering, first-position restoration after
`sys.meta_path` mutation, collision rejection, sibling import rewriting,
deterministic virtual filenames, cache invalidation, invalid syntax, rejected
dynamic imports, misplaced sources, and reserved target ambiguity. Package graph
tests cover root-owned authorization of dependency bridge imports, while the
compiled isolation fixture executes two packages that both own
`bridge.identifiers` under distinct resolved runtime namespaces.

The runnable archive proof packages the biip-backed bridge, unpacks it into a
distinct install root, removes the source checkout before build, removes the
installed bridge source before execution, and runs with an empty read-only
working and temporary directory. It requires the marker
`sifr-python-interop:package-bridge:gtin=7032069804988:format=13:check=8` and is
exposed through the package bridge showcase script.

## Typed Async Declaration Activation

`@python.coroutine(path)` and `cleanup=async_close` are active on one production
path. The generated application owns one asyncio loop thread and one submission
registry; typed functions, factories, methods, package bridges, recursive
conversion, and opaque results submit through it. Focused runtime tests cover
Python failure, conversion and awaitable-shape failure, pre-registration and
in-flight cancellation, Python `finally` ordering, cancellation suppression,
later-exception precedence, independent `CancelledError`, bounded malformed
fallbacks, shutdown drain, and async-close success/failure/poison/exact-once
behavior. Lowering tests own sync/async substitution and affine abandonment,
partial close, duplicate close, and reuse rejection.

The capability ledger points to
`fixtures/async_declaration/async_declaration_evidence.json`. Its compiled
httpx-style client uses a real `httpx.AsyncClient` with an offline ASGI
transport and requires the marker
`sifr-python-interop:async-declaration:status=207:message=async-ready:close=1:loop=shared:failure=covered:conversion=covered`.
The `async-declaration-examples` suite is a blocking selection in every required
validation profile and is exposed through the runnable typed-async demo.
Its first area-runner measurement was 105,034 ms including generated-package
compilation, so the blocking create-PR Python interop step budget is 180,000 ms.

## Typed Async Context Activation

`@python.context.aenter`, `@python.context.aexit`, and
`cleanup=async_context` use the application-owned asyncio runtime.
The compiler retains a dedicated Python async-context HIR and exact-once
manager obligation, replays originating Python exception triples, ignores
truthy decisions for ordinary Sifr/timeout/cancellation/fault causes, masks
async exit until terminal cleanup, and resumes parent cancellation only after
Python `finally` and exit complete. Focused lowering, codegen, and runtime tests
own invalid declarations, unentered obligations, distinct entered-resource
cleanup, direct exit rejection, every concrete body outcome, poison/close,
secondary evidence, biased cancellation, missing fallback, and nested context
envelopes.

The capability ledger points to
`fixtures/async_context/async_context_evidence.json`. Its compiled offline proof
uses real `aiosqlite` over in-memory SQLite and requires the marker
`sifr-python-interop:async-context:value=sqlite-ready:enter=7:exit=7:close=7:loop=shared:suppression=covered:sifr=unsuppressed:cancellation=ordered:nested=lifo:exit-failure=covered`.
The `async-context-examples` suite is blocking in create-PR, merge, nightly, and
release profiles and is exposed through a runnable checked-in demonstration.
Its first isolated compiled run completed in 26.5 seconds and the preceding
create-PR Python interop lane measured 9.5 seconds, so the existing blocking
180,000 ms lane budget remains sufficient and was not increased.

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

- Typed async-context validation on 2026-07-13:
  - `cargo test -p sifr_lowering python_async_context_contract_tests`: 9 passed.
  - `cargo test -p sifr_codegen`: 783 passed.
  - `cargo test -p sifr_runtime --features python python::async_context_tests`: 5 passed.
  - `verification/areas/python_interop/run.sh --self-test`: passed.
  - `uv run --project verification/areas/python_interop --locked python verification/areas/python_interop/runner/run.py --async-context-examples`: passed.
  - The checked-in async-context demonstration script passed with the required
    exact-once, one-loop, suppression, cancellation, nested-ordering, and
    exit-failure marker.
  - `cargo fmt -p sifr_codegen -p sifr_lowering -- --check`, file-size,
    lowering-maintainability, driver-maintainability, verification-taxonomy,
    and `git diff --check` guardrails: passed.
  - `scripts/run_all_tests.sh --profile create-pr`: 130 e2e passed, 0
    failed; Python interop completed in 27,621 ms against its 180,000 ms
    budget; zero blocking failures; advisory `warm wall-time budget exceeded`.
- py11 `create-pr` validation passed on 2026-06-19 with zero failures and advisory `warm wall-time budget exceeded`; this included Python interop package certification code as merged in PR #2676.
- py12 focused validation on 2026-06-19:
  - `python3 -m py_compile verification/areas/python_interop/runner/*.py`
  - `verification/areas/python_interop/run.sh --self-test`
  - `verification/areas/python_interop/run.sh --group scaffold`: `scaffold`
  - `verification/areas/python_interop/run.sh --group env`: `passed`
  - `verification/areas/python_interop/run.sh --tier tier1`: `matrix-passed`, 149 selected, 148 certified, 1 host-dependent skip.
  - `verification/areas/python_interop/run.sh --tier tier4`: `matrix-passed`, 30 selected, 0 certified, 30 host-dependent skips.
  - `verification/areas/python_interop/run.sh --group callbacks`: `matrix-passed`, 5 certified packages.
  - `verification/areas/python_interop/run.sh --group dataframes`: `matrix-passed`, 4 certified packages.
  - `verification/areas/python_interop/run.sh --group cloud --package boto3`: `matrix-passed`, 1 certified package.
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
- Additional verification productionization validation on 2026-06-19:
  - `scripts/run_all_tests.sh --profile create-pr`: passed with zero failures and advisory `warm wall-time budget exceeded`.
  - `scripts/run_all_tests.sh --profile python-interop-live`: passed; live Sifr source checks passed and service cases reported `structured-skip` because the local Docker daemon was unavailable.
  - Final Opus review through `plans/reviews/active/python-interop-live-examples-review-4.md`: no blockers.
- Message callback example validation on 2026-06-19:
  - `scripts/run_all_tests.sh --profile python-interop-live`: passed; Sifr source checks covered Redis, Postgres, Kafka, Pub/Sub-style SNS fanout, SNS, and SQS, then service cases reported `structured-skip` because the local Docker daemon was unavailable.
  - `scripts/run_all_tests.sh --profile create-pr`: passed with zero failures and advisory `warm wall-time budget exceeded`; final e2e pass cache hits were `44/44`.
  - Opus review through `plans/reviews/active/python-interop-message-callback-examples-review-2.md`: no blockers.

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
- verification area migration: [#2680](https://github.com/sifr-lang/sifr/pull/2680)
- verification live policy: [#2681](https://github.com/sifr-lang/sifr/pull/2681)
- verification live examples: [#2682](https://github.com/sifr-lang/sifr/pull/2682)
- verification final status: [#2683](https://github.com/sifr-lang/sifr/pull/2683)
