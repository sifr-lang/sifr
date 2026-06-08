RESULT: PASS

**Diff scope (vs origin/main):** clean — fixture `crates/sifr/tests/e2e/pass/process_scoped_parent_cancel.sifr`, manifest entries in both create-pr/merge JSONs, traceability and supported-host matrix doc updates, and ledger entries in the umbrella issue doc. No unrelated changes, no conflict markers.

**Fixture (`process_scoped_parent_cancel.sifr:1-51`)** — Deterministic claim is sound:
- Marker path is PID-scoped (`/tmp/sifr_process_scoped_parent_cancel_<pid>.txt`) and unconditionally cleaned both before and after the assertion.
- The scoped child runs `sh -c "sleep 1; printf not-stopped > <path>"` inside a `task.TaskGroup`. A sibling `group.spawn(fail_parent_scope())` raises `ValueError` after `task.sleep(0.0)`, triggering fail-fast cancellation.
- The 1.20s post-scope wait exceeds the script's 1s sleep, so if the shell child survived cancellation, the marker would exist; the `assert parent_cancelled_process` only passes if the kill actually stopped the child before the delayed write.
- Fixture asserts only the immediate-child side effect — does NOT claim process-group, descendant supervision, or non-Unix semantics.

**Docs alignment** — No overclaims:
- `supported_host_matrix.md` umbrella row drops only "parent cancellation evidence" from remaining work; "Termination escalation and non-Unix status semantics remain" preserved. Scoped subprocess row adds a single sentence scoped to immediate-child fail-fast and explicitly keeps "Windows support requires deterministic follow-up fixtures."
- `concurrency_runtime_m4_process_traceability.md` adds the fixture to the scoped-handle row, scopes the new claim to "stops the scoped child before delayed side effects can escape," updates the status line, and the "Intentional remaining M4 work" list now reads "Non-Unix signal status evidence and supported-host matrix updates for non-Unix process termination behavior" — non-Unix remains open as required.

**Validation evidence (cross-checked against `target/validation_lane_reports/create-pr.latest.json` and `.log`):** all match —
- `real_seconds=137.24` ✓, `advisories=["warm wall-time budget exceeded"]` only ✓, all lane steps `pass` ✓
- log line 1426: `[platform-golden] summary pass=6 skip=1` ✓
- log line 1443: `cache_hits=29/30` ✓
- log line 1453: `report_signature=b11e218d104a7820` ✓
- log line 1454: `114 pass tests completed (114 passed, 0 failed)` ✓

**Non-blocking observation (not a finding):** the same `TaskGroup` + leaking-shell + 1.20s-wait pattern already exists in `process_scoped_spawn_handle.sifr:75-82` as one of its assertions. The new fixture factors that evidence into a dedicated single-purpose fixture for clarity — defensible evidence-hygiene factoring, not redundant.

No blockers for misleading claims, flaky timing (1.20s buffer ≥ 1s sleep), stale wording, unrelated changes, conflict markers, file-size guardrails (fixture 50 lines), or validation mismatch.
