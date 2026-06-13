# M7 Validation Lane And Inventory Closure Review — Pass 1

Result: **FINDINGS** (not ready to commit). The validation-lane audit and the
new closure document are accurate, but running the inventory generator silently
reverts policy text and named evidence that earlier milestones (M2, M3, M5)
landed by hand. The regenerated artifacts contradict closed traceability and
existing e2e fail fixtures, and they remove evidence that the closure document
itself claims is present.

## What is correct

- **Validation lane audit.** `create_pr_e2e_manifest.json` has 125 fixtures and
  `merge_e2e_manifest.json` has 138 fixtures after `spawn_blocking_basic` was
  appended. Every fixture cited in
  `verification/stdlib/concurrency_runtime_m7_inventory_closure.md` is present
  in both lanes. Adding direct `spawn_blocking_basic` to the merge lane is
  appropriate: the create-pr lane already had it, and the merge lane previously
  carried only the indirect `join_set_spawn_blocking` path.
- **Closure document scope.** The new audit is correctly scoped to
  validation-lane and inventory closure and explicitly defers final external
  review and the full merge gate. It does not overclaim M7 or phase
  completion.
- **Legacy / waiver claims.** All 9 entries in
  `legacy_python_shaped_surfaces` carry a non-empty `revisit_rule`, matching
  the closure document's claim. The flake-quarantine claim (only
  `determinism-scale`, owned by `compiler/hardening`, with a re-enable
  criterion) and the "no active concurrency/runtime-owned performance waiver"
  claim are accurate.
- **CPython evidence count.** 135 scanned files, matching the generator's
  printed output and the closure document.
- **Traceability and issue ledger.** Validation lane manifests and Inventory
  closure are correctly marked `pending-pr`; the Required M7 PR Slices row is
  marked `pending PR`; Final external review stays `open`; the M7 doc keeps
  `Status: Open.`; the issue file keeps `M7: in progress.`. No overclaim.

## Blocking findings

### F1 — Semaphore-permit policy regression in regenerated inventory

`scripts/generate_concurrency_runtime_inventory.py`'s hardcoded
`M0_DECISIONS` rewrites the `lock/permit await policy` outcome from

> "Sync lock guards cannot cross await. Async lock guards are await-forbidden
> in M2 unless a specific guard is marked await-safe. Semaphore permits are
> guard-like: they cannot cross await and cannot escape through returns."

to

> "Sync lock guards cannot cross await. Async lock guards are await-forbidden
> in M2 unless a specific guard is marked await-safe. Owned semaphore permits
> may cross await; borrowed permits may not."

This contradicts the closed M2 policy. `verification/stdlib/concurrency_runtime_m2_sync_traceability.md`
states `SemaphorePermit` is "guard-like and await-forbidden in M2; permits
cannot escape through returns," and the policy is enforced by the existing
fail fixtures `crates/sifr/tests/e2e/fail/semaphore_permit_across_await_rejected.sifr`
and `semaphore_permit_escape_rejected.sifr`. The regenerated text would
permit a behavior the compiler currently rejects.

Action: update `M0_DECISIONS["lock/permit await policy"]` in the generator
to match the M2-closed policy (guard-like, cannot cross await, cannot escape
via return) before regenerating, or stop the M7 PR from overwriting the
manually-corrected outcome. The diff appears in both
`concurrency_runtime_substrate_inventory.md` and
`concurrency_runtime_substrate_inventory.json` and must be fixed in both.

### F2 — Workload database silently loses M3 named evidence

The generator's hardcoded `WORKLOAD_ROWS` has not been updated since M0.
Regenerating `concurrency_runtime_workload_database.md` strips rows that
landed across M3 PRs ("Implement M3 spawn_cpu offload wave", "Implement M3
JoinSet runtime wave", "Implement M3 scoped offload runtime wave"):

- `sifr.task.spawn_cpu` row with 6 named fixtures
  (`spawn_cpu_basic`, `spawn_cpu_user_error_typed`, `spawn_cpu_worker_panic_typed`,
  `spawn_cpu_unannotated_rejected`, `spawn_cpu_blocking_io_rejected`,
  `spawn_cpu_non_send_rejected`) is replaced by a single
  `sifr.runtime.spawn_cpu` row whose validation collapses to
  "spawn_cpu typed WorkerError fixture". The API name is also wrong —
  `concurrency_runtime_m3_offload_traceability.md` documents this as
  `sifr.task.spawn_cpu`.
- `sifr.task.TaskScope/TaskGroup scoped offload` (5 named fixtures) is
  removed entirely.
- `sifr.task.JoinSet` (9 named fixtures including
  `join_set_spawn_cpu_join_all_ordered`, `join_set_unconsumed_rejected`,
  `join_set_terminal_must_be_awaited_rejected`, ...) is removed entirely.
- `sifr.parallel.map/try_map` loses its M3 named-fixture detail and
  reverts to "async direct-call diagnostic fixture".

This directly contradicts the closure document's audit claim:

> "A workload database whose accepted APIs have effect classifications and
> named validation evidence."

The regenerated workload database is *less* evidenced than the pre-M7 file.
Same loss appears in `concurrency_runtime_substrate_inventory.json`
under `workload_database`.

Action: update `WORKLOAD_ROWS` in the generator to restore the M3 rows
(naming `sifr.task.spawn_cpu`, `sifr.task.TaskScope/TaskGroup scoped offload`,
and `sifr.task.JoinSet` with their named fixtures, and restoring the
M3 `sifr.parallel.map/try_map` evidence list) before regenerating.

### F3 — Production-surface notes lose M5 detail in inventory

`PRODUCTION_SURFACES` in the generator predates the M5 close-out edits, so
regenerating overwrites three surface notes:

| Surface | Pre-M7 note | Regenerated note |
| --- | --- | --- |
| `sifr.signal structured shutdown streams` | "Portable Signal, SIGINT, SIGTERM, and strsignal value-model evidence is importable; structured streams remain M5 work and arbitrary signal.signal handlers are unsupported." | "Structured streams for supported signals; arbitrary signal.signal handlers are unsupported." |
| `sifr.resource ExitStack/AsyncExitStack/closing/aclosing/nullcontext` | "Deterministic cleanup scopes independent of generator decorator compatibility; `nullcontext(...)` covers no-value and value-carrying generic helper evidence, while ExitStack/AsyncExitStack/closing/aclosing are closed as unsupported diagnostics until cleanup-error and owned-close protocols are implemented." | "Deterministic cleanup scopes independent of generator decorator compatibility." |
| `sifr.task.Context/ContextKey[T]` | "Value-model foundation is importable; explicit propagation remains M5 work with no contextvars parity or implicit dynamic mutation." | "Explicit propagation only; no contextvars parity or implicit dynamic mutation." |

These notes were the M5 record that ExitStack/AsyncExitStack/closing/aclosing
landed *as unsupported diagnostics* (not as supported helpers, despite the
surface heading still listing them) and that `nullcontext(...)` covers
value-carrying generic evidence. Dropping them silently shrinks the
production-surface evidence right when the audit asks readers to rely on it.

Action: update the relevant `PRODUCTION_SURFACES.notes` entries in the
generator to preserve the M5 detail, then regenerate. The diff appears in
both the `.md` and `.json` artifacts.

## Non-blocking observations

- The closure document header reads "Status: M7 inventory closure pending PR."
  while the traceability table uses `pending-pr` and the slice table uses
  `pending PR`. The spelling drift is not a correctness issue but is worth
  unifying on one of the two forms in a single follow-up edit.
- `scripts/generate_concurrency_runtime_inventory.py`'s module docstring still
  reads "Generate M0 concurrency/runtime CPython evidence and Sifr inventory
  artifacts." It is now also responsible for the M2/M3/M5-closed policy text
  it ships; consider updating the docstring at the same time as F1/F2/F3.
- The generator's hardcoded production-surface entry for
  `sifr.runtime spawn_blocking/spawn_cpu/JoinSet` keeps the `sifr.runtime`
  prefix, while M3 traceability documents `sifr.task.spawn_cpu` and
  `task.JoinSet`. This is consistent with the closed inventory and not new in
  this PR, but it adds to the naming drift flagged in F2.

## What to fix before commit

1. Update `M0_DECISIONS` in the generator so the `lock/permit await policy`
   outcome matches the closed M2 policy (F1) — and regenerate.
2. Update `WORKLOAD_ROWS` in the generator to include the M3 `sifr.task.spawn_cpu`,
   `sifr.task.TaskScope/TaskGroup scoped offload`, `sifr.task.JoinSet`, and
   updated `sifr.parallel.map/try_map` rows with their named fixtures (F2) —
   and regenerate.
3. Update the three `PRODUCTION_SURFACES.notes` entries (F3) — and regenerate.
4. Re-run `python3 scripts/generate_concurrency_runtime_inventory.py`,
   `python3 -m json.tool` on the four manifests cited in the ledger,
   `cargo run -q -p sifr -- run crates/sifr/tests/e2e/pass/spawn_blocking_basic.sifr`,
   `git diff --check`, and `python3 scripts/check_file_size_guardrails.py`
   after the generator update; refresh the ledger validation lines with the new
   line counts.
5. After the regeneration regression is gone, re-spin a pass-2 review on the
   refreshed diff before recording the merge ledger.

Until F1–F3 are addressed, the closure document's "named validation evidence"
and the inventory's `lock/permit await policy` outcome do not match the
artifacts they describe, and the PR cannot close the validation-lane /
inventory gate without overclaiming evidence the regenerated files no longer
contain.
