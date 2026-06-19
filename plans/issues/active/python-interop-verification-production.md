# Python Interop Verification Productionization

> Status: in progress. This follow-up converts Python interop verification from a standalone matrix/probe runner into a first-class verification area and adds production-grade live dependency examples backed by testcontainers.

## Objective

Make embedded Python interop verification authoritative enough to prove real examples with real dependencies, without overstating matrix-only evidence as live execution.

## Milestones

- [x] `verification_py_area_1`: First-class verification area.
  - Move the Python interop runner and fixtures under `verification/areas/python_interop/`.
  - Add an area `manifest.json` and root `runner.py`.
  - Wire non-container Python interop suites into validation profiles.
  - Update public/internal docs and exit evidence paths.
  - Local validation passed with `scripts/run_all_tests.sh --profile create-pr` on 2026-06-19; Opus review `plans/reviews/active/python-interop-area-migration-review-1.md` returned no blockers.
- [x] `verification_py_area_2`: Container-runtime profile and policy.
  - Add explicit container-runtime resource classification and profile support.
  - Define live Python interop profile semantics, skip rules, and result statuses.
  - Keep offline profiles honest and free of implicit container/runtime requirements.
  - Added `python-interop-live` as an explicit selected-areas-only profile and `live-policy` as the policy gate for later testcontainers suites.
- [ ] `verification_py_area_3`: Live dependency examples.
  - Add testcontainer-backed examples for service dependencies.
  - Cover Redis, Postgres, Kafka-compatible broker, and AWS-compatible SQS/SNS behavior.
  - Run Sifr examples through the embedded Python interop surface against live dependency endpoints.
- [ ] `verification_py_area_4`: Final review and closeout.
  - Run local validation gates.
  - Run Opus review rounds until no blockers remain.
  - Record merged PR links and final evidence.

## Non-Negotiables

- Do not use matrix-only evidence as a substitute for live dependency examples.
- Do not make container-backed tests implicit in offline `create-pr` validation.
- Do not install or sync Python packages implicitly inside the runner.
- Container-backed suites must have explicit resource classification, timeouts, deterministic result artifacts, and cleanup.

## Milestone Evidence

- `verification_py_area_1`: implemented in PR [#2680](https://github.com/sifr-lang/sifr/pull/2680).
- `verification_py_area_2`: implemented on branch `python-interop-container-policy`; PR pending.
