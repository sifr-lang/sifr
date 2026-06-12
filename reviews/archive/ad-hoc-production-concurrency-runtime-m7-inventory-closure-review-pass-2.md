# M7 Validation Lane And Inventory Closure Review — Pass 2

Result: **PASS** (ready to commit after recording this artifact in the ledger).
All three pass-1 blocking findings have been remediated in the generator and the
regenerated artifacts. The closure document, validation lane manifests, M7
closeout traceability, and ledger remain honest about the open M7 gates.

## Pass-1 findings — verification

### F1 — Semaphore-permit policy preserved (resolved)

`scripts/generate_concurrency_runtime_inventory.py` `M0_DECISIONS["lock/permit
await policy"]` now reads:

> "Sync lock guards cannot cross await. Async lock guards are await-forbidden
> in M2 unless a specific guard is marked await-safe. Semaphore permits are
> guard-like: they cannot cross await and cannot escape through returns."

Evidence rationale was updated in lockstep to "Prevents hidden shared mutable
state and unbounded permit retention across suspension points." This matches
the closed M2 policy in `concurrency_runtime_m2_sync_traceability.md` and the
existing fail fixtures `semaphore_permit_across_await_rejected.sifr` and
`semaphore_permit_escape_rejected.sifr`. Verified in the regenerated
`concurrency_runtime_substrate_inventory.md` row and the matching
`m0_resolved_decisions` entry in `concurrency_runtime_substrate_inventory.json`.

### F2 — M3 workload-database evidence preserved (resolved)

`WORKLOAD_ROWS` now carries the closed M3 rows with their named fixtures:

- `sifr.task.spawn_cpu` (M3, `@cpu_heavy` offload boundary with typed
  runtime/worker evidence) — validation cites `spawn_cpu_basic`,
  `spawn_cpu_user_error_typed`, `spawn_cpu_worker_panic_typed`,
  `spawn_cpu_unannotated_rejected`, `spawn_cpu_blocking_io_rejected`,
  `spawn_cpu_non_send_rejected`.
- `sifr.task.TaskScope/TaskGroup scoped offload` (M3, scoped owner
  `@blocking_io/@cpu_heavy` offload with typed task evidence) — validation
  cites `task_scope_spawn_blocking`, `task_group_spawn_cpu`,
  `task_group_spawn_cpu_user_error`,
  `task_scope_spawn_cpu_unannotated_rejected`,
  `task_group_spawn_blocking_error_mismatch_rejected`.
- `sifr.task.JoinSet` (M3, homogeneous task/offload collection with explicit
  observation/cancellation) — validation cites all nine JoinSet fixtures
  including `join_set_add_task_join_all`,
  `join_set_spawn_cpu_join_all_ordered`, `join_set_cancel_all_evidence`,
  `join_set_cancel_all_task_cancelled`, `join_set_spawn_blocking`,
  `join_set_bound_terminal_await`, `join_set_reassign_live_rejected`,
  `join_set_unconsumed_rejected`,
  `join_set_terminal_must_be_awaited_rejected`.
- `sifr.parallel.map/try_map` (M3, `@cpu_heavy` synchronous, typed
  worker-runtime boundary) — validation cites `parallel_map_basic`,
  `parallel_try_map_basic`, `parallel_map_worker_panic_typed`,
  `parallel_try_map_user_error_typed`, and the async direct-call diagnostic
  fixture.

The regenerated `concurrency_runtime_workload_database.md` and the
`workload_database` array in `concurrency_runtime_substrate_inventory.json`
both reflect these 17 rows. The closure document's "named validation evidence"
claim is now backed by the artifact it points at.

### F3 — M5 production-surface notes preserved (resolved)

`PRODUCTION_SURFACES.notes` now preserves the M5 detail for the three affected
surfaces:

| Surface | Note |
| --- | --- |
| `sifr.signal structured shutdown streams` | "Portable `Signal`, `SIGINT`, `SIGTERM`, and `strsignal` value-model evidence is importable; structured streams remain M5 work and arbitrary signal.signal handlers are unsupported." |
| `sifr.resource ExitStack/AsyncExitStack/closing/aclosing/nullcontext` | "`nullcontext(...)` covers no-value and value-carrying generic helper evidence; ExitStack/AsyncExitStack/closing/aclosing are closed as unsupported diagnostics until cleanup-error and owned-close protocols are implemented." |
| `sifr.task.Context/ContextKey[T]` | "Value-model foundation is importable; explicit propagation remains M5 work with no contextvars parity or implicit dynamic mutation." |

These match the M5 close-out edits and propagate to both
`concurrency_runtime_substrate_inventory.md` and the
`production_surfaces` array in `concurrency_runtime_substrate_inventory.json`.

The non-blocking pass-1 observation about the generator's module docstring is
also addressed — it now reads "Generate concurrency/runtime CPython evidence
and Sifr inventory artifacts." instead of the stale "M0" wording.

## Expected final-state audit

- **Merge manifest fixture.** `verification/validation_lanes/merge_e2e_manifest.json`
  contains `spawn_blocking_basic` (138 fixtures total); create-pr lane is at
  125 fixtures.
- **Closure document.** `verification/stdlib/concurrency_runtime_m7_inventory_closure.md`
  audits the create-pr and merge lanes by family, records the 11
  production-surface and 9 legacy disposition counts, the 135 CPython evidence
  entries, platform-golden concurrency-owned fixtures, the 36 concurrency
  host-matrix rows, and confirms the only flake quarantine is the
  `compiler/hardening`-owned `determinism-scale` template. It does not
  overclaim phase or M7 completion and explicitly defers final external review
  and full merge-gate validation.
- **Generator status text.** `write_inventory_json` writes
  `"milestone_concurrency_runtime_7-inventory-audited"` and `write_inventory_md`,
  `write_evidence_md`, and `write_workload_md` all use `M7 inventory audited`
  status strings. M2/M3/M5-closed evidence is preserved across the generator,
  the regenerated `.md` artifacts, and the regenerated `.json`.
- **Regenerated artifacts.** Re-running `python3 scripts/generate_concurrency_runtime_inventory.py`
  is a no-op against the committed files: substrate inventory MD/JSON, evidence
  matrix, and workload database all match the generator's deterministic output.
- **Traceability.** `concurrency_runtime_m7_closeout_traceability.md` keeps
  `Status: Open.`, marks only `Validation lane manifests` and `Inventory
  closure` as `pending-pr`, the slice row as `pending PR`, and `Final external
  review` as `open`. `M0`-`M6` closure rows remain closed. The issue ledger
  keeps `M7 validation lane and inventory closure: pending PR.` and
  `M7: in progress.`.
- **Ledger validation.** The implementation/validation block at lines
  1592-1607 of the issue file records the regeneration command and entry count
  (135), JSON checks on all five manifests, the F1/F2 assertion ("M2 semaphore
  permit policy is preserved as guard-like and await/return-forbidden; M3
  spawn_cpu, scoped offload, JoinSet, and parallel.map/try_map workload rows
  remain present"), `cargo run` PASS for `spawn_blocking_basic`, and `git diff
  --check` plus `python3 scripts/check_file_size_guardrails.py` PASS.

## Independent re-validation (this review)

- `python3 scripts/generate_concurrency_runtime_inventory.py` → PASS, `generated 135 CPython evidence entries`; subsequent `git diff` against the working tree showed no further changes to the generated artifacts (regeneration is idempotent against the committed state).
- `python3 -m json.tool` on `verification/stdlib/concurrency_runtime_substrate_inventory.json`, `verification/validation_lanes/create_pr_e2e_manifest.json`, `verification/validation_lanes/merge_e2e_manifest.json`, `verification/platform/golden/manifest.json`, and `verification/platform/platform_contract.json` → PASS.
- Validation-lane and inventory assertions → PASS: create-pr `fixture_names` is 125 entries, merge `fixture_names` is 138 entries; `spawn_blocking_basic` is in both; `join_set_spawn_blocking` remains in the merge lane; inventory JSON `status == "milestone_concurrency_runtime_7-inventory-audited"`; production surfaces count is 11, legacy surfaces count is 9, M0 decisions count is 9, workload rows count is 17, scanned files count is 135; F1 semaphore-permit policy text matches the closed M2 wording; F2 rows for `sifr.task.spawn_cpu`, `sifr.task.TaskScope/TaskGroup scoped offload`, and `sifr.task.JoinSet` are all present; F3 notes for signal, resource, and task Context surfaces all preserve M5 detail.
- `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/spawn_blocking_basic.sifr` → PASS (exit code 0; warm cache hit).
- `git diff --check` → PASS (no output).
- `python3 scripts/check_file_size_guardrails.py` → PASS (2273 files, limit 900 lines).

## Non-blocking observations carried forward

- The closure document header still reads "Status: M7 inventory closure
  pending PR." while the traceability and slice tables use `pending-pr` and
  `pending PR`. Pure spelling drift, no correctness impact; unify in a later
  pass if desired.
- The `PRODUCTION_SURFACES` entry for blocking/CPU offload still uses
  `sifr.runtime spawn_blocking/spawn_cpu/JoinSet` while the workload database
  rows now name `sifr.task.spawn_cpu` and `sifr.task.JoinSet`. This naming
  asymmetry is preserved from before this PR and was already noted as
  non-blocking; a later docs slice can align the surface heading with the M3
  traceability names.

## Verdict

PASS. F1/F2/F3 are remediated, regeneration is deterministic against the
committed artifacts, validation-lane manifests and the closure document agree,
M7 closeout traceability and the issue ledger remain honest about the
remaining `Final external review` and merge-gate gates, and local validation
evidence is recorded in the ledger. Safe to commit the M7 validation lane and
inventory closure slice after this pass-2 artifact is added to the ledger.
