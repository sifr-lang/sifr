# M17 Wave 2 Review — PR #2998 (`2285b49..6edc0af9`, single commit)

## What I inspected and ran

**Inspections:** full diff of all 21 changed files; the six `.sifr` fixtures and four Python bridges at head; `live_packages.py`, `live_services.py`, `live_case_config.py`, `live_examples.py`, `live_policy.py` in full; `live_policy.json`, `python-interop-live.json` profile, area `manifest.json`, `run.py` dispatch, `env.py`; the trust machinery (`crates/sifr_package/src/python/trust_policy.rs`, `bridge_inventory/imports.rs`) and the runtime bridge loader (`crates/sifr_runtime/src/python/bridge_loader.rs`); Wave 1 fixture conventions; the tracking entry in `plans/issues/active/ad-hoc-declaration-first-python-interop.md`; submodule trees at base and head.

**Commands:** `py_compile` on all changed Python; `run.py --self-test` (passed, includes the new negative self-tests); `run.py --live-policy` (policy-passed, 0 failures); a full **independent `--live-examples` run** — it built and SHA-256-hashed all six binaries (~19–21s each) *before* probing Docker, then produced exactly six `structured-skip` execution cases with `executed_binaries: 0`, `total_failures: 0`, reproducing the tracking claims; and a direct smoke-run of the built redis binary against an unreachable endpoint — it failed closed (exit 1, no marker) with a traceback proving the embedded hermetic bridge (`__sifr_bridge__.p_<hash>.redis_live`), the locked area venv, and typed `PythonError` propagation.

## Contract verification

- **Compiled-binary execution:** every service-client import/operation was removed from the runner; `live_services.py` only manages containers and calls `execute_live_binary`. A new tripwire (`_validate_service_runner_source`) fails the policy suite if client tokens reappear in the runner, with a negative self-test.
- **testcontainers boundary:** only lifecycle + endpoint discovery (`get_connection_url`, `get_bootstrap_server`, `get_url`, host/port); no `get_client(` anywhere.
- **Hermetic typed bridges:** bridges are copied into each package, embedded at build (confirmed by the smoke-run traceback), and declared via typed `@python(bridge.…)` signatures returning `Result[str, PythonError]`.
- **Foreign-thread callback + typed ack:** Kafka/AWS bridges invoke the handler from a real spawned OS thread, rethrow callback exceptions on the declaring thread, and hard-verify `ack:{token}` before returning the marker; fixtures declare `dispatch=foreign, concurrency=serial`. The self-test proves `binary_executed=False` on a "passing" case forces `live-failed`.
- **Build/hash before probe:** verified in code (`build_live_binaries` precedes `probe_docker`) and reproduced empirically.
- **Skip semantics:** skips come only from the Docker probe; container/runtime/bridge errors inside cases become `live-failed`; source/build failures short-circuit with `live-failed` and Docker deliberately unprobed (self-tested). Skips preserve build evidence and promote nothing (`executed_binaries: 0`).
- **Leak enforcement + deterministic markers:** fixtures gate the unique per-case `resources=zero` marker behind a `resource_diagnostics` before/after comparison; bridges close clients in `finally`/context managers; AWS teardown runs every cleanup action, preserves the primary exception, and raises cleanup errors only when the primary path succeeded.
- **Least authority:** per-case `sifr.toml` trust roots exactly match each bridge's direct imports (e.g. redis: `redis` only; AWS: `boto3, json, threading`), enforced by the compiler's AST-level bridge import inventory — the successful builds prove exact coverage. No wildcards.
- **Fail-closed report schema:** new policy keys are required with drift checks and negative self-tests; live-passed cases missing execution evidence, duplicate/missing case ids, all force failure.
- **No injection/timeout gaps:** all subprocess calls use argv lists (no shell), parameters cross via env vars, Postgres uses `sql.Identifier`, build capped at 900s, execution at 120s (kill on expiry), Docker probe at 5s; suite hard cap 1800s fails the lane rather than promoting evidence.
- **third_party/ruff:** submodule pointer identical at base and head (`8111415`); the `m third_party/ruff` in local git status is an uncommitted, semantics-free formatting tweak in the local submodule working tree — not part of the PR and it cannot have affected validation semantics.
- **Tracking claims:** the Wave 2 plans entry matches repository evidence and my independent reproduction.

## Findings

**Blocker:** none.
**Major:** none.
**Minor:** none actionable.

Non-actionable observations (no change requested): `live_services.py:33`'s `"binary_built": case_id in LIVE_CASES` is tautologically true, but it is accurate in the only real call path (builds gate execution) and appears only on `live-failed` records, so it cannot fabricate success evidence; the Kafka bridge's internal timeouts can theoretically sum past the 120s execution cap, but the outcome is still a structured `live-failed` with captured output tails; the runner-boundary check is a substring tripwire rather than a security boundary, which is appropriate for its drift-detection role; a pathologically hung callback would stall interpreter shutdown until the execution timeout kills the binary — still fail-closed.

Prior-pass re-evaluation: the Wave 2 review artifact file in `plans/reviews/active/` is empty, and every concern class named in the task contract was re-examined above; the Docker-unavailable execution gap (binaries never run live on this host) is mitigated by the negative smoke-run I performed, Wave 1's offline foreign-dispatch callback coverage, and the fail-closed evidence gates that force `live-failed` on any Docker-present execution defect.

VERDICT: SATISFIED
